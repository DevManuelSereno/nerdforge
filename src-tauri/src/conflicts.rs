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
