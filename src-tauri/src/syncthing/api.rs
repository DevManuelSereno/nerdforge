use serde::{Deserialize, Serialize};

const SYNCTHING_DEFAULT_API_PORT: u16 = 8384;
const SYNCTHING_DEFAULT_HOST: &str = "127.0.0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub my_id: String,
    pub uptime_s: i64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStatus {
    pub id: String,
    pub label: String,
    pub path: String,
    #[serde(rename = "type")]
    pub folder_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderConfigResult {
    pub configured: bool,
    pub folder_id: String,
    pub path: String,
    pub error: Option<String>,
}

fn syncthing_api_url(path: &str) -> String {
    format!(
        "http://{}:{}{}",
        SYNCTHING_DEFAULT_HOST, SYNCTHING_DEFAULT_API_PORT, path
    )
}

pub async fn health_check(api_key: &str) -> HealthCheckResult {
    let start = std::time::Instant::now();
    match reqwest::Client::new()
        .get(&syncthing_api_url("/rest/system/ping"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                HealthCheckResult {
                    healthy: true,
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: None,
                }
            } else {
                HealthCheckResult {
                    healthy: false,
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: Some(format!("HTTP {}", response.status())),
                }
            }
        }
        Err(e) => HealthCheckResult {
            healthy: false,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
    }
}

pub async fn get_system_status(api_key: &str) -> Result<SystemStatus, String> {
    let response = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/system/status"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to call system status: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<SystemStatus>()
        .await
        .map_err(|e| format!("Failed to parse system status: {}", e))
}

pub async fn get_folders(api_key: &str) -> Result<Vec<FolderStatus>, String> {
    let response = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/config/folders"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to call folders endpoint: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<Vec<FolderStatus>>()
        .await
        .map_err(|e| format!("Failed to parse folders: {}", e))
}

pub async fn get_folder_exists(api_key: &str, folder_id: &str) -> Result<bool, String> {
    let folders = get_folders(api_key).await?;
    Ok(folders.iter().any(|f| f.id == folder_id))
}

pub async fn configure_vault_folder(
    api_key: &str,
    vault_root: &str,
) -> Result<FolderConfigResult, String> {
    let folder_id = "nerdforge-vault";
    let label = "NerdForge Vault";

    if get_folder_exists(api_key, folder_id).await? {
        return Ok(FolderConfigResult {
            configured: true,
            folder_id: folder_id.to_string(),
            path: vault_root.to_string(),
            error: None,
        });
    }

    let mut folders = get_folders(api_key).await?;

    folders.push(FolderStatus {
        id: folder_id.to_string(),
        label: label.to_string(),
        path: vault_root.to_string(),
        folder_type: "sendreceive".to_string(),
    });

    let response = reqwest::Client::new()
        .put(&syncthing_api_url("/rest/config/folders"))
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&folders)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to save folders config: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP error {}: {}", status, body));
    }

    Ok(FolderConfigResult {
        configured: true,
        folder_id: folder_id.to_string(),
        path: vault_root.to_string(),
        error: None,
    })
}

pub async fn get_vault_folder_status(
    api_key: &str,
    folder_id: &str,
) -> Result<Option<FolderStatus>, String> {
    let folders = get_folders(api_key).await?;
    Ok(folders.into_iter().find(|f| f.id == folder_id))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: String,
    pub addresses: Vec<String>,
    pub connected: bool,
    pub introducer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDevice {
    pub device_id: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDeviceResult {
    pub added: bool,
    pub device_id: String,
    pub error: Option<String>,
}

pub async fn get_device_id(api_key: &str) -> Result<String, String> {
    let status = get_system_status(api_key).await?;
    Ok(status.my_id)
}

pub async fn get_devices(api_key: &str) -> Result<Vec<DeviceInfo>, String> {
    let config_response = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/config/devices"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to call devices config: {}", e))?;

    if !config_response.status().is_success() {
        return Err(format!(
            "HTTP error on devices config: {}",
            config_response.status()
        ));
    }

    let config_devices = config_response
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| format!("Failed to parse devices config: {}", e))?;

    let connections_response = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/system/connections"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to call connections: {}", e))?;

    let mut connected_ids = std::collections::HashSet::new();
    if connections_response.status().is_success() {
        if let Ok(connections) = connections_response.json::<serde_json::Value>().await {
            if let Some(conns) = connections.get("connections").and_then(|v| v.as_object()) {
                for (id, info) in conns {
                    if info.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
                        connected_ids.insert(id.clone());
                    }
                }
            }
        }
    }

    let devices: Vec<DeviceInfo> = config_devices
        .into_iter()
        .map(|device| {
            let device_id = device
                .get("deviceID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = device
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&device_id)
                .to_string();
            let addresses = device
                .get("addresses")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let introducer = device
                .get("introducer")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            DeviceInfo {
                device_id: device_id.clone(),
                name,
                addresses,
                connected: connected_ids.contains(&device_id),
                introducer,
            }
        })
        .collect();

    Ok(devices)
}

pub async fn get_pending_devices(api_key: &str) -> Result<Vec<PendingDevice>, String> {
    let response = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/cluster/pending/devices"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to call pending devices: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let pending = response
        .json::<std::collections::HashMap<String, serde_json::Value>>()
        .await
        .map_err(|e| format!("Failed to parse pending devices: {}", e))?;

    let devices: Vec<PendingDevice> = pending
        .into_iter()
        .map(|(device_id, info)| {
            let name = info
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let address = info
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            PendingDevice {
                device_id,
                name,
                address,
            }
        })
        .collect();

    Ok(devices)
}

