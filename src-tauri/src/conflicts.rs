use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub file_name: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub size: u64,
    pub last_modified: Option<u64>,
}

/// Scan a vault directory recursively for Syncthing conflict files.
/// Syncthing creates files matching `*.sync-conflict-*` when two devices
/// edit the same file simultaneously.
pub fn find_conflicts(root: &str) -> Result<Vec<SyncConflict>, String> {
    let root_path = Path::new(root);
    if !root_path.exists() {
        return Err(format!("Vault path does not exist: {}", root));
    }

    let mut conflicts = Vec::new();
    scan_dir(root_path, root_path, &mut conflicts)?;
    conflicts.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(conflicts)
}

fn scan_dir(current: &Path, root: &Path, out: &mut Vec<SyncConflict>) -> Result<(), String> {
    for entry in std::fs::read_dir(current).map_err(|e| {
        format!("Failed to read directory {}: {}", current.display(), e)
    })? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Skip .nerdforge and .obsidian directories
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
            scan_dir(&path, root, out)?;
        } else if is_conflict_file(&path) {
            let relative_path = path
                .strip_prefix(root)
                .map_err(|e| format!("Failed to compute relative path: {}", e))?
                .to_string_lossy()
                .to_string()
                .replace("\\", "/");

            let file_name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let last_modified = metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            out.push(SyncConflict {
                file_name,
                relative_path,
                absolute_path: path.to_string_lossy().to_string(),
                size,
                last_modified,
            });
        }
    }

    Ok(())
}

fn is_conflict_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    // Syncthing conflict files match: *.sync-conflict-*
    name.contains(".sync-conflict-")
}

