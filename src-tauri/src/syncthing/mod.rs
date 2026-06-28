pub mod api;
pub mod process;

pub use process::*;

use tauri::State;

fn get_config_dir(state: State<'_, process::SyncthingState>) -> Result<String, String> {
    let config_dir = state
        .config_dir
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    config_dir
        .as_ref()
        .cloned()
        .ok_or_else(|| "Syncthing config directory not set".to_string())
}

#[tauri::command]
pub async fn syncthing_health_check_command(
    state: State<'_, process::SyncthingState>,
) -> Result<api::HealthCheckResult, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    Ok(api::health_check(&api_key).await)
}

#[tauri::command]
pub async fn syncthing_system_status_command(
    state: State<'_, process::SyncthingState>,
) -> Result<api::SystemStatus, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_system_status(&api_key).await
}

#[tauri::command]
pub async fn syncthing_folders_command(
    state: State<'_, process::SyncthingState>,
) -> Result<Vec<api::FolderStatus>, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_folders(&api_key).await
}

#[tauri::command]
pub async fn configure_vault_folder_command(
    state: State<'_, process::SyncthingState>,
    root: String,
) -> Result<api::FolderConfigResult, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::configure_vault_folder(&api_key, &root).await
}

#[tauri::command]
pub async fn get_vault_folder_status_command(
    state: State<'_, process::SyncthingState>,
) -> Result<Option<api::FolderStatus>, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_vault_folder_status(&api_key, "nerdforge-vault").await
}

#[tauri::command]
pub async fn get_device_id_command(
    state: State<'_, process::SyncthingState>,
) -> Result<String, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_device_id(&api_key).await
}

#[tauri::command]
pub async fn list_devices_command(
    state: State<'_, process::SyncthingState>,
) -> Result<Vec<api::DeviceInfo>, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_devices(&api_key).await
}

#[tauri::command]
pub async fn list_pending_devices_command(
    state: State<'_, process::SyncthingState>,
) -> Result<Vec<api::PendingDevice>, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_pending_devices(&api_key).await
}

#[tauri::command]
pub async fn add_device_command(
    state: State<'_, process::SyncthingState>,
    device_id: String,
    device_name: String,
) -> Result<api::AddDeviceResult, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::add_device(&api_key, &device_id, &device_name).await
}

#[tauri::command]
pub async fn accept_device_command(
    state: State<'_, process::SyncthingState>,
    device_id: String,
) -> Result<api::AddDeviceResult, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::accept_device(&api_key, &device_id).await
}

#[tauri::command]
pub async fn list_vault_devices_command(
    state: State<'_, process::SyncthingState>,
) -> Result<Vec<String>, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::get_folder_devices(&api_key, "nerdforge-vault").await
}

#[tauri::command]
pub async fn share_vault_with_device_command(
    state: State<'_, process::SyncthingState>,
    device_id: String,
) -> Result<api::FolderConfigResult, String> {
    let config_dir = get_config_dir(state)?;
    let api_key = api::read_api_key(&config_dir)?;
    api::share_folder_with_device(&api_key, "nerdforge-vault", &device_id).await
}
