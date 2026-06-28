use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, AppConfig};
use crate::indexer::load_index;
use crate::vault;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub created_at: Option<u64>,
    pub updated_at: Option<u64>,
    pub file_count: usize,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub timestamp: u64,
    pub module: String,
    pub title: String,
    pub path: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub projects: Vec<ProjectInfo>,
    pub recent_activities: Vec<ActivityEvent>,
}

pub fn list_projects(current_root: Option<&str>) -> Result<Vec<ProjectInfo>, String> {
    let mut projects = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Some(root) = current_root {
        if let Ok(info) = project_info_from_path(root, true) {
            seen.insert(info.path.clone());
            projects.push(info);
        }
    }

    let parent_candidates = vec![
        dirs::document_dir().map(|d| d.join("NerdForge")),
        dirs::home_dir().map(|d| d.join("NerdForge")),
    ];

    for candidate in parent_candidates.into_iter().flatten() {
        if !candidate.exists() {
            continue;
        }
        let entries = fs::read_dir(&candidate).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let path_str = path.to_string_lossy().to_string();
            if seen.contains(&path_str) {
                continue;
            }
            if is_nerdforge_project(&path) {
                if let Ok(info) = project_info_from_path(&path_str, false) {
                    seen.insert(info.path.clone());
                    projects.push(info);
                }
            }
        }
    }

    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(projects)
}

pub fn project_info_from_path(path: &str, is_current: bool) -> Result<ProjectInfo, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let created_at = meta
        .created()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    let updated_at = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    let config_path = Path::new(path).join(".nerdforge").join("config.yaml");
    let name = if config_path.exists() {
        load_config(config_path.to_string_lossy().to_string())
            .ok()
            .map(|cfg| cfg.workspace.name)
            .unwrap_or_else(|| {
                Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Projeto".to_string())
            })
    } else {
        Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Projeto".to_string())
    };

    let file_count = count_files(path);

    Ok(ProjectInfo {
        name,
        path: path.to_string(),
        created_at,
        updated_at,
        file_count,
        is_current,
    })
}

fn is_nerdforge_project(path: &Path) -> bool {
    let nerdforge_dir = path.join(".nerdforge");
    nerdforge_dir.exists() && nerdforge_dir.is_dir()
}

fn count_files(path: &str) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                count += 1;
            } else if p.is_dir() && p.file_name() != Some(".nerdforge".as_ref()) {
                count += count_files(&p.to_string_lossy());
            }
        }
    }
    count
}

pub fn get_recent_activities(root: &str, limit: usize) -> Result<Vec<ActivityEvent>, String> {
    let index = load_index(root)?;

    let mut activities: Vec<ActivityEvent> = index
        .entries
        .into_iter()
        .filter(|e| e.kind == "file")
        .map(|entry| ActivityEvent {
            timestamp: entry.last_modified.unwrap_or(0),
            module: entry.module.clone(),
            title: entry.title.unwrap_or_else(|| entry.name.clone()),
            path: entry.absolute_path,
            event_type: "updated".to_string(),
        })
        .collect();

    activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    activities.truncate(limit);
    Ok(activities)
}

pub fn create_project(name: &str, parent_dir: &str, template: &str) -> Result<ProjectInfo, String> {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let project_dir = Path::new(parent_dir).join(sanitized.trim());

    if project_dir.exists() {
        return Err(format!(
            "Projeto j\u{00e1} existe: {}",
            project_dir.display()
        ));
    }

    fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;

    vault::initialize_vault(&project_dir.to_string_lossy())?;

    let config_path = project_dir.join(".nerdforge").join("config.yaml");
    let mut config = AppConfig::default();
    config.workspace.name = name.to_string();
    config.workspace.root_path = project_dir.to_string_lossy().to_string();

    if template == "rpg" {
        config.modules.npc.enabled = true;
        config.modules.lore.enabled = true;
    } else if template == "book" {
        config.modules.npc.enabled = false;
        config.modules.lore.enabled = true;
    } else {
        config.modules.npc.enabled = true;
        config.modules.lore.enabled = true;
    }

    save_config(config_path.to_string_lossy().to_string(), config)?;

    project_info_from_path(&project_dir.to_string_lossy(), false)
}

#[tauri::command]
pub fn list_projects_command(current_root: Option<String>) -> Result<Vec<ProjectInfo>, String> {
    list_projects(current_root.as_deref())
}

#[tauri::command]
pub fn get_recent_activities_command(
    root: String,
    limit: Option<usize>,
) -> Result<Vec<ActivityEvent>, String> {
    get_recent_activities(&root, limit.unwrap_or(20))
}

#[tauri::command]
pub fn create_project_command(
    name: String,
    parent_dir: String,
    template: String,
) -> Result<ProjectInfo, String> {
    let resolved_parent = match parent_dir.as_str() {
        "documents" => dirs::document_dir()
            .map(|p| p.join("NerdForge"))
            .ok_or("Pasta Documents n\u{00e3}o encontrada")?,
        "home" => dirs::home_dir().ok_or("Pasta home n\u{00e3}o encontrada")?,
        path => std::path::PathBuf::from(path),
    };

    std::fs::create_dir_all(&resolved_parent).map_err(|e| e.to_string())?;
    create_project(&name, &resolved_parent.to_string_lossy(), &template)
}

#[tauri::command]
pub fn get_dashboard_data_command(
    root: String,
    limit: Option<usize>,
) -> Result<DashboardData, String> {
    let projects = list_projects(Some(&root))?;
    let recent_activities = get_recent_activities(&root, limit.unwrap_or(20))?;
    Ok(DashboardData {
        projects,
        recent_activities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_nerdforge_project() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".nerdforge")).unwrap();
        assert!(is_nerdforge_project(tmp.path()));
    }

    #[test]
    fn test_count_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.md"), "a").unwrap();
        fs::create_dir(tmp.path().join("sub")).unwrap();
        fs::write(tmp.path().join("sub/b.yaml"), "b").unwrap();
        assert_eq!(count_files(&tmp.path().to_string_lossy()), 2);
    }

    #[test]
    fn test_create_project() {
        let tmp = TempDir::new().unwrap();
        let parent = tmp.path().to_string_lossy().to_string();
        let project = create_project("My World", &parent, "rpg").unwrap();
        assert_eq!(project.name, "My World");
        assert!(Path::new(&project.path).join(".nerdforge").exists());
        assert!(Path::new(&project.path).join("npcs").exists());
    }
}
