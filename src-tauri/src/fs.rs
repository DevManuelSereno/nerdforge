use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use tauri::{AppHandle, Emitter, Runtime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_file: bool,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileChangeEvent {
    pub kind: String,
    pub paths: Vec<String>,
}

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
pub fn list_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|e| format!("Failed to list dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let metadata = entry.metadata().map_err(|e| format!("Failed to read metadata: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().to_string_lossy().to_string();
        entries.push(DirEntry {
            name,
            path,
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
        });
    }
    Ok(entries)
}

#[tauri::command]
pub fn create_dir(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create dir: {}", e))
}

#[tauri::command]
pub fn remove_dir(path: String) -> Result<(), String> {
    std::fs::remove_dir_all(&path).map_err(|e| format!("Failed to remove dir: {}", e))
}

pub fn start_watcher<R: Runtime>(
    app_handle: AppHandle<R>,
    paths: Vec<String>,
) -> Result<Sender<()>, String> {
    let (tx, rx) = channel::<()>();
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<Event, notify::Error>| match res {
            Ok(event) => {
                let kind = format!("{:?}", event.kind);
                let paths: Vec<String> = event
                    .paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                let payload = FileChangeEvent { kind, paths };
                let _ = app_handle.emit("fs:change", payload);
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    for path in paths {
        let path_buf = PathBuf::from(path);
        watcher
            .watch(&path_buf, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch path: {}", e))?;
    }

    thread::spawn(move || {
        let _ = rx.recv();
        let _ = watcher;
    });

    Ok(tx)
}
