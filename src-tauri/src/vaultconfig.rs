use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const VAULTCONFIG_FILE: &str = "vaultconfig.yaml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultConfig {
    pub version: u32,
    #[serde(default = "default_folder_map")]
    pub folder_map: HashMap<String, String>,
    #[serde(default = "default_folder_template")]
    pub folder_template: FolderTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FolderTemplate {
    pub name: String,
    pub folders: Vec<String>,
}

pub fn default_folder_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("npc".to_string(), "npcs".to_string());
    map.insert("lore".to_string(), "lore".to_string());
    map.insert("faction".to_string(), "factions".to_string());
    map.insert("location".to_string(), "locations".to_string());
    map.insert("item".to_string(), "items".to_string());
    map.insert("scene".to_string(), "scenes".to_string());
    map.insert("timeline".to_string(), "timeline".to_string());
    map
}

pub fn default_folder_template() -> FolderTemplate {
    FolderTemplate {
        name: "Padr\u{00e3}o RPG".to_string(),
        folders: vec![
            "npcs".to_string(),
            "lore".to_string(),
            "factions".to_string(),
            "locations".to_string(),
            "items".to_string(),
            "scenes".to_string(),
            "timeline".to_string(),
            ".nerdforge".to_string(),
            ".nerdforge/index".to_string(),
            ".nerdforge/cache".to_string(),
        ],
    }
}

pub fn project_template(template: &str) -> (VaultConfig, Vec<String>) {
    match template {
        "book" => {
            let mut folder_map = HashMap::new();
            folder_map.insert("npc".to_string(), "characters".to_string());
            folder_map.insert("lore".to_string(), "worldbuilding".to_string());
            folder_map.insert("faction".to_string(), "notes".to_string());
            folder_map.insert("location".to_string(), "notes".to_string());
            folder_map.insert("item".to_string(), "notes".to_string());
            folder_map.insert("scene".to_string(), "notes".to_string());
            folder_map.insert("timeline".to_string(), "notes".to_string());

            let folders = vec![
                "chapters".to_string(),
                "characters".to_string(),
                "worldbuilding".to_string(),
                "research".to_string(),
                "notes".to_string(),
                ".nerdforge".to_string(),
                ".nerdforge/index".to_string(),
                ".nerdforge/cache".to_string(),
            ];

            let config = VaultConfig {
                version: 1,
                folder_map,
                folder_template: FolderTemplate {
                    name: "Livro / Romance".to_string(),
                    folders: folders.clone(),
                },
            };
            (config, folders)
        }
        "custom" => {
            let config = VaultConfig::default();
            let folders = config.folder_template.folders.clone();
            (
                VaultConfig {
                    version: 1,
                    folder_map: config.folder_map,
                    folder_template: FolderTemplate {
                        name: "Personalizado".to_string(),
                        folders: folders.clone(),
                    },
                },
                folders,
            )
        }
        _ => {
            let config = VaultConfig::default();
            let folders = config.folder_template.folders.clone();
            (config, folders)
        }
    }
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            version: 1,
            folder_map: default_folder_map(),
            folder_template: default_folder_template(),
        }
    }
}

pub fn vaultconfig_path(root: &str) -> PathBuf {
    Path::new(root).join(".nerdforge").join(VAULTCONFIG_FILE)
}

pub fn load_vault_config(root: &str) -> Result<VaultConfig, String> {
    let path = vaultconfig_path(root);
    if !path.exists() {
        return Ok(VaultConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read vaultconfig: {}", e))?;
    serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse vaultconfig: {}", e))
}

pub fn save_vault_config(root: &str, config: &VaultConfig) -> Result<(), String> {
    let path = vaultconfig_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create vaultconfig directory: {}", e))?;
    }
    let content = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize vaultconfig: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write vaultconfig: {}", e))
}

pub fn ensure_vault_config(root: &str) -> Result<VaultConfig, String> {
    let path = vaultconfig_path(root);
    if path.exists() {
        load_vault_config(root)
    } else {
        let config = VaultConfig::default();
        save_vault_config(root, &config)?;
        Ok(config)
    }
}

#[tauri::command]
pub fn load_vault_config_command(root: String) -> Result<VaultConfig, String> {
    tracing::info!("loading vault config for: {}", root);
    load_vault_config(&root)
}

#[tauri::command]
pub fn save_vault_config_command(root: String, config: VaultConfig) -> Result<(), String> {
    tracing::info!("saving vault config for: {}", root);
    save_vault_config(&root, &config)
}

#[tauri::command]
pub fn ensure_vault_config_command(root: String) -> Result<VaultConfig, String> {
    tracing::info!("ensuring vault config for: {}", root);
    ensure_vault_config(&root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_vault_config() {
        let config = VaultConfig::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.folder_map.get("npc"), Some(&"npcs".to_string()));
        assert_eq!(config.folder_template.name, "Padr\u{00e3}o RPG");
        assert!(config.folder_template.folders.contains(&"npcs".to_string()));
    }

    #[test]
    fn test_ensure_vault_config_creates_file() {
        let dir = std::env::temp_dir().join("nerdforge_vaultconfig_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        let config = ensure_vault_config(&root).unwrap();
        assert!(vaultconfig_path(&root).exists());
        assert_eq!(config.folder_map.get("npc"), Some(&"npcs".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_and_load_vault_config() {
        let dir = std::env::temp_dir().join("nerdforge_vaultconfig_save_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        let mut config = VaultConfig::default();
        config.folder_template.name = "Custom".to_string();
        config.folder_template.folders = vec!["notes".to_string()];
        save_vault_config(&root, &config).unwrap();

        let loaded = load_vault_config(&root).unwrap();
        assert_eq!(loaded, config);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