#[tauri::command]
pub fn find_conflicts_command(root: String) -> Result<Vec<SyncConflict>, String> {
    tracing::info!("scanning for sync conflicts in: {}", root);
    find_conflicts(&root)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPair {
    pub original_path: String,
    pub conflict_path: String,
    pub original_content: Option<String>,
    pub conflict_content: String,
    pub original_name: String,
    pub conflict_name: String,
}

/// Given a conflict file path, determine the original file path and return both contents.
/// Syncthing names conflict files as: `<name>.sync-conflict-<date>-<time>-<device>.<ext>`
/// The original file is `<name>.<ext>`.
pub fn load_conflict_pair(conflict_path: &str) -> Result<ConflictPair, String> {
    let path = Path::new(conflict_path);
    if !path.exists() {
        return Err(format!("Conflict file does not exist: {}", conflict_path));
    }

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;

    let original_name = derive_original_name(file_name)?;
    let original_path = path
        .parent()
        .ok_or_else(|| "Conflict file has no parent".to_string())?
        .join(&original_name);

    let original_content = if original_path.exists() {
        Some(
            std::fs::read_to_string(&original_path)
                .map_err(|e| format!("Failed to read original: {}", e))?,
        )
    } else {
        None
    };

    let conflict_content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read conflict file: {}", e))?;

    Ok(ConflictPair {
        original_path: original_path.to_string_lossy().to_string(),
        conflict_path: conflict_path.to_string(),
        original_content,
        conflict_content,
        original_name,
        conflict_name: file_name.to_string(),
    })
}

/// Derive the original file name from a Syncthing conflict file name.
/// `index.sync-conflict-20260826-120000-abc.md` → `index.md`
fn derive_original_name(conflict_name: &str) -> Result<String, String> {
    // Find the `.sync-conflict-` marker
    let marker = ".sync-conflict-";
    let marker_pos = conflict_name
        .find(marker)
        .ok_or_else(|| format!("Not a conflict file: {}", conflict_name))?;

    // Everything before the marker is the base name
    let base = &conflict_name[..marker_pos];

    // After the marker, find the last dot to get the extension
    let after_marker = &conflict_name[marker_pos + marker.len()..];
    let ext_pos = after_marker
        .rfind('.')
        .ok_or_else(|| format!("Conflict file has no extension: {}", conflict_name))?;

    let extension = &after_marker[ext_pos..]; // includes the dot

    Ok(format!("{}{}", base, extension))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolveAction {
    #[serde(rename = "keep_local")]
    KeepLocal,
    #[serde(rename = "keep_remote")]
    KeepRemote,
    #[serde(rename = "delete_conflict")]
    DeleteConflict,
}

/// Resolve a conflict file by either:
/// - KeepLocal: delete the conflict file (keep the original as-is)
/// - KeepRemote: overwrite the original with the conflict content, then delete conflict
/// - DeleteConflict: just delete the conflict file (same as keep_local but explicit)
pub fn resolve_conflict(conflict_path: &str, action: ResolveAction) -> Result<(), String> {
    let path = Path::new(conflict_path);
    if !path.exists() {
        return Err(format!("Conflict file does not exist: {}", conflict_path));
    }

    match action {
        ResolveAction::KeepLocal | ResolveAction::DeleteConflict => {
            std::fs::remove_file(path)
                .map_err(|e| format!("Failed to delete conflict file: {}", e))
        }
        ResolveAction::KeepRemote => {
            let pair = load_conflict_pair(conflict_path)?;
            std::fs::write(&pair.original_path, &pair.conflict_content)
                .map_err(|e| format!("Failed to overwrite original: {}", e))?;
            std::fs::remove_file(path)
                .map_err(|e| format!("Failed to delete conflict file: {}", e))
        }
    }
}

#[tauri::command]
pub fn load_conflict_pair_command(conflict_path: String) -> Result<ConflictPair, String> {
    tracing::info!("loading conflict pair: {}", conflict_path);
    load_conflict_pair(&conflict_path)
}

#[tauri::command]
pub fn resolve_conflict_command(
    conflict_path: String,
    action: ResolveAction,
) -> Result<(), String> {
    tracing::info!("resolving conflict: {} ({:?})", conflict_path, action);
    resolve_conflict(&conflict_path, action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_conflict_file() {
        assert!(is_conflict_file(Path::new("index.sync-conflict-20260826-120000-abc.md")));
        assert!(is_conflict_file(Path::new("visual-dna.sync-conflict-20260826.yaml")));
        assert!(!is_conflict_file(Path::new("index.md")));
        assert!(!is_conflict_file(Path::new("visual-dna.yaml")));
        assert!(!is_conflict_file(Path::new("sync-conflict-test.md")));
    }

    #[test]
    fn test_find_conflicts_empty() {
        let dir = std::env::temp_dir().join("nerdforge_conflicts_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let root = dir.to_string_lossy().to_string();
        let conflicts = find_conflicts(&root).unwrap();
        assert!(conflicts.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_conflicts_finds_files() {
        let dir = std::env::temp_dir().join("nerdforge_conflicts_find");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs").join("Kael")).unwrap();

        // Normal file
        std::fs::write(dir.join("npcs").join("Kael").join("index.md"), "# Kael").unwrap();

        // Conflict file
        std::fs::write(
            dir.join("npcs").join("Kael").join("index.sync-conflict-20260826-120000-abc.md"),
            "# Kael (conflict)",
        ).unwrap();

        // Conflict in subfolder
        std::fs::create_dir_all(dir.join("lore").join("History")).unwrap();
        std::fs::write(
            dir.join("lore").join("History").join("article.sync-conflict-20260826.yaml"),
            "conflict",
        ).unwrap();

        let root = dir.to_string_lossy().to_string();
        let conflicts = find_conflicts(&root).unwrap();
        assert_eq!(conflicts.len(), 2);

        let names: Vec<String> = conflicts.iter().map(|c| c.file_name.clone()).collect();
        assert!(names.iter().any(|n| n.contains("index.sync-conflict")));
        assert!(names.iter().any(|n| n.contains("article.sync-conflict")));

        // Verify relative paths
        let kael_conflict = conflicts
            .iter()
            .find(|c| c.file_name.contains("index.sync-conflict"))
            .unwrap();
        assert!(kael_conflict.relative_path.starts_with("npcs/"));
        assert!(kael_conflict.relative_path.contains("Kael"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_original_name() {
        assert_eq!(
            derive_original_name("index.sync-conflict-20260826-120000-abc.md").unwrap(),
            "index.md"
        );
        assert_eq!(
            derive_original_name("visual-dna.sync-conflict-20260826.yaml").unwrap(),
            "visual-dna.yaml"
        );
        assert_eq!(
            derive_original_name("data.sync-conflict-123.json").unwrap(),
            "data.json"
        );
        assert!(derive_original_name("index.md").is_err());
    }

    #[test]
    fn test_load_conflict_pair() {
        let dir = std::env::temp_dir().join("nerdforge_conflict_pair_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("index.md"), "# Local content").unwrap();
        std::fs::write(
            dir.join("index.sync-conflict-20260826-120000-abc.md"),
            "# Remote content",
        ).unwrap();

        let conflict_path = dir.join("index.sync-conflict-20260826-120000-abc.md");
        let pair = load_conflict_pair(conflict_path.to_string_lossy().as_ref()).unwrap();

        assert_eq!(pair.original_name, "index.md");
        assert_eq!(pair.original_content, Some("# Local content".to_string()));
        assert_eq!(pair.conflict_content, "# Remote content");
        assert!(pair.original_path.ends_with("index.md"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_keep_local() {
        let dir = std::env::temp_dir().join("nerdforge_resolve_local_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("index.md"), "# Local").unwrap();
        std::fs::write(dir.join("index.sync-conflict-123.md"), "# Remote").unwrap();

        let conflict_path = dir.join("index.sync-conflict-123.md");
        resolve_conflict(conflict_path.to_string_lossy().as_ref(), ResolveAction::KeepLocal).unwrap();

        // Original stays, conflict deleted
        assert!(dir.join("index.md").exists());
        assert!(!dir.join("index.sync-conflict-123.md").exists());
        assert_eq!(std::fs::read_to_string(dir.join("index.md")).unwrap(), "# Local");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_keep_remote() {
        let dir = std::env::temp_dir().join("nerdforge_resolve_remote_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("index.md"), "# Local").unwrap();
        std::fs::write(dir.join("index.sync-conflict-123.md"), "# Remote").unwrap();

        let conflict_path = dir.join("index.sync-conflict-123.md");
        resolve_conflict(conflict_path.to_string_lossy().as_ref(), ResolveAction::KeepRemote).unwrap();

        // Original overwritten, conflict deleted
        assert!(dir.join("index.md").exists());
        assert!(!dir.join("index.sync-conflict-123.md").exists());
        assert_eq!(std::fs::read_to_string(dir.join("index.md")).unwrap(), "# Remote");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_conflicts_skips_hidden_dirs() {
        let dir = std::env::temp_dir().join("nerdforge_conflicts_hidden");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".nerdforge").join("cache")).unwrap();
        std::fs::create_dir_all(dir.join(".obsidian")).unwrap();

        // Conflict file inside hidden dir (should be skipped)
        std::fs::write(
            dir.join(".nerdforge").join("cache").join("data.sync-conflict-123.md"),
            "hidden conflict",
        ).unwrap();
        std::fs::write(
            dir.join(".obsidian").join("config.sync-conflict-123.json"),
            "hidden",
        ).unwrap();

        // Normal file at root level
        std::fs::write(dir.join("visible.sync-conflict-456.md"), "visible").unwrap();

        let root = dir.to_string_lossy().to_string();
        let conflicts = find_conflicts(&root).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].file_name.contains("visible.sync-conflict"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
