use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<ChatMessage>,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub history: Vec<ChatMessage>,
    pub conversation_id: Option<String>,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub reply: String,
    pub conversation_id: String,
    pub messages: Vec<ChatMessage>,
    pub error: Option<String>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn conversations_dir(root: &str) -> std::path::PathBuf {
    Path::new(root).join(".nerdforge").join("conversations")
}

fn conversation_path(root: &str, id: &str) -> std::path::PathBuf {
    conversations_dir(root).join(format!("{}.json", id))
}

pub fn load_conversation(root: &str, id: &str) -> Result<Conversation, String> {
    let path = conversation_path(root, id);
    if !path.exists() {
        return Err(format!("Conversa n\u{00e3}o encontrada: {}", id));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_conversation(root: &str, conversation: &Conversation) -> Result<(), String> {
    let dir = conversations_dir(root);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = conversation_path(root, &conversation.id);
    let content = serde_json::to_string_pretty(conversation).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn list_conversations(root: &str) -> Result<Vec<Conversation>, String> {
    let dir = conversations_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut conversations = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(conversation) = serde_json::from_str::<Conversation>(&content) {
                conversations.push(conversation);
            }
        }
    }
    conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(conversations)
}

pub fn chat_with_hermes(root: &str, request: ChatRequest) -> Result<ChatResponse, String> {
    let conversation_id = request
        .conversation_id
        .clone()
        .unwrap_or_else(|| format!("conv_{}", now_secs()));

    let mut conversation = match request.conversation_id.as_deref() {
        Some(id) => load_conversation(root, id).unwrap_or_else(|_| Conversation {
            id: conversation_id.clone(),
            title: request.prompt.lines().next().map(|s| s.to_string()),
            created_at: now_secs(),
            updated_at: now_secs(),
            messages: Vec::new(),
            module: request.module.clone(),
        }),
        None => Conversation {
            id: conversation_id.clone(),
            title: request.prompt.lines().next().map(|s| s.to_string()),
            created_at: now_secs(),
            updated_at: now_secs(),
            messages: Vec::new(),
            module: request.module.clone(),
        },
    };

    conversation.messages.push(ChatMessage {
        role: "user".to_string(),
        content: request.prompt.clone(),
        timestamp: now_secs(),
    });

    let full_prompt = build_prompt(&request.history, &request.prompt);

    let output = run_hermes_cli(&full_prompt)?;
    let reply = output.trim().to_string();

    conversation.messages.push(ChatMessage {
        role: "assistant".to_string(),
        content: reply.clone(),
        timestamp: now_secs(),
    });
    conversation.updated_at = now_secs();

    save_conversation(root, &conversation)?;

    Ok(ChatResponse {
        reply,
        conversation_id,
        messages: conversation.messages.clone(),
        error: None,
    })
}

fn build_prompt(history: &[ChatMessage], prompt: &str) -> String {
    let mut parts = Vec::new();
    for msg in history
        .iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let label = match msg.role.as_str() {
            "user" => "Usu\u{00e1}rio",
            _ => "Assistente",
        };
        parts.push(format!("{}: {}", label, msg.content));
    }
    parts.push(format!("Usu\u{00e1}rio: {}", prompt));
    parts.push("Assistente:".to_string());
    parts.join("\n\n")
}

fn run_hermes_cli(prompt: &str) -> Result<String, String> {
    let program = if cfg!(target_os = "windows") {
        "hermes.exe"
    } else {
        "hermes"
    };

    let mut command = Command::new(program);
    command.arg("chat").arg("-q").arg(prompt).arg("-Q");
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| {
        format!(
            "Falha ao iniciar Hermes CLI: {}. Verifique se 'hermes' est\u{00e1} no PATH.",
            e
        )
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or("N\u{00e3}o foi poss\u{00ed}vel capturar stdout do Hermes")?;

    let reader = BufReader::new(stdout);
    let mut lines = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => lines.push(l),
            Err(e) => return Err(format!("Erro ao ler sa\u{00ed}da do Hermes: {}", e)),
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Erro ao aguardar Hermes: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Hermes CLI retornou c\u{00f3}digo de erro: {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(lines.join("\n"))
}

#[tauri::command]
pub fn chat_with_hermes_command(
    root: String,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    chat_with_hermes(&root, request)
}

#[tauri::command]
pub fn list_conversations_command(root: String) -> Result<Vec<Conversation>, String> {
    list_conversations(&root)
}

#[tauri::command]
pub fn load_conversation_command(root: String, id: String) -> Result<Conversation, String> {
    load_conversation(&root, &id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_prompt_includes_history() {
        let history = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Ol\u{00e1}".to_string(),
                timestamp: 1,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Oi".to_string(),
                timestamp: 2,
            },
        ];
        let prompt = build_prompt(&history, "Como vai?");
        assert!(prompt.contains("Usu\u{00e1}rio: Ol\u{00e1}"));
        assert!(prompt.contains("Assistente: Oi"));
        assert!(prompt.contains("Usu\u{00e1}rio: Como vai?"));
    }

    #[test]
    fn test_save_and_load_conversation() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        let conversation = Conversation {
            id: "conv_1".to_string(),
            title: Some("Teste".to_string()),
            created_at: 1,
            updated_at: 2,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Oi".to_string(),
                timestamp: 1,
            }],
            module: None,
        };
        save_conversation(&root, &conversation).unwrap();
        let loaded = load_conversation(&root, "conv_1").unwrap();
        assert_eq!(loaded.id, "conv_1");
        assert_eq!(loaded.messages.len(), 1);
    }

    #[test]
    fn test_list_conversations_sorted() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        let a = Conversation {
            id: "a".to_string(),
            title: None,
            created_at: 1,
            updated_at: 1,
            messages: vec![],
            module: None,
        };
        let b = Conversation {
            id: "b".to_string(),
            title: None,
            created_at: 1,
            updated_at: 2,
            messages: vec![],
            module: None,
        };
        save_conversation(&root, &a).unwrap();
        save_conversation(&root, &b).unwrap();
        let list = list_conversations(&root).unwrap();
        assert_eq!(list[0].id, "b");
        assert_eq!(list[1].id, "a");
    }
}
