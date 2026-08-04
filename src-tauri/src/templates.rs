use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::vaultconfig::{load_vault_config, VaultConfig};

pub const TEMPLATES_DIR: &str = ".nerdforge/templates";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Template {
    pub name: String,
    pub module: String,
    pub description: String,
    pub files: Vec<TemplateFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateFile {
    pub relative_path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTemplateResult {
    pub entity_path: String,
    pub files_created: Vec<String>,
}

pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
        .replace("--", "-")
}

pub fn today_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86_400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    format!(
        "{}-{:02}-{:02}",
        year,
        1 + day_of_year / 31,
        1 + day_of_year % 31
    )
}

fn replace_placeholders(content: &str, module: &str, name: &str) -> String {
    let slug = slugify(name);
    let id = slug.replace('-', "_");
    let date = today_iso();

    content
        .replace("{{name}}", name)
        .replace("{{slug}}", &slug)
        .replace("{{id}}", &id)
        .replace("{{date}}", &date)
        .replace("{{module}}", module)
}

pub fn builtin_templates() -> Vec<Template> {
    vec![
        Template {
            name: "Padrão NPC".to_string(),
            module: "npc".to_string(),
            description: "NPC completo com conceito, personalidade, lore e visual DNA.".to_string(),
            files: vec![
                TemplateFile {
                    relative_path: "index.md".to_string(),
                    content: r#"---
id: {{id}}
module: npc
status: draft
tags: [npc]
---

# {{name}}

## Conceito

Escreva aqui uma frase ou parágrafo curto sobre quem é este personagem.

## Personalidade

Como ele se comporta? Quais são seus maneirismos, valores e defeitos?

## Papel Narrativo

Qual a função dele na história? Aliado, vilão, informante, mentor?

## Lore

História, conexões e segredos do personagem.
"#
                    .to_string(),
                },
                TemplateFile {
                    relative_path: "visual-dna.yaml".to_string(),
                    content: r#"version: 1
last_updated: {{date}}

face:
  shape: angular
  features: []

skin:
  tone: olive
  marks: []

eyes:
  color: dark brown
  shape: average
  distinctive: null

hair:
  color: black
  style: short
  length: short

body:
  build: average
  height: average
  age_appearance: adult

clothing:
  style: casual
  colors: [gray, brown]
  distinctive_items: []

art_style:
  medium: digital painting
  mood: dark fantasy
  reference: ""
  aspect_ratio: "3:4"
  orientation: portrait
"#
                    .to_string(),
                },
                TemplateFile {
                    relative_path: "prompt.md".to_string(),
                    content: r#"# Prompt Visual: {{name}}

Personagem dark fantasy. {{name}}, adulto, vestes práticas e discretas.

Estilo: digital painting, atmosfera sombria, luz cinematográfica.
Aspect ratio: 3:4, portrait.
"#
                    .to_string(),
                },
            ],
        },
        Template {
            name: "Padrão Lore".to_string(),
            module: "lore".to_string(),
            description: "Artigo de lore com título, resumo e conteúdo.".to_string(),
            files: vec![TemplateFile {
                relative_path: "index.md".to_string(),
                content: r#"---
id: {{id}}
module: lore
status: draft
tags: [lore]
---

# {{name}}

## Resumo

Resuma o artigo em um ou dois parágrafos.

## Categoria

História, geografia, cultura, magia, religião, etc.

## Conteúdo

Desenvolva o artigo aqui. Use seções se necessário.
"#
                .to_string(),
            }],
        },
        Template {
            name: "Padrão Faction".to_string(),
            module: "faction".to_string(),
            description: "Organização, guilda, culto ou facção do mundo.".to_string(),
            files: vec![TemplateFile {
                relative_path: "index.md".to_string(),
                content: r#"---
id: {{id}}
module: faction
status: draft
tags: [faction]
---

# {{name}}

## Conceito

O que é esta facção? Qual seu propósito público?

## Objetivos

Quais metas ela persegue? Poder, conhecimento, sobrevivência, mudança social?

## Relacionamentos

Como ela se relaciona com outras facções, NPCs e o mundo?

## Estrutura

Liderança, hierarquia, território e recursos.
"#
                .to_string(),
            }],
        },
        Template {
            name: "Padrão Location".to_string(),
            module: "location".to_string(),
            description: "Local, região, cidade ou ponto de interesse.".to_string(),
            files: vec![TemplateFile {
                relative_path: "index.md".to_string(),
                content: r#"---
id: {{id}}
module: location
status: draft
tags: [location]
---

# {{name}}

## Descrição

Como este lugar parece? Clima, arquitetura, geografia.

## Características

Pontos notáveis, recursos, defesas, caminhos.

## Atmosfera

Sensação geral do local. Perigoso, acolhedor, misterioso, decadente?

## Conexões

NPCs, facções e eventos ligados a este local.
"#
                .to_string(),
            }],
        },
    ]
}