pub async fn add_device(
    api_key: &str,
    device_id: &str,
    device_name: &str,
) -> Result<AddDeviceResult, String> {
    let mut devices = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/config/devices"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to load devices: {}", e))?
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| format!("Failed to parse devices: {}", e))?;

    if devices.iter().any(|d| {
        d.get("deviceID")
            .and_then(|v| v.as_str())
            .map(|id| id == device_id)
            .unwrap_or(false)
    }) {
        return Ok(AddDeviceResult {
            added: false,
            device_id: device_id.to_string(),
            error: Some("Device already exists".to_string()),
        });
    }

    let new_device = serde_json::json!({
        "deviceID": device_id,
        "name": device_name,
        "addresses": ["dynamic"],
        "introducer": false,
        "paused": false
    });

    devices.push(new_device);

    let response = reqwest::Client::new()
        .put(&syncthing_api_url("/rest/config/devices"))
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&devices)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to save devices: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP error {}: {}", status, body));
    }

    Ok(AddDeviceResult {
        added: true,
        device_id: device_id.to_string(),
        error: None,
    })
}

pub async fn accept_device(api_key: &str, device_id: &str) -> Result<AddDeviceResult, String> {
    add_device(api_key, device_id, "Dispositivo pendente").await
}

pub async fn get_folder_devices(api_key: &str, folder_id: &str) -> Result<Vec<String>, String> {
    let folders = get_folders_full(api_key).await?;
    let folder = folders
        .into_iter()
        .find(|f| {
            f.get("id")
                .and_then(|v| v.as_str())
                .map(|id| id == folder_id)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Folder {} not found", folder_id))?;

    let devices = folder
        .get("devices")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(devices)
}

pub async fn share_folder_with_device(
    api_key: &str,
    folder_id: &str,
    device_id: &str,
) -> Result<FolderConfigResult, String> {
    let mut folders = get_folders_full(api_key).await?;

    let folder_index = folders.iter().position(|f| {
        f.get("id")
            .and_then(|v| v.as_str())
            .map(|id| id == folder_id)
            .unwrap_or(false)
    });

    let folder_index = folder_index.ok_or_else(|| format!("Folder {} not found", folder_id))?;

    let folder = &mut folders[folder_index];

    let devices_array = folder
        .as_object_mut()
        .ok_or_else(|| "Folder is not an object".to_string())?
        .entry("devices")
        .or_insert_with(|| serde_json::Value::Array(vec![]))
        .as_array_mut()
        .ok_or_else(|| "devices field is not an array".to_string())?;

    if !devices_array.iter().any(|d| d.as_str() == Some(device_id)) {
        devices_array.push(serde_json::Value::String(device_id.to_string()));
    }

    let response = reqwest::Client::new()
        .put(&syncthing_api_url("/rest/config/folders"))
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&folders)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to save folders config: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP error {}: {}", status, body));
    }

    Ok(FolderConfigResult {
        configured: true,
        folder_id: folder_id.to_string(),
        path: String::new(),
        error: None,
    })
}

async fn get_folders_full(api_key: &str) -> Result<Vec<serde_json::Value>, String> {
    let response = reqwest::Client::new()
        .get(&syncthing_api_url("/rest/config/folders"))
        .header("X-API-Key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to call folders endpoint: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| format!("Failed to parse folders: {}", e))
}

pub fn read_api_key(config_dir: &str) -> Result<String, String> {
    let config_path = std::path::Path::new(config_dir).join("config.xml");
    if !config_path.exists() {
        return Err(format!("config.xml not found at: {}", config_path.display()));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config.xml: {}", e))?;

    // Simple regex-free extraction: find <apikey>...</apikey>
    if let Some(start) = content.find("<apikey>") {
        let after_start = &content[start + 8..];
        if let Some(end) = after_start.find("</apikey>") {
            return Ok(after_start[..end].to_string());
        }
    }

    Err("API key not found in config.xml".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syncthing_api_url() {
        let url = syncthing_api_url("/rest/system/ping");
        assert_eq!(url, "http://127.0.0.1:8384/rest/system/ping");
    }

    #[test]
    fn test_read_api_key() {
        let dir = std::env::temp_dir().join("nerdforge_syncthing_api_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let xml = r#"<configuration version="37">
            <gui enabled="true" tls="false">
                <apikey>test-api-key-12345</apikey>
            </gui>
        </configuration>"#;
        std::fs::write(dir.join("config.xml"), xml).unwrap();

        let key = read_api_key(&dir.to_string_lossy()).unwrap();
        assert_eq!(key, "test-api-key-12345");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_api_key_missing_file() {
        let dir = std::env::temp_dir().join("nerdforge_syncthing_api_missing");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let result = read_api_key(&dir.to_string_lossy());
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_folder_config_result_default() {
        let result = FolderConfigResult {
            configured: true,
            folder_id: "nerdforge-vault".to_string(),
            path: "D:\\nerdforge".to_string(),
            error: None,
        };
        assert!(result.configured);
        assert_eq!(result.folder_id, "nerdforge-vault");
    }

    #[test]
    fn test_folder_status_matches_id() {
        let folders = vec![
            FolderStatus {
                id: "nerdforge-vault".to_string(),
                label: "Vault".to_string(),
                path: "D:\\nerdforge".to_string(),
                folder_type: "sendreceive".to_string(),
            },
            FolderStatus {
                id: "other".to_string(),
                label: "Other".to_string(),
                path: "D:\\other".to_string(),
                folder_type: "sendreceive".to_string(),
            },
        ];
        let found = folders.into_iter().find(|f| f.id == "nerdforge-vault");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label, "Vault");
    }
}
