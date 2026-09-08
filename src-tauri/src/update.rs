use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::syncthing::process::{
    bundled_syncthing_executable, find_syncthing_executable,
};

const GITHUB_API_URL: &str = "https://api.github.com/repos/syncthing/syncthing/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Get the currently installed Syncthing version by running `syncthing --version`.
fn get_current_version() -> String {
    let exe = bundled_syncthing_executable()
        .or_else(find_syncthing_executable);

    let program = if cfg!(target_os = "windows") {
        "syncthing.exe"
    } else {
        "syncthing"
    };

    let mut command = match &exe {
        Some(path) => std::process::Command::new(path),
        None => std::process::Command::new(program),
    };

    command.arg("--version");

    match command.output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            let line = version.lines().next().unwrap_or("");
            // Extract version from "syncthing v1.27.10" or similar
            if let Some(pos) = line.find('v') {
                let rest = &line[pos..];
                rest.split_whitespace().next().unwrap_or("unknown").to_string()
            } else {
                line.trim().to_string()
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Check for Syncthing updates via GitHub API.
pub async fn check_syncthing_update() -> Result<UpdateCheck, String> {
    let current_version = get_current_version();

    let client = reqwest::Client::builder()
        .user_agent("NerdForge/0.1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(GITHUB_API_URL)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch GitHub API: {}", e))?;

    if !response.status().is_success() {
        return Ok(UpdateCheck {
            current_version,
            latest_version: "unknown".to_string(),
            update_available: false,
            download_url: None,
            release_notes_url: None,
            error: Some(format!("GitHub API returned: {}", response.status())),
        });
    }

    let release: GithubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let current_clean = current_version.trim_start_matches('v').to_string();

    // Find the right asset for the platform
    let arch_suffix = if cfg!(target_os = "windows") {
        "windows-amd64"
    } else if cfg!(target_os = "macos") {
        "macos-universal"
    } else {
        "linux-amd64"
    };

    let download_url = release
        .assets
        .iter()
        .find(|a| a.name.contains(arch_suffix))
        .map(|a| a.browser_download_url.clone());

    let update_available = current_clean != latest_version && current_clean != "unknown";

    Ok(UpdateCheck {
        current_version,
        latest_version: format!("v{}", latest_version),
        update_available,
        download_url,
        release_notes_url: Some(release.html_url),
        error: None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub success: bool,
    pub message: String,
    pub new_version: Option<String>,
}

/// Download and apply a Syncthing update.
/// This downloads the ZIP, extracts the syncthing executable, backs up the old one,
/// and replaces it.
pub async fn download_and_apply_update(download_url: String) -> Result<UpdateResult, String> {
    let client = reqwest::Client::builder()
        .user_agent("NerdForge/0.1.0")
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    tracing::info!("downloading syncthing update from: {}", download_url);

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let zip_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;

    // Save to temp file
    let temp_dir = std::env::temp_dir().join("nerdforge_update");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let zip_path = temp_dir.join("syncthing.zip");
    std::fs::write(&zip_path, &zip_bytes)
        .map_err(|e| format!("Failed to write zip: {}", e))?;

    // Extract the executable from the zip
    let new_exe = extract_syncthing_exe(&zip_path, &temp_dir)?;

    // Find the current executable
    let current_exe = bundled_syncthing_executable()
        .or_else(find_syncthing_executable)
        .ok_or_else(|| "Current syncthing executable not found".to_string())?;

    // Backup the old executable
    let backup_path = current_exe.with_extension("exe.bak");
    std::fs::copy(&current_exe, &backup_path)
        .map_err(|e| format!("Failed to backup old executable: {}", e))?;

    // Replace with the new one
    std::fs::copy(&new_exe, &current_exe)
        .map_err(|e| {
            // Try to restore backup on failure
            let _ = std::fs::copy(&backup_path, &current_exe);
            format!("Failed to replace executable: {}", e)
        })?;

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Get the new version
    let new_version = get_current_version();

    Ok(UpdateResult {
        success: true,
        message: format!("Syncthing updated to {}", new_version),
        new_version: Some(new_version),
    })
}

/// Extract the syncthing executable from a downloaded ZIP archive.
/// On Windows, we use `tar` (available in Windows 10+) to extract.
fn extract_syncthing_exe(zip_path: &Path, extract_to: &Path) -> Result<PathBuf, String> {
    // Use system tar to extract (Windows 10+ has tar built-in)
    let output = std::process::Command::new("tar")
        .arg("-xf")
        .arg(zip_path)
        .arg("-C")
        .arg(extract_to)
        .output()
        .map_err(|e| format!("Failed to run tar: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tar extraction failed: {}", stderr));
    }

    // Find the syncthing executable in the extracted files
    let exe_name = if cfg!(target_os = "windows") {
        "syncthing.exe"
    } else {
        "syncthing"
    };

    // Walk the extracted directory to find the executable
    fn find_exe(dir: &Path, exe_name: &str) -> Option<PathBuf> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = find_exe(&path, exe_name) {
                        return Some(found);
                    }
                } else if path.file_name().and_then(|s| s.to_str()) == Some(exe_name) {
                    return Some(path);
                }
            }
        }
        None
    }

    find_exe(extract_to, exe_name)
        .ok_or_else(|| "syncthing executable not found in archive".to_string())
}

#[tauri::command]
pub async fn check_syncthing_update_command() -> Result<UpdateCheck, String> {
    tracing::info!("checking for syncthing updates");
    check_syncthing_update().await
}

#[tauri::command]
pub async fn download_syncthing_update_command(
    download_url: String,
) -> Result<UpdateResult, String> {
    tracing::info!("downloading and applying syncthing update");
    download_and_apply_update(download_url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_version_returns_string() {
        let version = get_current_version();
        // Should return something (either a version or "unknown")
        assert!(!version.is_empty());
    }
}
