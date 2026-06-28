use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMarkdown {
    pub frontmatter: HashMap<String, serde_yaml::Value>,
    pub body: String,
    pub title: Option<String>,
    pub module: Option<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedYaml {
    pub data: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

pub fn parse_markdown(content: &str) -> Result<ParsedMarkdown, String> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Ok(ParsedMarkdown {
            frontmatter: HashMap::new(),
            body: content.to_string(),
            title: extract_title_from_body(content),
            module: None,
            status: None,
            tags: Vec::new(),
        });
    }

    let after_first_marker = &trimmed[3..];
    if let Some(end_pos) = after_first_marker.find("\n---") {
        let frontmatter_text = &after_first_marker[..end_pos];
        let body = &after_first_marker[end_pos + 4..];

        let frontmatter: HashMap<String, serde_yaml::Value> =
            serde_yaml::from_str(frontmatter_text)
                .map_err(|e| format!("Failed to parse frontmatter: {}", e))?;

        let title = frontmatter
            .get("title")
            .or_else(|| frontmatter.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| extract_title_from_body(body));

        let module = frontmatter
            .get("module")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let status = frontmatter
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tags = frontmatter
            .get("tags")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ParsedMarkdown {
            frontmatter,
            body: body.trim_start().to_string(),
            title,
            module,
            status,
            tags,
        })
    } else {
        Err("Markdown frontmatter is not closed".to_string())
    }
}

pub fn parse_markdown_file(path: &str) -> Result<ParsedMarkdown, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read markdown file: {}", e))?;
    parse_markdown(&content)
}

pub fn parse_yaml(content: &str) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(content).map_err(|e| format!("Failed to parse YAML: {}", e))
}

pub fn parse_yaml_file(path: &str) -> Result<ParsedYaml, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read YAML file: {}", e))?;
    let data = parse_yaml(&content)?;
    Ok(ParsedYaml { data })
}

fn extract_title_from_body(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}

pub fn validate_visual_dna(value: &serde_yaml::Value) -> ValidationResult {
    let mut errors = Vec::new();

    if value.get("version").is_none() {
        errors.push("Missing field: version".to_string());
    }

    let required_sections = ["face", "skin", "eyes", "hair", "body", "clothing", "art_style"];
    for section in required_sections {
        if value.get(section).is_none() {
            errors.push(format!("Missing section: {}", section));
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

#[tauri::command]
pub fn parse_markdown_file_command(path: String) -> Result<ParsedMarkdown, String> {
    tracing::info!("parsing markdown file: {}", path);
    parse_markdown_file(&path)
}

#[tauri::command]
pub fn parse_yaml_file_command(path: String) -> Result<ParsedYaml, String> {
    tracing::info!("parsing yaml file: {}", path);
    parse_yaml_file(&path)
}

#[tauri::command]
pub fn validate_visual_dna_command(path: String) -> Result<ValidationResult, String> {
    tracing::info!("validating visual-dna file: {}", path);
    let parsed = parse_yaml_file(&path)?;
    Ok(validate_visual_dna(&parsed.data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_with_frontmatter() {
        let content = r#"---
id: kael-thornwood
title: Kael Thornwood
module: npc
status: complete
tags: [npc, faction-thornwood]
---

# Kael Thornwood

Um mercador de informações.
"#;

        let parsed = parse_markdown(content).unwrap();
        assert_eq!(parsed.title, Some("Kael Thornwood".to_string()));
        assert_eq!(parsed.module, Some("npc".to_string()));
        assert_eq!(parsed.status, Some("complete".to_string()));
        assert_eq!(parsed.tags, vec!["npc", "faction-thornwood"]);
        assert!(parsed.body.contains("Um mercador"));
    }

    #[test]
    fn test_parse_markdown_without_frontmatter() {
        let content = "# Apenas Título\n\nCorpo do texto.";
        let parsed = parse_markdown(content).unwrap();
        assert_eq!(parsed.title, Some("Apenas Título".to_string()));
        assert!(parsed.frontmatter.is_empty());
    }

    #[test]
    fn test_parse_yaml() {
        let content = "version: 1\nface:\n  shape: angular";
        let value = parse_yaml(content).unwrap();
        assert_eq!(value["version"].as_i64(), Some(1));
    }

    #[test]
    fn test_validate_visual_dna_valid() {
        let content = r#"version: 1
face:
  shape: angular
skin:
  tone: olive
eyes:
  color: brown
hair:
  color: black
body:
  build: lean
clothing:
  style: rogue
art_style:
  medium: digital
"#;
        let value = parse_yaml(content).unwrap();
        let result = validate_visual_dna(&value);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_visual_dna_invalid() {
        let content = "version: 1\nface:\n  shape: angular";
        let value = parse_yaml(content).unwrap();
        let result = validate_visual_dna(&value);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_parse_yaml_file() {
        let dir = std::env::temp_dir().join("nerdforge_parser_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.yaml").to_string_lossy().to_string();
        std::fs::write(&path, "version: 1\nname: Test").unwrap();

        let parsed = parse_yaml_file(&path).unwrap();
        assert_eq!(parsed.data["version"].as_i64(), Some(1));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
