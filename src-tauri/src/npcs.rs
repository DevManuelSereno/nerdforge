use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub last_modified: Option<u64>,
}

pub fn list_npcs(root: &str) -> Result<Vec<NpcSummary>, String> {
    let config = crate::vaultconfig::ensure_vault_config(root)?;
    let npc_folder = config
        .folder_map
        .get("npc")
        .cloned()
        .unwrap_or_else(|| "npcs".to_string());
    let npc_path = Path::new(root).join(&npc_folder);

    if !npc_path.exists() {
        std::fs::create_dir_all(&npc_path)
            .map_err(|e| format!("Failed to create npc folder: {}", e))?;
    }

    if !npc_path.is_dir() {
        return Err(format!("Npc path is not a directory: {}", npc_path.display()));
    }

    let mut npcs = Vec::new();
    for entry in std::fs::read_dir(&npc_path)
        .map_err(|e| format!("Failed to read npc folder: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let index_md = path.join("index.md");
        let (status, tags) = if index_md.exists() {
            match crate::parser::parse_markdown_file(index_md.to_string_lossy().as_ref()) {
                Ok(parsed) => (parsed.status, parsed.tags),
                Err(e) => {
                    tracing::warn!("failed to parse npc index {}: {}", index_md.display(), e);
                    (None, Vec::new())
                }
            }
        } else {
            (None, Vec::new())
        };

        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let slug = slugify(&name);
        let relative_path = path
            .strip_prefix(root)
            .map_err(|e| format!("Failed to compute relative path: {}", e))?
            .to_string_lossy()
            .to_string()
            .replace("\\", "/");
        let id = relative_path.replace("/", "-");
        let last_modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        npcs.push(NpcSummary {
            id,
            name,
            slug,
            relative_path,
            absolute_path: path.to_string_lossy().to_string(),
            status,
            tags,
            last_modified,
        });
    }

    npcs.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(npcs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcFiles {
    pub index_md: String,
    pub visual_dna_yaml: Option<String>,
    pub prompt_md: Option<String>,
}

pub fn load_npc_files(root: &str, relative_path: &str) -> Result<NpcFiles, String> {
    let base = Path::new(root).join(relative_path);
    let index_md = std::fs::read_to_string(base.join("index.md"))
        .map_err(|e| format!("Failed to read index.md: {}", e))?;

    let visual_dna_yaml = std::fs::read_to_string(base.join("visual-dna.yaml")).ok();
    let prompt_md = std::fs::read_to_string(base.join("prompt.md")).ok();

    Ok(NpcFiles {
        index_md,
        visual_dna_yaml,
        prompt_md,
    })
}

#[tauri::command]
pub fn load_npc_files_command(root: String, relative_path: String) -> Result<NpcFiles, String> {
    tracing::info!("loading npc files for: {}", relative_path);
    load_npc_files(&root, &relative_path)
}

#[tauri::command]
pub fn list_npcs_command(root: String) -> Result<Vec<NpcSummary>, String> {
    tracing::info!("listing npcs for: {}", root);
    list_npcs(&root)
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_npcs_empty_folder() {
        let dir = std::env::temp_dir().join("nerdforge_npcs_empty_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let root = dir.to_string_lossy().to_string();
        let npcs = list_npcs(&root).unwrap();
        assert!(npcs.is_empty());
        assert!(dir.join("npcs").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_npcs_with_two_npcs() {
        let dir = std::env::temp_dir().join("nerdforge_npcs_list_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs").join("Kael Thornwood")).unwrap();
        std::fs::write(
            dir.join("npcs").join("Kael Thornwood").join("index.md"),
            "---\nstatus: complete\ntags: [npc, merchant]\n---\n\n# Kael Thornwood\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("npcs").join("Lira Voss")).unwrap();
        std::fs::write(
            dir.join("npcs").join("Lira Voss").join("index.md"),
            "---\nstatus: draft\ntags: [npc]\n---\n\n# Lira Voss\n",
        )
        .unwrap();

        let root = dir.to_string_lossy().to_string();
        let npcs = list_npcs(&root).unwrap();
        assert_eq!(npcs.len(), 2);
        let names: Vec<String> = npcs.iter().map(|n| n.name.clone()).collect();
        assert!(names.contains(&"Kael Thornwood".to_string()));
        assert!(names.contains(&"Lira Voss".to_string()));

        let kael = npcs.iter().find(|n| n.name == "Kael Thornwood").unwrap();
        assert_eq!(kael.status, Some("complete".to_string()));
        assert_eq!(kael.tags, vec!["npc", "merchant"]);
        assert_eq!(kael.slug, "kael-thornwood");
        assert!(kael.relative_path.starts_with("npcs/"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_npcs_creates_missing_folder() {
        let dir = std::env::temp_dir().join("nerdforge_npcs_missing_folder_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let root = dir.to_string_lossy().to_string();
        let npcs = list_npcs(&root).unwrap();
        assert!(npcs.is_empty());
        assert!(dir.join("npcs").is_dir());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_npc_files_reads_all() {
        let dir = std::env::temp_dir().join("nerdforge_load_npc_files_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs").join("Kael Thornwood")).unwrap();
        std::fs::write(
            dir.join("npcs").join("Kael Thornwood").join("index.md"),
            "---\nstatus: complete\ntags: [npc, merchant]\n---\n\n# Kael Thornwood\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("npcs").join("Kael Thornwood").join("visual-dna.yaml"),
            "style: dark\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("npcs").join("Kael Thornwood").join("prompt.md"),
            "Generate an NPC named Kael.\n",
        )
        .unwrap();

        std::fs::write(
            dir.join("vaultconfig.yaml"),
            "version: 1\nfolder_map:\n  npc: npcs\nfolder_template:\n  name: Padrão RPG\n  folders:\n    - npcs\n",
        )
        .unwrap();

        let root = dir.to_string_lossy().to_string();
        let files = load_npc_files(&root, "npcs/Kael Thornwood").unwrap();
        assert!(files.index_md.contains("# Kael Thornwood"));
        assert_eq!(files.visual_dna_yaml, Some("style: dark\n".to_string()));
        assert_eq!(files.prompt_md, Some("Generate an NPC named Kael.\n".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_npc_files_missing_optional() {
        let dir = std::env::temp_dir().join("nerdforge_load_npc_files_optional_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs").join("Lira Voss")).unwrap();
        std::fs::write(
            dir.join("npcs").join("Lira Voss").join("index.md"),
            "---\nstatus: draft\ntags: [npc]\n---\n\n# Lira Voss\n",
        )
        .unwrap();

        std::fs::write(
            dir.join("vaultconfig.yaml"),
            "version: 1\nfolder_map:\n  npc: npcs\nfolder_template:\n  name: Padrão RPG\n  folders:\n    - npcs\n",
        )
        .unwrap();

        let root = dir.to_string_lossy().to_string();
        let files = load_npc_files(&root, "npcs/Lira Voss").unwrap();
        assert!(files.index_md.contains("# Lira Voss"));
        assert_eq!(files.visual_dna_yaml, None);
        assert_eq!(files.prompt_md, None);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
