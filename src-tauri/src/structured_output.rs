use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

use crate::hermes::run_hermes_cli;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputModule {
    #[serde(rename = "npc")]
    Npc,
    #[serde(rename = "lore")]
    Lore,
}

impl OutputModule {
    fn as_str(&self) -> &'static str {
        match self {
            OutputModule::Npc => "npc",
            OutputModule::Lore => "lore",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredOutputRequest {
    pub module: OutputModule,
    pub prompt: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredOutput {
    pub module: String,
    pub data: HashMap<String, Value>,
    pub raw_reply: String,
    pub extracted_yaml: String,
    pub markdown: String,
    pub visual_dna_yaml: Option<String>,
    pub prompt_md: Option<String>,
    pub validation_errors: Vec<String>,
}

pub fn generate_structured_output(
    request: StructuredOutputRequest,
) -> Result<StructuredOutput, String> {
    let system_prompt = build_system_prompt(&request);
    let raw_reply = run_hermes_cli(&system_prompt)?;

    let extracted_yaml = extract_yaml(&raw_reply)?;
    let data: HashMap<String, Value> =
        serde_yaml::from_str(&extracted_yaml).map_err(|e| format!("YAML inv\u{00e1}lido: {}", e))?;

    let validation_errors = validate_schema(&request.module, &data);
    let module_name = request.module.as_str().to_string();

    let markdown = format_to_markdown(&request.module, &data);
    let visual_dna_yaml = if matches!(request.module, OutputModule::Npc) {
        Some(format_visual_dna(&data))
    } else {
        None
    };

    let prompt_md = Some(format_prompt_md(&request.module, &data));

    Ok(StructuredOutput {
        module: module_name,
        data,
        raw_reply,
        extracted_yaml,
        markdown,
        visual_dna_yaml,
        prompt_md,
        validation_errors,
    })
}

fn build_system_prompt(request: &StructuredOutputRequest) -> String {
    let module_label = match request.module {
        OutputModule::Npc => "NPC",
        OutputModule::Lore => "artigo de Lore",
    };

    let schema_hint = match request.module {
        OutputModule::Npc => r#"name: string (obrigatório)
concept: string (obrigatório)
lore: string (obrigatório)
personality: string (opcional)
role: string (opcional)
tags: lista de strings (opcional)
visual_appearance: objeto com detalhes visuais (opcional)"#,
        OutputModule::Lore => r#"title: string (obrigatório)
summary: string (obrigatório)
content: string (obrigatório)
category: string (opcional)
tags: lista de strings (opcional)"#,
    };

    let mut prompt = format!(
        "Você é um assistente de criação de conteúdo para RPG e worldbuilding.\n\nGere um {} baseado na solicitação do usuário.\n\nREGRAS:\n1. Retorne APENAS um bloco YAML válido entre os delimitadores ---YAML_START--- e ---YAML_END---.\n2. Não inclua texto explicativo fora dos delimitadores.\n3. Use português brasileiro.\n4. Seja conciso e criativo.\n\nSchema esperado:\n{}\n",
        module_label, schema_hint
    );

    if let Some(ctx) = &request.context {
        prompt.push_str(&format!("\nContexto adicional:\n{}\n", ctx));
    }

    prompt.push_str(&format!(
        "\nSolicitação do usuário:\n{}\n\n---YAML_START---\n",
        request.prompt
    ));

    prompt
}

fn extract_yaml(reply: &str) -> Result<String, String> {
    let start_marker = "---YAML_START---";
    let end_marker = "---YAML_END---";

    let start = reply
    .find(start_marker)
    .ok_or_else(|| "Delimitador ---YAML_START--- não encontrado na resposta".to_string())?
    + start_marker.len();

    let after_start = &reply[start..];
    let end = after_start
    .find(end_marker)
    .ok_or_else(|| "Delimitador ---YAML_END--- não encontrado na resposta".to_string())?;

    Ok(after_start[..end].trim().to_string())
}

fn validate_schema(module: &OutputModule, data: &HashMap<String, Value>) -> Vec<String> {
    let mut errors = Vec::new();

    let required = match module {
        OutputModule::Npc => vec!["name", "concept", "lore"],
        OutputModule::Lore => vec!["title", "summary", "content"],
    };

    for field in required {
        if !data.contains_key(field) {
            errors.push(format!("Campo obrigatório ausente: {}", field));
        } else if let Some(v) = data.get(field) {
            if v.as_str().map(|s| s.trim().is_empty()).unwrap_or(true) {
                errors.push(format!("Campo obrigatório está vazio: {}", field));
            }
        }
    }

    errors
}

fn format_to_markdown(module: &OutputModule, data: &HashMap<String, Value>) -> String {
    let mut frontmatter = serde_yaml::Mapping::new();

    let id_field = match module {
        OutputModule::Npc => data.get("name"),
        OutputModule::Lore => data.get("title"),
    };

    if let Some(id_value) = id_field {
        let id = id_value
            .as_str()
            .map(slugify)
            .unwrap_or_else(|| "entidade".to_string());
        frontmatter.insert(
            serde_yaml::Value::String("id".to_string()),
            serde_yaml::Value::String(id),
        );
    }

    frontmatter.insert(
        serde_yaml::Value::String("module".to_string()),
        serde_yaml::Value::String(module.as_str().to_string()),
    );

    frontmatter.insert(
        serde_yaml::Value::String("status".to_string()),
        serde_yaml::Value::String("draft".to_string()),
    );

    if let Some(tags) = data.get("tags") {
        frontmatter.insert(
            serde_yaml::Value::String("tags".to_string()),
            tags.clone(),
        );
    }

    let mut sections = Vec::new();

    let title = match module {
        OutputModule::Npc => data.get("name").and_then(|v| v.as_str()),
        OutputModule::Lore => data.get("title").and_then(|v| v.as_str()),
    };

    if let Some(t) = title {
        sections.push(format!("# {}\n", t));
    }

    let ordered_fields: Vec<(&str, &str)> = match module {
        OutputModule::Npc => vec![
            ("concept", "Conceito"),
            ("personality", "Personalidade"),
            ("role", "Papel Narrativo"),
            ("lore", "Lore"),
        ],
        OutputModule::Lore => vec![
            ("summary", "Resumo"),
            ("category", "Categoria"),
            ("content", "Conteúdo"),
        ],
    };

    for (field, label) in ordered_fields {
        if let Some(value) = data.get(field).and_then(|v| v.as_str()) {
            if !value.trim().is_empty() {
                sections.push(format!("## {}\n\n{}\n", label, value));
            }
        }
    }

    format!(
        "---\n{}---\n\n{}",
        serde_yaml::to_string(&frontmatter).unwrap_or_default(),
        sections.join("\n")
    )
}

fn format_visual_dna(data: &HashMap<String, Value>) -> String {
    let appearance = data
        .get("visual_appearance")
        .cloned()
        .unwrap_or_else(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    let mapping = if let serde_yaml::Value::Mapping(mut m) = appearance {
        m.insert(
            serde_yaml::Value::String("version".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(1)),
        );
        m.insert(
            serde_yaml::Value::String("last_updated".to_string()),
            serde_yaml::Value::String(today_iso()),
        );
        serde_yaml::Value::Mapping(m)
    } else {
        let mut m = serde_yaml::Mapping::new();
        m.insert(
            serde_yaml::Value::String("version".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(1)),
        );
        m.insert(
            serde_yaml::Value::String("last_updated".to_string()),
            serde_yaml::Value::String(today_iso()),
        );
        serde_yaml::Value::Mapping(m)
    };

    serde_yaml::to_string(&mapping).unwrap_or_default()
}

fn format_prompt_md(module: &OutputModule, data: &HashMap<String, Value>) -> String {
    let title = match module {
        OutputModule::Npc => data.get("name").and_then(|v| v.as_str()).unwrap_or("NPC"),
        OutputModule::Lore => data.get("title").and_then(|v| v.as_str()).unwrap_or("Lore"),
    };

    let mut lines = vec![format!("# Prompt Visual: {}\n", title)];

    if let Some(appearance) = data.get("visual_appearance").and_then(|v| v.as_mapping()) {
        for (k, v) in appearance {
            if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                lines.push(format!("- **{}**: {}", key, value));
            }
        }
    }

    lines.push("\nEstilo: dark fantasy, digital painting.".to_string());
    lines.join("\n")
}

fn slugify(input: &str) -> String {
    input
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
        .replace("--", "-")
}

fn today_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days = secs / 86_400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    format!("{}-{:03}", year, day_of_year)
}

#[tauri::command]
pub fn generate_structured_output_command(
    request: StructuredOutputRequest,
) -> Result<StructuredOutput, String> {
    tracing::info!(
        "generating structured output for module: {}",
        request.module.as_str()
    );
    generate_structured_output(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_simple() {
        let reply = "Algum texto\n---YAML_START---\nname: Kael\nconcept: Mercador\n---YAML_END---\nMais texto";
        let yaml = extract_yaml(reply).unwrap();
        assert!(yaml.contains("name: Kael"));
        assert!(yaml.contains("concept: Mercador"));
    }

    #[test]
    fn test_extract_yaml_missing_start() {
        let reply = "não tem delimitador";
        assert!(extract_yaml(reply).is_err());
    }

    #[test]
    fn test_validate_schema_npc_missing_fields() {
        let data = HashMap::new();
        let errors = validate_schema(&OutputModule::Npc, &data);
        assert!(errors.iter().any(|e| e.contains("name")));
        assert!(errors.iter().any(|e| e.contains("concept")));
        assert!(errors.iter().any(|e| e.contains("lore")));
    }

    #[test]
    fn test_validate_schema_npc_valid() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("Kael".to_string()));
        data.insert("concept".to_string(), Value::String("Mercador".to_string()));
        data.insert("lore".to_string(), Value::String("Nascido nas docas.".to_string()));
        let errors = validate_schema(&OutputModule::Npc, &data);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_format_to_markdown_npc() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("Kael Thornwood".to_string()));
        data.insert("concept".to_string(), Value::String("Mercador de informações.".to_string()));
        data.insert("lore".to_string(), Value::String("Nascido nas docas.".to_string()));
        data.insert("personality".to_string(), Value::String("Calculista.".to_string()));
        data.insert(
            "tags".to_string(),
            Value::Sequence(vec![
                Value::String("npc".to_string()),
                Value::String("valdris".to_string()),
            ]),
        );

        let md = format_to_markdown(&OutputModule::Npc, &data);
        assert!(md.contains("# Kael Thornwood"));
        assert!(md.contains("id: kael-thornwood"));
        assert!(md.contains("module: npc"));
        assert!(md.contains("## Conceito"));
        assert!(md.contains("## Personalidade"));
        assert!(md.contains("## Lore"));
    }

    #[test]
    fn test_format_to_markdown_lore() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), Value::String("A Queda de Valdris".to_string()));
        data.insert("summary".to_string(), Value::String("Resumo.".to_string()));
        data.insert("content".to_string(), Value::String("Conteúdo longo.".to_string()));
        data.insert("category".to_string(), Value::String("História".to_string()));

        let md = format_to_markdown(&OutputModule::Lore, &data);
        assert!(md.contains("# A Queda de Valdris"));
        assert!(md.contains("id: a-queda-de-valdris"));
        assert!(md.contains("module: lore"));
        assert!(md.contains("## Resumo"));
        assert!(md.contains("## Categoria"));
        assert!(md.contains("## Conteúdo"));
    }
}