fn module_folder(root: &str, module: &str) -> Result<String, String> {
    let config: VaultConfig = load_vault_config(root)?;
    config
        .folder_map
        .get(module)
        .cloned()
        .ok_or_else(|| format!("Módulo não mapeado em vaultconfig.yaml: {}", module))
}

pub fn entity_path(root: &str, module: &str, name: &str) -> Result<PathBuf, String> {
    let folder = module_folder(root, module)?;
    let slug = slugify(name);
    Ok(Path::new(root).join(folder).join(slug))
}

pub fn list_templates(root: &str, module: &str) -> Result<Vec<Template>, String> {
    let mut templates: Vec<Template> = builtin_templates()
        .into_iter()
        .filter(|t| t.module == module)
        .collect();

    if let Ok(custom) = load_custom_templates(root, module) {
        templates.extend(custom);
    }

    Ok(templates)
}

#[allow(dead_code)]
pub fn load_builtin_template(module: &str, name: &str) -> Result<Template, String> {
    builtin_templates()
        .into_iter()
        .find(|t| t.module == module && t.name == name)
        .ok_or_else(|| format!("Template embutido não encontrado: {} / {}", module, name))
}

pub fn load_custom_templates(root: &str, module: &str) -> Result<Vec<Template>, String> {
    let dir = Path::new(root).join(TEMPLATES_DIR).join(module);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(template) = serde_yaml::from_str::<Template>(&content) {
                if template.module == module {
                    templates.push(template);
                }
            }
        }
    }

    Ok(templates)
}

