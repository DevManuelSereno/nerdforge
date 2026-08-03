use std::path::PathBuf;

use crate::vaultconfig::{ensure_vault_config, VaultConfig};

#[allow(dead_code)]
pub const VAULT_FOLDERS: &[&str] = &[
    "npcs",
    "lore",
    "factions",
    "locations",
    "items",
    "scenes",
    "timeline",
    ".nerdforge",
    ".nerdforge/index",
    ".nerdforge/cache",
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultEntry {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub is_file: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultTreeNode {
    pub name: String,
    pub path: String,
    pub is_file: bool,
    pub children: Vec<VaultTreeNode>,
}

pub fn initialize_vault(root: &str) -> Result<Vec<String>, String> {
    let config = ensure_vault_config(root)?;
    initialize_vault_with_folders(root, &config.folder_template.folders)
}

pub fn initialize_vault_with_folders(root: &str, folders: &[String]) -> Result<Vec<String>, String> {
    let root_path = PathBuf::from(root);
    if !root_path.exists() {
        std::fs::create_dir_all(&root_path)
            .map_err(|e| format!("Failed to create vault root: {}", e))?;
    }

    let mut created = Vec::new();
    for folder in folders {
        let path = root_path.join(folder);
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create folder '{}': {}", folder, e))?;
            created.push(folder.to_string());
        }
    }

    Ok(created)
}

pub fn get_vault_structure(root: &str) -> Result<Vec<VaultEntry>, String> {
    let config = load_vault_config_or_default(root)?;
    let root_path = PathBuf::from(root);
    if !root_path.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for folder in &config.folder_template.folders {
        let path = root_path.join(folder);
        entries.push(VaultEntry {
            name: folder.to_string(),
            path: path.to_string_lossy().to_string(),
            exists: path.exists(),
            is_file: path.is_file(),
        });
    }

    Ok(entries)
}

pub fn get_vault_template() -> VaultConfig {
    VaultConfig::default()
}

pub fn scan_vault(root: &str, max_depth: Option<usize>) -> Result<VaultTreeNode, String> {
    let root_path = PathBuf::from(root);
    if !root_path.exists() {
        return Err(format!("Vault root does not exist: {}", root));
    }

    fn scan_dir(path: &std::path::Path, current_depth: usize, max_depth: Option<usize>) -> Result<VaultTreeNode, String> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_file = path.is_file();
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;

        let mut children = Vec::new();
        if metadata.is_dir() {
            let should_descend = match max_depth {
                Some(limit) => current_depth < limit,
                None => true,
            };

            if should_descend {
                let mut entries: Vec<std::path::PathBuf> = Vec::new();
                for entry in std::fs::read_dir(path)
                    .map_err(|e| format!("Failed to read directory {}: {}", path.display(), e))?
                {
                    let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                    entries.push(entry.path());
                }
                entries.sort();

                for entry in entries {
                    children.push(scan_dir(&entry, current_depth + 1, max_depth)?);
                }
            }
        }

        Ok(VaultTreeNode {
            name,
            path: path.to_string_lossy().to_string(),
            is_file,
            children,
        })
    }

    scan_dir(&root_path, 0, max_depth)
}

fn load_vault_config_or_default(root: &str) -> Result<VaultConfig, String> {
    ensure_vault_config(root)
}

#[tauri::command]
pub fn initialize_vault_command(root: String) -> Result<Vec<String>, String> {
    tracing::info!("initializing vault structure at: {}", root);
    initialize_vault(&root)
}

#[tauri::command]
pub fn initialize_vault_with_folders_command(
    root: String,
    folders: Vec<String>,
) -> Result<Vec<String>, String> {
    tracing::info!("initializing vault with {} custom folders at: {}", folders.len(), root);
    initialize_vault_with_folders(&root, &folders)
}

#[tauri::command]
pub fn get_vault_structure_command(root: String) -> Result<Vec<VaultEntry>, String> {
    tracing::info!("getting vault structure for: {}", root);
    get_vault_structure(&root)
}

#[tauri::command]
pub fn get_vault_template_command() -> VaultConfig {
    tracing::info!("getting vault template");
    get_vault_template()
}

#[tauri::command]
pub fn scan_vault_command(root: String, max_depth: Option<usize>) -> Result<VaultTreeNode, String> {
    tracing::info!("scanning vault at: {}", root);
    scan_vault(&root, max_depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vaultconfig::vaultconfig_path;

    #[test]
    fn test_initialize_vault_creates_folders() {
        let dir = std::env::temp_dir().join("nerdforge_vault_init_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        let created = initialize_vault(&root).unwrap();

        assert!(created.contains(&"npcs".to_string()));
        assert!(dir.join("npcs").exists());
        assert!(dir.join(".nerdforge/index").exists());
        assert!(vaultconfig_path(&root).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_vault_structure() {
        let dir = std::env::temp_dir().join("nerdforge_vault_structure_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        initialize_vault(&root).unwrap();
        let entries = get_vault_structure(&root).unwrap();

        assert_eq!(entries.len(), VAULT_FOLDERS.len());
        assert!(entries.iter().all(|e| e.exists));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_vault_tree() {
        let dir = std::env::temp_dir().join("nerdforge_scan_vault_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        initialize_vault(&root).unwrap();
        std::fs::create_dir_all(dir.join("npcs").join("Kael Thornwood")).unwrap();
        std::fs::write(dir.join("npcs").join("Kael Thornwood").join("index.md"), "# Kael").unwrap();

        let tree = scan_vault(&root, Some(2)).unwrap();
        assert_eq!(tree.name, "nerdforge_scan_vault_test");
        assert!(tree.children.iter().any(|c| c.name == "npcs"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_initialize_vault_with_custom_folders() {
        let dir = std::env::temp_dir().join("nerdforge_vault_custom_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        let folders = vec!["notes".to_string(), "drafts".to_string()];
        let created = initialize_vault_with_folders(&root, &folders).unwrap();

        assert!(created.contains(&"notes".to_string()));
        assert!(created.contains(&"drafts".to_string()));
        assert!(dir.join("notes").exists());
        assert!(dir.join("drafts").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
