use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncthingStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub executable: Option<String>,
    pub config_dir: Option<String>,
    pub data_dir: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncthingConfig {
    pub config_dir: String,
    pub data_dir: String,
}

pub struct SyncthingState {
    pub child: Mutex<Option<Child>>,
    pub config_dir: Mutex<Option<String>>,
}

impl SyncthingState {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            config_dir: Mutex::new(None),
        }
    }
}

impl Default for SyncthingState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn bundled_syncthing_executable() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();

    let candidates = [
        exe_dir.join("resources").join("syncthing.exe"),
        exe_dir.join("syncthing.exe"),
    ];

    for path in candidates.iter() {
        if let Ok(canonical) = path.canonicalize() {
            if canonical.exists() {
                return Some(canonical);
            }
        }
    }

    None
}

pub fn find_syncthing_executable() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        let candidates = [
            "syncthing.exe",
            "C:\\Program Files\\Syncthing\\syncthing.exe",
            "C:\\Program Files (x86)\\Syncthing\\syncthing.exe",
        ];
        for candidate in candidates.iter() {
            let path = Path::new(candidate);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }
    }

    let command = if cfg!(target_os = "windows") { "where" } else { "which" };
    Command::new(command)
        .arg("syncthing")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .map(|s| s.trim().to_string())?;
                let path = PathBuf::from(path);
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
}

pub fn start_syncthing(
    state: tauri::State<SyncthingState>,
    config: SyncthingConfig,
) -> Result<SyncthingStatus, String> {
    let executable = bundled_syncthing_executable()
        .or_else(find_syncthing_executable)
        .ok_or_else(|| "Syncthing executable not found".to_string())?;

    let config_dir = Path::new(&config.config_dir);
    let data_dir = Path::new(&config.data_dir);

    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    std::fs::create_dir_all(data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let config_dir_str = config_dir.to_string_lossy().to_string();
    let data_dir_str = data_dir.to_string_lossy().to_string();

    let mut cmd = Command::new(&executable);
    cmd.arg("-no-browser")
        .arg("-no-restart")
        .arg("-config")
        .arg(&config_dir_str)
        .arg("-data")
        .arg(&data_dir_str)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = cmd.spawn().map_err(|e| format!("Failed to start Syncthing: {}", e))?;
    let pid = child.id();

    {
        let mut guard = state.child.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(mut old) = guard.take() {
            let _ = old.kill();
        }
        *guard = Some(child);
    }

    {
        let mut guard = state
            .config_dir
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *guard = Some(config_dir_str.clone());
    }

    Ok(SyncthingStatus {
        running: true,
        pid: Some(pid),
        executable: Some(executable.to_string_lossy().to_string()),
        config_dir: Some(config_dir_str),
        data_dir: Some(data_dir_str),
        error: None,
    })
}

pub fn stop_syncthing(state: tauri::State<SyncthingState>) -> Result<SyncthingStatus, String> {
    let mut guard = state.child.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(mut child) = guard.take() {
        let pid = child.id();
        child.kill().map_err(|e| format!("Failed to kill Syncthing: {}", e))?;
        return Ok(SyncthingStatus {
            running: false,
            pid: Some(pid),
            executable: None,
            config_dir: None,
            data_dir: None,
            error: None,
        });
    }

    Ok(SyncthingStatus {
        running: false,
        pid: None,
        executable: None,
        config_dir: None,
        data_dir: None,
        error: None,
    })
}

pub fn syncthing_status(state: tauri::State<SyncthingState>) -> Result<SyncthingStatus, String> {
    let mut guard = state.child.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(ref mut child) = *guard {
        match child.try_wait() {
            Ok(None) => {
                let config_dir = state
                    .config_dir
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().cloned());
                return Ok(SyncthingStatus {
                    running: true,
                    pid: Some(child.id()),
                    executable: None,
                    config_dir,
                    data_dir: None,
                    error: None,
                });
            }
            Ok(Some(_exit_status)) => {
                *guard = None;
                return Ok(SyncthingStatus {
                    running: false,
                    pid: None,
                    executable: None,
                    config_dir: None,
                    data_dir: None,
                    error: Some("Process exited".to_string()),
                });
            }
            Err(e) => {
                return Err(format!("Failed to check process status: {}", e));
            }
        }
    }

    Ok(SyncthingStatus {
        running: false,
        pid: None,
        executable: None,
        config_dir: None,
        data_dir: None,
        error: None,
    })
}

#[tauri::command]
pub fn start_syncthing_command(
    state: tauri::State<SyncthingState>,
    root: String,
) -> Result<SyncthingStatus, String> {
    let config_dir = Path::new(&root).join(".nerdforge").join("syncthing-config");
    let data_dir = Path::new(&root).join(".nerdforge").join("syncthing-data");

    let config = SyncthingConfig {
        config_dir: config_dir.to_string_lossy().to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
    };

    tracing::info!("starting syncthing for vault: {}", root);
    start_syncthing(state, config)
}

#[tauri::command]
pub fn stop_syncthing_command(state: tauri::State<SyncthingState>) -> Result<SyncthingStatus, String> {
    tracing::info!("stopping syncthing");
    stop_syncthing(state)
}

#[tauri::command]
pub fn syncthing_status_command(state: tauri::State<SyncthingState>) -> Result<SyncthingStatus, String> {
    syncthing_status(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syncthing_state_new() {
        let state = SyncthingState::new();
        assert!(state.child.lock().unwrap().is_none());
    }

    #[test]
    fn test_syncthing_config_dir_from_root() {
        let root = "C:\\Temp\\test-vault";
        let config_dir = Path::new(root).join(".nerdforge").join("syncthing-config");
        let data_dir = Path::new(root).join(".nerdforge").join("syncthing-data");
        assert!(config_dir.to_string_lossy().contains("syncthing-config"));
        assert!(data_dir.to_string_lossy().contains("syncthing-data"));
    }

    #[test]
    fn test_status_default_is_stopped() {
        let state = SyncthingState::new();
        let child_guard = state.child.lock().unwrap();
        assert!(child_guard.is_none());
    }
}