pub fn save_custom_template(root: &str, template: &Template) -> Result<(), String> {
    let dir = Path::new(root).join(TEMPLATES_DIR).join(&template.module);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let slug = slugify(&template.name);
    let path = dir.join(format!("{}.yaml", slug));
    let content = serde_yaml::to_string(template).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn apply_template(
    root: &str,
    template: &Template,
    name: &str,
) -> Result<ApplyTemplateResult, String> {
    let entity_dir = entity_path(root, &template.module, name)?;
    if entity_dir.exists() {
        return Err(format!("Entidade já existe: {}", entity_dir.display()));
    }

    std::fs::create_dir_all(&entity_dir).map_err(|e| e.to_string())?;

    let mut files_created = Vec::new();
    for file in &template.files {
        let content = replace_placeholders(&file.content, &template.module, name);
        let path = entity_dir.join(&file.relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        files_created.push(path.to_string_lossy().to_string());
    }

    Ok(ApplyTemplateResult {
        entity_path: entity_dir.to_string_lossy().to_string(),
        files_created,
    })
}

pub fn create_blank_entity(
    root: &str,
    module: &str,
    name: &str,
) -> Result<ApplyTemplateResult, String> {
    let template = Template {
        name: "blank".to_string(),
        module: module.to_string(),
        description: "Estrutura mínima vazia.".to_string(),
        files: vec![TemplateFile {
            relative_path: "index.md".to_string(),
            content: "---\nid: {{id}}\nmodule: {{module}}\nstatus: draft\ntags: []\n---\n\n# {{name}}\n"
                .to_string(),
        }],
    };
    apply_template(root, &template, name)
}

#[tauri::command]
pub fn list_templates_command(root: String, module: String) -> Result<Vec<Template>, String> {
    tracing::info!("listing templates for module: {}", module);
    list_templates(&root, &module)
}

#[tauri::command]
pub fn apply_template_command(
    root: String,
    template: Template,
    name: String,
) -> Result<ApplyTemplateResult, String> {
    tracing::info!("applying template {} to entity {}", template.name, name);
    apply_template(&root, &template, &name)
}

#[tauri::command]
pub fn create_blank_entity_command(
    root: String,
    module: String,
    name: String,
) -> Result<ApplyTemplateResult, String> {
    tracing::info!("creating blank entity {} for module {}", name, module);
    create_blank_entity(&root, &module, &name)
}

#[tauri::command]
pub fn save_custom_template_command(root: String, template: Template) -> Result<(), String> {
    tracing::info!(
        "saving custom template {} for module {}",
        template.name,
        template.module
    );
    save_custom_template(&root, &template)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_default_vault_folders(root: &str) {
        let _ = std::fs::create_dir_all(Path::new(root).join("npcs"));
        let _ = std::fs::create_dir_all(Path::new(root).join("lore"));
        let _ = std::fs::create_dir_all(Path::new(root).join("factions"));
        let _ = std::fs::create_dir_all(Path::new(root).join("locations"));
        let _ = std::fs::create_dir_all(Path::new(root).join(".nerdforge"));

        let config = VaultConfig {
            version: 1,
            folder_map: crate::vaultconfig::default_folder_map(),
            folder_template: crate::vaultconfig::default_folder_template(),
        };
        let content = serde_yaml::to_string(&config).unwrap();
        let _ = std::fs::write(
            Path::new(root).join(".nerdforge").join("vaultconfig.yaml"),
            content,
        );
    }

    #[test]
    fn test_list_templates_npc() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        create_default_vault_folders(&root);
        let templates = list_templates(&root, "npc").unwrap();
        assert!(templates.iter().any(|t| t.name == "Padrão NPC"));
    }

    #[test]
    fn test_apply_npc_template() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        create_default_vault_folders(&root);
        let template = load_builtin_template("npc", "Padrão NPC").unwrap();
        let result = apply_template(&root, &template, "Kael Thornwood").unwrap();
        assert!(result.entity_path.replace('\\', "/").ends_with("npcs/kael-thornwood"));
        assert!(result
            .files_created
            .iter()
            .any(|p| p.ends_with("index.md")));
        assert!(result
            .files_created
            .iter()
            .any(|p| p.ends_with("visual-dna.yaml")));
        assert!(result.files_created.iter().any(|p| p.ends_with("prompt.md")));

        let index = std::fs::read_to_string(&result.files_created[0]).unwrap();
        assert!(index.contains("# Kael Thornwood"));
        assert!(index.contains("module: npc"));
    }

    #[test]
    fn test_create_blank_entity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        create_default_vault_folders(&root);
        let result = create_blank_entity(&root, "npc", "Test NPC").unwrap();
        assert!(result.entity_path.replace('\\', "/").ends_with("npcs/test-npc"));
        assert_eq!(result.files_created.len(), 1);
        let index = std::fs::read_to_string(&result.files_created[0]).unwrap();
        assert!(index.contains("# Test NPC"));
        assert!(index.contains("module: npc"));
    }

    #[test]
    fn test_apply_template_refuses_duplicate() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        create_default_vault_folders(&root);
        let template = load_builtin_template("npc", "Padrão NPC").unwrap();
        apply_template(&root, &template, "Kael").unwrap();
        assert!(apply_template(&root, &template, "Kael").is_err());
    }

    #[test]
    fn test_save_and_load_custom_template() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        let template = Template {
            name: "Custom NPC".to_string(),
            module: "npc".to_string(),
            description: "Meu template".to_string(),
            files: vec![TemplateFile {
                relative_path: "index.md".to_string(),
                content: "# {{name}}".to_string(),
            }],
        };
        save_custom_template(&root, &template).unwrap();
        let loaded = load_custom_templates(&root, "npc").unwrap();
        assert!(loaded.iter().any(|t| t.name == "Custom NPC"));
    }
}
