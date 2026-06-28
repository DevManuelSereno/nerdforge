use std::path::PathBuf;

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

pub fn initialize_vault(root: &str) -> Result<Vec<String>, String> {
    let root_path = PathBuf::from(root);
    if !root_path.exists() {
        std::fs::create_dir_all(&root_path)
            .map_err(|e| format!("Failed to create vault root: {}", e))?;
    }

    let mut created = Vec::new();
    for folder in VAULT_FOLDERS {
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
    let root_path = PathBuf::from(root);
    if !root_path.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for folder in VAULT_FOLDERS {
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultEntry {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub is_file: bool,
}

#[tauri::command]
pub fn initialize_vault_command(root: String) -> Result<Vec<String>, String> {
    tracing::info!("initializing vault structure at: {}", root);
    initialize_vault(&root)
}

#[tauri::command]
pub fn get_vault_structure_command(root: String) -> Result<Vec<VaultEntry>, String> {
    tracing::info!("getting vault structure for: {}", root);
    get_vault_structure(&root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_vault_creates_folders() {
        let dir = std::env::temp_dir().join("nerdforge_vault_init_test");
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.to_string_lossy().to_string();

        let created = initialize_vault(&root).unwrap();

        assert!(created.contains(&"npcs".to_string()));
        assert!(dir.join("npcs").exists());
        assert!(dir.join(".nerdforge/index").exists());

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
}
