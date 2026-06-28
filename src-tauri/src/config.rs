use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkspaceConfig {
    pub name: String,
    pub root_path: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            name: "Meu Vault".to_string(),
            root_path: dirs::document_dir()
                .map(|p| p.join("NerdForge").to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModuleConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModulesConfig {
    pub npc: ModuleConfig,
    pub lore: ModuleConfig,
}

impl Default for ModulesConfig {
    fn default() -> Self {
        let mut npc_extra = HashMap::new();
        npc_extra.insert(
            "default_style".to_string(),
            serde_yaml::Value::String("dark-fantasy".to_string()),
        );

        Self {
            npc: ModuleConfig {
                enabled: true,
                extra: npc_extra,
            },
            lore: ModuleConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SyncConfig {
    pub engine: String,
    pub embedded: bool,
    pub api_key: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            engine: "syncthing".to_string(),
            embedded: true,
            api_key: "auto-generated".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HermesConfig {
    pub mode: String,
    pub context_window: u32,
    pub auto_summarize: bool,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            mode: "cli".to_string(),
            context_window: 20,
            auto_summarize: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    pub workspace: WorkspaceConfig,
    pub modules: ModulesConfig,
    pub sync: SyncConfig,
    pub hermes: HermesConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig::default(),
            modules: ModulesConfig::default(),
            sync: SyncConfig::default(),
            hermes: HermesConfig::default(),
        }
    }
}

#[tauri::command]
pub fn load_config(path: String) -> Result<AppConfig, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    let config: AppConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    config.validate()?;
    Ok(config)
}

#[tauri::command]
pub fn save_config(path: String, config: AppConfig) -> Result<(), String> {
    config.validate()?;
    let content = serde_yaml::to_string(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn ensure_config(path: String) -> Result<AppConfig, String> {
    if std::path::Path::new(&path).exists() {
        load_config(path)
    } else {
        let config = AppConfig::default();
        save_config(path, config.clone())?;
        Ok(config)
    }
}

#[tauri::command]
pub fn is_first_run(path: String) -> bool {
    !std::path::Path::new(&path).exists()
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.workspace.name.trim().is_empty() {
            return Err("workspace.name cannot be empty".to_string());
        }
        if self.workspace.root_path.trim().is_empty() {
            return Err("workspace.root_path cannot be empty".to_string());
        }
        if self.sync.engine.trim().is_empty() {
            return Err("sync.engine cannot be empty".to_string());
        }
        if self.hermes.mode.trim().is_empty() {
            return Err("hermes.mode cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_default_config_validation() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = std::env::temp_dir().join("nerdforge_config_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.yaml").to_string_lossy().to_string();

        let config = AppConfig::default();
        save_config(path.clone(), config.clone()).unwrap();
        let loaded = load_config(path).unwrap();

        assert_eq!(config, loaded);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ensure_config_creates_default() {
        let dir = std::env::temp_dir().join("nerdforge_ensure_config_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".nerdforge").join("config.yaml").to_string_lossy().to_string();

        let config = ensure_config(path.clone()).unwrap();
        assert!(Path::new(&path).exists());
        assert_eq!(config.workspace.name, "Meu Vault");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validation_errors() {
        let mut config = AppConfig::default();
        config.workspace.name = "   ".to_string();
        assert!(config.validate().is_err());
    }
}
