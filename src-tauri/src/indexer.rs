use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub id: String,
    pub name: String,
    pub module: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub kind: String,
    pub last_modified: Option<u64>,
    pub size: Option<u64>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultIndex {
    pub version: u32,
    pub generated_at: String,
    pub root: String,
    pub entries: Vec<IndexEntry>,
}

pub fn get_index_path(root: &str) -> PathBuf {
    Path::new(root).join(".nerdforge").join("index.json")
}

pub fn rebuild_index(root: &str) -> Result<VaultIndex, String> {
    let root_path = Path::new(root);
    if !root_path.exists() {
        return Err(format!("Vault root does not exist: {}", root));
    }

    let mut entries = Vec::new();
    let modules = ["npcs", "lore", "factions", "locations", "items", "scenes", "timeline"];

    for module in modules {
        let module_path = root_path.join(module);
        if module_path.exists() && module_path.is_dir() {
            scan_directory(&module_path, root_path, module, &mut entries)?;
        }
    }

    let index = VaultIndex {
        version: 1,
        generated_at: chrono::Utc::now().to_rfc3339(),
        root: root.to_string(),
        entries,
    };

    save_index(&index)?;
    Ok(index)
}

fn scan_directory(
    dir: &Path,
    root: &Path,
    module: &str,
    entries: &mut Vec<IndexEntry>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let relative_path = path
            .strip_prefix(root)
            .map_err(|e| format!("Failed to compute relative path: {}", e))?
            .to_string_lossy()
            .to_string()
            .replace("\\", "/");

        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let id = relative_path.replace("/", "-").replace("\\", "-");
        let kind = if metadata.is_dir() { "folder" } else { "file" };

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string().to_lowercase());

        let (title, status, tags) = if kind == "file" && extension.as_deref() == Some("md") {
            match crate::parser::parse_markdown_file(path.to_string_lossy().as_ref()) {
                Ok(parsed) => (parsed.title, parsed.status, parsed.tags),
                Err(e) => {
                    tracing::warn!("failed to parse markdown {}: {}", path.display(), e);
                    (None, None, Vec::new())
                }
            }
        } else {
            (None, None, Vec::new())
        };

        entries.push(IndexEntry {
            id,
            name,
            module: module.to_string(),
            relative_path: relative_path.clone(),
            absolute_path: path.to_string_lossy().to_string(),
            kind: kind.to_string(),
            last_modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            size: if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            },
            title,
            status,
            tags,
        });

        if metadata.is_dir() {
            scan_directory(&path, root, module, entries)?;
        }
    }

    Ok(())
}

pub fn load_index(root: &str) -> Result<VaultIndex, String> {
    let path = get_index_path(root);
    if !path.exists() {
        return rebuild_index(root);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read index file: {}", e))?;
    let index: VaultIndex = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse index file: {}", e))?;
    Ok(index)
}

fn save_index(index: &VaultIndex) -> Result<(), String> {
    let path = get_index_path(&index.root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create index directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(index)
        .map_err(|e| format!("Failed to serialize index: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write index file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn rebuild_index_command(root: String) -> Result<VaultIndex, String> {
    tracing::info!("rebuilding vault index for: {}", root);
    rebuild_index(&root)
}

#[tauri::command]
pub fn get_index_command(root: String) -> Result<VaultIndex, String> {
    tracing::info!("loading vault index for: {}", root);
    load_index(&root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rebuild_index_creates_index_file() {
        let dir = std::env::temp_dir().join("nerdforge_indexer_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs").join("Kael Thornwood")).unwrap();
        std::fs::write(dir.join("npcs").join("Kael Thornwood").join("index.md"), "# Kael").unwrap();
        std::fs::write(dir.join("npcs").join("Lista.md"), "# Lista").unwrap();

        let root = dir.to_string_lossy().to_string();
        let index = rebuild_index(&root).unwrap();

        assert_eq!(index.root, root);
        assert!(index.entries.iter().any(|e| e.name == "index" && e.module == "npcs"));
        assert!(index.entries.iter().any(|e| e.name == "Lista" && e.module == "npcs"));
        assert!(get_index_path(&root).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_index_rebuilds_if_missing() {
        let dir = std::env::temp_dir().join("nerdforge_indexer_load_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("lore")).unwrap();
        std::fs::write(dir.join("lore").join("historia.md"), "# História").unwrap();

        let root = dir.to_string_lossy().to_string();
        let index = load_index(&root).unwrap();

        assert!(index.entries.iter().any(|e| e.name == "historia" && e.module == "lore"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
