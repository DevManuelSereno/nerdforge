use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::vaultconfig::ensure_vault_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub last_modified: Option<u64>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFiles {
    pub index_md: String,
    pub visual_dna_yaml: Option<String>,
    pub prompt_md: Option<String>,
}

/// List all entities of a given module (e.g. "npc", "lore", "faction", "location").
/// Supports an optional category sub-folder level (e.g. lore/<category>/<title>/index.md).
pub fn list_entities(root: &str, module: &str) -> Result<Vec<EntitySummary>, String> {
    let config = ensure_vault_config(root)?;
    let folder_name = config
        .folder_map
        .get(module)
        .cloned()
        .unwrap_or_else(|| module.to_string());

    let module_path = Path::new(root).join(&folder_name);

    if !module_path.exists() {
        std::fs::create_dir_all(&module_path)
            .map_err(|e| format!("Failed to create {} folder: {}", module, e))?;
    }

    if !module_path.is_dir() {
        return Err(format!(
            "{} path is not a directory: {}",
            module,
            module_path.display()
        ));
    }

    let mut entities = Vec::new();
    collect_entities(root, &module_path, &module_path, module, &mut entities)?;
    entities.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(entities)
}

/// Recursively walk a module directory looking for folders containing index.md.
fn collect_entities(
    root: &str,
    current: &Path,
    module_root: &Path,
    module: &str,
    out: &mut Vec<EntitySummary>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(current)
        .map_err(|e| format!("Failed to read directory {}: {}", current.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let index_md = path.join("index.md");

        if index_md.exists() {
            // This is an entity folder.
            let (status, tags) =
                crate::parser::parse_markdown_file(index_md.to_string_lossy().as_ref())
                    .map(|p| (p.status, p.tags))
                    .unwrap_or_else(|e| {
                        tracing::warn!(
                            "failed to parse {} index {}: {}",
                            module,
                            index_md.display(),
                            e
                        );
                        (None, Vec::new())
                    });

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

            // Determine category: the path component between module_root and the entity folder.
            let category = path.parent().and_then(|p| {
                if p == module_root {
                    None
                } else {
                    p.strip_prefix(module_root)
                        .ok()
                        .and_then(|rel| rel.to_str())
                        .map(|s| s.to_string())
                }
            });

            let last_modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            out.push(EntitySummary {
                id,
                name,
                slug,
                relative_path,
                absolute_path: path.to_string_lossy().to_string(),
                status,
                tags,
                last_modified,
                category,
            });
        } else {
            // This might be a category sub-folder. Recurse into it.
            collect_entities(root, &path, module_root, module, out)?;
        }
    }

    Ok(())
}

/// Load the files of an entity at the given relative path.
pub fn load_entity_files(root: &str, relative_path: &str) -> Result<EntityFiles, String> {
    let base = Path::new(root).join(relative_path);
    let index_md = std::fs::read_to_string(base.join("index.md"))
        .map_err(|e| format!("Failed to read index.md: {}", e))?;

    let visual_dna_yaml = std::fs::read_to_string(base.join("visual-dna.yaml")).ok();
    let prompt_md = std::fs::read_to_string(base.join("prompt.md")).ok();

    Ok(EntityFiles {
        index_md,
        visual_dna_yaml,
        prompt_md,
    })
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
        .replace("--", "-")
}

#[tauri::command]
pub fn list_entities_command(root: String, module: String) -> Result<Vec<EntitySummary>, String> {
    tracing::info!("listing entities for module: {} in {}", module, root);
    list_entities(&root, &module)
}

#[tauri::command]
pub fn load_entity_files_command(
    root: String,
    relative_path: String,
) -> Result<EntityFiles, String> {
    tracing::info!("loading entity files for: {}", relative_path);
    load_entity_files(&root, &relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_entities_lore_empty() {
        let dir = std::env::temp_dir().join("nerdforge_entities_lore_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let root = dir.to_string_lossy().to_string();
        let entities = list_entities(&root, "lore").unwrap();
        assert!(entities.is_empty());
        assert!(dir.join("lore").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_entities_lore_with_categories() {
        let dir = std::env::temp_dir().join("nerdforge_entities_lore_cats");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("lore").join("História").join("A Queda de Valdris"))
            .unwrap();
        std::fs::write(
            dir.join("lore")
                .join("História")
                .join("A Queda de Valdris")
                .join("index.md"),
            "---\nstatus: draft\ntags: [lore, history]\n---\n\n# A Queda de Valdris\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("lore").join("Geografia").join("Montanhas Geladas"))
            .unwrap();
        std::fs::write(
            dir.join("lore")
                .join("Geografia")
                .join("Montanhas Geladas")
                .join("index.md"),
            "---\nstatus: complete\ntags: [lore, geography]\n---\n\n# Montanhas Geladas\n",
        )
        .unwrap();

        // Also test an entity without a category (directly in lore/)
        std::fs::create_dir_all(dir.join("lore").join("Personagem Anônimo")).unwrap();
        std::fs::write(
            dir.join("lore").join("Personagem Anônimo").join("index.md"),
            "---\nstatus: draft\ntags: [lore]\n---\n\n# Personagem Anônimo\n",
        )
        .unwrap();

        let root = dir.to_string_lossy().to_string();
        let entities = list_entities(&root, "lore").unwrap();
        assert_eq!(entities.len(), 3);

        let names: Vec<String> = entities.iter().map(|e| e.name.clone()).collect();
        assert!(names.contains(&"A Queda de Valdris".to_string()));
        assert!(names.contains(&"Montanhas Geladas".to_string()));
        assert!(names.contains(&"Personagem Anônimo".to_string()));

        let queda = entities
            .iter()
            .find(|e| e.name == "A Queda de Valdris")
            .unwrap();
        assert_eq!(queda.category, Some("História".to_string()));
        assert_eq!(queda.status, Some("draft".to_string()));
        assert_eq!(queda.tags, vec!["lore", "history"]);
        assert!(queda.relative_path.starts_with("lore/"));

        let montanhas = entities
            .iter()
            .find(|e| e.name == "Montanhas Geladas")
            .unwrap();
        assert_eq!(montanhas.category, Some("Geografia".to_string()));

        let anonimo = entities
            .iter()
            .find(|e| e.name == "Personagem Anônimo")
            .unwrap();
        assert_eq!(anonimo.category, None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_entity_files() {
        let dir = std::env::temp_dir().join("nerdforge_load_entity_files_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("lore").join("História").join("A Queda")).unwrap();
        std::fs::write(
            dir.join("lore")
                .join("História")
                .join("A Queda")
                .join("index.md"),
            "---\nstatus: draft\ntags: [lore]\n---\n\n# A Queda\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("lore")
                .join("História")
                .join("A Queda")
                .join("visual-dna.yaml"),
            "style: dark\n",
        )
        .unwrap();

        let root = dir.to_string_lossy().to_string();
        let files = load_entity_files(&root, "lore/História/A Queda").unwrap();
        assert!(files.index_md.contains("# A Queda"));
        assert_eq!(files.visual_dna_yaml, Some("style: dark\n".to_string()));
        assert_eq!(files.prompt_md, None);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
