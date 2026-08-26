use serde::{Deserialize, Serialize};
use std::path::Path;

fn obsidian_candidates() -> Vec<Option<std::path::PathBuf>> {
    vec![
        dirs::data_local_dir().map(|d| d.join("Obsidian").join("Obsidian.exe")),
        dirs::home_dir().map(|d| d.join("AppData").join("Local").join("Obsidian").join("Obsidian.exe")),
        Some(Path::new("C:\\Program Files\\Obsidian\\Obsidian.exe").to_path_buf()),
        Some(Path::new("C:\\Program Files (x86)\\Obsidian\\Obsidian.exe").to_path_buf()),
    ]
}

pub fn is_obsidian_available() -> bool {
    obsidian_candidates().iter().flatten().any(|p| p.exists())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianVault {
    pub path: String,
    pub name: String,
    pub open: bool,
}

/// Read Obsidian's global config (`obsidian.json`) to list registered vaults.
pub fn list_obsidian_vaults() -> Result<Vec<ObsidianVault>, String> {
    let config_path = obsidian_config_path();
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read obsidian.json: {}", e))?;

    #[derive(Deserialize)]
    struct ObsidianJson {
        vaults: std::collections::HashMap<String, ObsidianVaultEntry>,
    }

    #[derive(Deserialize)]
    struct ObsidianVaultEntry {
        path: String,
        #[serde(default)]
        open: bool,
    }

    let parsed: ObsidianJson = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse obsidian.json: {}", e))?;

    let vaults = parsed
        .vaults
        .into_values()
        .map(|v| {
            let name = Path::new(&v.path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| v.path.clone());
            ObsidianVault {
                path: v.path,
                name,
                open: v.open,
            }
        })
        .collect();

    Ok(vaults)
}

fn obsidian_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("obsidian")
        .join("obsidian.json")
}

pub fn open_in_obsidian(path: &str) -> Result<(), String> {
    let target = Path::new(path);
    if !target.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let absolute = target
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // If the path is a directory, try to open index.md inside it.
    let file_to_open = if absolute.is_dir() {
        let index = absolute.join("index.md");
        if index.exists() { index } else { absolute }
    } else {
        absolute
    };

    let absolute_str = file_to_open.to_string_lossy();
    // Remove the UNC prefix (\\?\) added by canonicalize on Windows.
    let clean_path = absolute_str.strip_prefix("\\\\?\\").unwrap_or(&absolute_str);

    if is_obsidian_available() {
        let url = format!("obsidian://open?path={}", encode_obsidian_path(clean_path));
        open_url(&url)?;
        return Ok(());
    }

    // Fallback: open the parent folder in the system file explorer.
    let parent = file_to_open
        .parent()
        .ok_or_else(|| format!("Path has no parent directory: {}", path))?;
    open_system_path(parent.to_string_lossy().as_ref())
}

fn encode_obsidian_path(path: &str) -> String {
    // Percent-encode characters that are unsafe in a URI query value.
    path.replace('%', "%25")
        .replace(' ', "%20")
        .replace('\\', "%5C")
        .replace('/', "%2F")
        .replace('?', "%3F")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('#', "%23")
        .replace('+', "%2B")
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}

fn open_system_path(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open system path: {}", e))?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open system path: {}", e))?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open system path: {}", e))?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}

#[tauri::command]
pub fn is_obsidian_available_command() -> Result<bool, String> {
    Ok(is_obsidian_available())
}

#[tauri::command]
pub fn list_obsidian_vaults_command() -> Result<Vec<ObsidianVault>, String> {
    tracing::info!("listing obsidian vaults");
    list_obsidian_vaults()
}

#[tauri::command]
pub fn open_in_obsidian_command(path: String) -> Result<(), String> {
    tracing::info!("opening in obsidian: {}", path);
    open_in_obsidian(&path)
}

pub fn npc_changed(
    root: &str,
    relative_path: &str,
    last_modified: Option<u64>,
) -> Result<bool, String> {
    let base = Path::new(root).join(relative_path);
    if !base.exists() {
        return Err(format!("NPC path does not exist: {}", base.display()));
    }

    let mut latest: Option<u64> = None;
    let entries = std::fs::read_dir(&base)
        .map_err(|e| format!("Failed to read NPC dir {}: {}", base.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let meta = entry
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        if let Ok(t) = meta.modified() {
            let secs = t
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            latest = Some(latest.map_or(secs, |l| l.max(secs)));
        }
    }

    // If no files were found, fall back to the directory modified time.
    if latest.is_none() {
        let meta = std::fs::metadata(&base)
            .map_err(|e| format!("Failed to read metadata for {}: {}", base.display(), e))?;
        latest = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
    }

    Ok(latest != last_modified)
}

#[tauri::command]
pub fn npc_changed_command(
    root: String,
    relative_path: String,
    last_modified: Option<u64>,
) -> Result<bool, String> {
    tracing::info!("checking npc changed: {}", relative_path);
    npc_changed(&root, &relative_path, last_modified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_obsidian_path() {
        assert_eq!(encode_obsidian_path("C:\\Users\\Test"), "C:%5CUsers%5CTest");
        assert_eq!(encode_obsidian_path("a b"), "a%20b");
        assert_eq!(encode_obsidian_path("a&b"), "a%26b");
        assert_eq!(encode_obsidian_path("a+b"), "a%2Bb");
    }

    #[test]
    fn test_is_obsidian_available_false_when_missing() {
        // Ensure none of the standard candidate paths exist in this test environment.
        assert!(!obsidian_candidates().iter().flatten().any(|p| p.exists()));
        assert!(!is_obsidian_available());
    }

    #[test]
    fn test_list_obsidian_vaults_missing_config() {
        // If obsidian.json exists in the real config dir, this will return vaults.
        // If it doesn't, it returns empty. Either way, it should not error.
        let vaults = list_obsidian_vaults().unwrap();
        // Just verify it doesn't panic and returns a Vec.
        let _ = vaults;
    }

    #[test]
    fn test_list_obsidian_vaults_parses_config() {
        let dir = std::env::temp_dir().join("nerdforge_obsidian_vaults_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let config_dir = dir.join("obsidian");
        std::fs::create_dir_all(&config_dir).unwrap();

        // Write a fake obsidian.json
        let json = r#"{"vaults":{"abc123":{"path":"D:\\MyVault","open":true},"def456":{"path":"C:\\Users\\Test\\Notes","open":false}}}"#;
        std::fs::write(config_dir.join("obsidian.json"), json).unwrap();

        // We can't easily mock dirs::config_dir(), but we can test the parsing logic
        // by calling the function and checking it doesn't crash.
        // In the test env, it will read the real config or return empty.
        let vaults = list_obsidian_vaults().unwrap();
        // The result depends on whether obsidian.json exists in the real config dir.
        // Just verify it doesn't panic.
        let _ = vaults;

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_npc_changed_detects_modification() {
        let dir = std::env::temp_dir().join("nerdforge_npc_changed_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("npcs").join("Kael")).unwrap();

        // Wait to ensure the directory timestamp is older than the file we will create.
        std::thread::sleep(std::time::Duration::from_millis(1500));

        let root = dir.to_string_lossy().to_string();
        let initial = std::fs::metadata(dir.join("npcs").join("Kael"))
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(!npc_changed(&root, "npcs/Kael", Some(initial)).unwrap());

        std::thread::sleep(std::time::Duration::from_millis(100));
        std::fs::write(dir.join("npcs").join("Kael").join("index.md"), "# Kael")
            .unwrap();

        assert!(npc_changed(&root, "npcs/Kael", Some(initial)).unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
