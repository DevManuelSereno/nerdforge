use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::npcs::{load_npc_files, NpcFiles};
use crate::structured_output::{generate_structured_output, OutputModule, StructuredOutputRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NpcPart {
    #[serde(rename = "lore")]
    Lore,
    #[serde(rename = "visual")]
    VisualDna,
    #[serde(rename = "prompt")]
    Prompt,
}

impl NpcPart {
    fn as_str(&self) -> &'static str {
        match self {
            NpcPart::Lore => "lore",
            NpcPart::VisualDna => "visual",
            NpcPart::Prompt => "prompt",
        }
    }

    fn file_name(&self) -> &'static str {
        match self {
            NpcPart::Lore => "index.md",
            NpcPart::VisualDna => "visual-dna.yaml",
            NpcPart::Prompt => "prompt.md",
        }
    }

    fn history_extension(&self) -> &'static str {
        match self {
            NpcPart::Lore => "md",
            NpcPart::VisualDna => "yaml",
            NpcPart::Prompt => "md",
        }
    }
}

pub fn regenerate_npc_part(
    root: &str,
    relative_path: &str,
    part: NpcPart,
) -> Result<NpcFiles, String> {
    let base = Path::new(root).join(relative_path);
    if !base.exists() {
        return Err(format!("NPC path does not exist: {}", base.display()));
    }

    let files = load_npc_files(root, relative_path)?;
    save_history(&base, &part, &files)?;

    let request = build_request(&part, &files)?;
    let output = generate_structured_output(request)?;

    write_part(&base, &part, &output)?;

    load_npc_files(root, relative_path)
}

fn save_history(base: &Path, part: &NpcPart, files: &NpcFiles) -> Result<(), String> {
    let history_dir = base.join("history");
    std::fs::create_dir_all(&history_dir)
        .map_err(|e| format!("Failed to create history dir: {}", e))?;

    let version = next_history_version(&history_dir, part)?;
    let file_name = format!(
        "v{}_{}.{}",
        version,
        part.as_str(),
        part.history_extension()
    );
    let content = match part {
        NpcPart::Lore => &files.index_md,
        NpcPart::VisualDna => files.visual_dna_yaml.as_deref().unwrap_or(""),
        NpcPart::Prompt => files.prompt_md.as_deref().unwrap_or(""),
    };

    std::fs::write(history_dir.join(file_name), content)
        .map_err(|e| format!("Failed to write history file: {}", e))
}

fn next_history_version(history_dir: &Path, part: &NpcPart) -> Result<u32, String> {
    let prefix = format!("v");
    let suffix = format!("_{}.{}", part.as_str(), part.history_extension());
    let mut max_version = 0;

    if history_dir.exists() {
        for entry in std::fs::read_dir(history_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name.ends_with(&suffix) {
                let number_part = &name[prefix.len()..name.len() - suffix.len()];
                if let Ok(v) = number_part.parse::<u32>() {
                    max_version = max_version.max(v);
                }
            }
        }
    }

    Ok(max_version + 1)
}

fn build_request(part: &NpcPart, files: &NpcFiles) -> Result<StructuredOutputRequest, String> {
    let context = format!(
        "Personagem atual:\n\nLore:\n{}\n\nVisual DNA:\n{}\n\nPrompt:\n{}",
        files.index_md,
        files.visual_dna_yaml.as_deref().unwrap_or("(nenhum)"),
        files.prompt_md.as_deref().unwrap_or("(nenhum)")
    );

    let prompt = match part {
        NpcPart::Lore => "Regenere APENAS a seção de lore/conceito do NPC, mantendo o nome e a essência do personagem.".to_string(),
        NpcPart::VisualDna => "Regenere APENAS a aparência visual do NPC (visual-dna), mantendo consistente com a personalidade e lore.".to_string(),
        NpcPart::Prompt => "Regenere APENAS o prompt visual do NPC para geração de imagem, com base na aparência e lore atuais.".to_string(),
    };

    Ok(StructuredOutputRequest {
        module: OutputModule::Npc,
        prompt,
        context: Some(context),
    })
}

fn write_part(
    base: &Path,
    part: &NpcPart,
    output: &crate::structured_output::StructuredOutput,
) -> Result<(), String> {
    let path = base.join(part.file_name());
    let content = match part {
        NpcPart::Lore => &output.markdown,
        NpcPart::VisualDna => output
            .visual_dna_yaml
            .as_deref()
            .ok_or("Visual DNA não gerado")?,
        NpcPart::Prompt => output.prompt_md.as_deref().ok_or("Prompt não gerado")?,
    };

    std::fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

pub fn upload_npc_portrait(
    root: &str,
    relative_path: &str,
    base64_image: String,
) -> Result<(), String> {
    let base = Path::new(root).join(relative_path);
    if !base.exists() {
        return Err(format!("NPC path does not exist: {}", base.display()));
    }

    let data = base64_image
        .strip_prefix("data:image/png;base64,")
        .or_else(|| base64_image.strip_prefix("data:image/jpeg;base64,"))
        .or_else(|| base64_image.strip_prefix("data:image/webp;base64,"))
        .unwrap_or(&base64_image);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Invalid base64 image: {}", e))?;

    let path = base.join("portrait.png");
    std::fs::write(&path, bytes).map_err(|e| format!("Failed to write portrait: {}", e))
}

#[tauri::command]
pub fn regenerate_npc_part_command(
    root: String,
    relative_path: String,
    part: NpcPart,
) -> Result<NpcFiles, String> {
    tracing::info!(
        "regenerating npc part: {} for {}",
        part.as_str(),
        relative_path
    );
    regenerate_npc_part(&root, &relative_path, part)
}

#[tauri::command]
pub fn upload_npc_portrait_command(
    root: String,
    relative_path: String,
    base64_image: String,
) -> Result<(), String> {
    tracing::info!("uploading npc portrait for {}", relative_path);
    upload_npc_portrait(&root, &relative_path, base64_image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_history_version_empty() {
        let dir = std::env::temp_dir().join("nerdforge_history_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(next_history_version(&dir, &NpcPart::Lore).unwrap(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_next_history_version_finds_max() {
        let dir = std::env::temp_dir().join("nerdforge_history_max");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("v1_lore.md"), "x").unwrap();
        std::fs::write(dir.join("v3_lore.md"), "x").unwrap();
        std::fs::write(dir.join("v2_visual.yaml"), "x").unwrap();
        assert_eq!(next_history_version(&dir, &NpcPart::Lore).unwrap(), 4);
        assert_eq!(next_history_version(&dir, &NpcPart::VisualDna).unwrap(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_history_creates_file() {
        let dir = std::env::temp_dir().join("nerdforge_history_save");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let files = NpcFiles {
            index_md: "# Kael".to_string(),
            visual_dna_yaml: None,
            prompt_md: None,
        };
        save_history(&dir, &NpcPart::Lore, &files).unwrap();

        let saved = std::fs::read_to_string(dir.join("history").join("v1_lore.md")).unwrap();
        assert_eq!(saved, "# Kael");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
