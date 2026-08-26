mod config;
mod conflicts;
mod entities;
mod fs;
mod indexer;
mod hermes;
mod logging;
mod npcs;
mod obsidian;
mod parser;
mod projects;
mod regenerate;
mod structured_output;
mod syncthing;
mod templates;
mod vault;
mod vaultconfig;

use config::{ensure_config, is_first_run, load_config, save_config, AppConfig};
use conflicts::find_conflicts_command;
use entities::{list_entities_command, load_entity_files_command};
use fs::{create_dir, list_dir, read_file, remove_dir, write_file, start_watcher};
use indexer::{get_index_command, rebuild_index_command};
use logging::{default_log_dir, get_log_path, init_logging};
use parser::{
    parse_markdown_file_command, parse_yaml_file_command, validate_visual_dna_command,
};
use hermes::{
    chat_with_hermes_command, list_conversations_command, load_conversation_command,
    summarize_conversation_command,
};
use npcs::{list_npcs_command, load_npc_files_command};
use obsidian::{is_obsidian_available_command, list_obsidian_vaults_command, npc_changed_command, open_in_obsidian_command};
use regenerate::{upload_npc_portrait_command, regenerate_npc_part_command};
use structured_output::generate_structured_output_command;
use templates::{
    apply_template_command, create_blank_entity_command, list_templates_command,
    save_custom_template_command,
};
use projects::{
    create_project_command, get_dashboard_data_command, get_recent_activities_command,
    list_projects_command,
};
use syncthing::{
    accept_device_command, add_device_command, configure_vault_folder_command,
    get_device_id_command, get_vault_folder_status_command, list_devices_command,
    list_pending_devices_command, list_vault_devices_command, share_vault_with_device_command,
    start_syncthing_command, stop_syncthing_command, syncthing_folders_command,
    syncthing_health_check_command, syncthing_status_command, syncthing_system_status_command,
    SyncthingState,
};
use vault::{
    get_vault_structure_command, get_vault_template_command, initialize_vault_command,
    initialize_vault_with_folders_command, scan_vault_command,
};
use vaultconfig::{
    ensure_vault_config_command, load_vault_config_command, save_vault_config_command,
};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder},
    Manager, Runtime, State,
};

#[tauri::command]
fn greet(name: &str) -> String {
    tracing::info!("greet command called for: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct WatcherState(Mutex<Option<std::sync::mpsc::Sender<()>>>);

#[tauri::command]
fn start_file_watcher(
    app: tauri::AppHandle,
    state: State<WatcherState>,
    paths: Vec<String>,
) -> Result<(), String> {
    tracing::info!("starting file watcher for {} paths", paths.len());
    let sender = start_watcher(app, paths)?;
    let mut guard = state.0.lock().map_err(|e| {
        tracing::error!("failed to lock watcher state: {}", e);
        e.to_string()
    })?;
    *guard = Some(sender);
    Ok(())
}

#[tauri::command]
fn stop_file_watcher(state: State<WatcherState>) -> Result<(), String> {
    tracing::info!("stopping file watcher");
    let mut guard = state.0.lock().map_err(|e| {
        tracing::error!("failed to lock watcher state: {}", e);
        e.to_string()
    })?;
    if let Some(sender) = guard.take() {
        let _ = sender.send(());
    }
    Ok(())
}

#[tauri::command]
fn load_config_command(path: String) -> Result<AppConfig, String> {
    tracing::info!("loading config from: {}", path);
    load_config(path)
}

#[tauri::command]
fn save_config_command(path: String, config: AppConfig) -> Result<(), String> {
    tracing::info!("saving config to: {}", path);
    save_config(path, config)
}

#[tauri::command]
fn ensure_config_command(path: String) -> Result<AppConfig, String> {
    tracing::info!("ensuring config at: {}", path);
    ensure_config(path)
}

pub fn run() {
    if let Err(e) = init_logging(default_log_dir()) {
        eprintln!("Failed to initialize logging: {}", e);
    }

    tracing::info!("NerdForge starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tracing::info!("single instance event received");
            let _ = show_window(app);
        }))
        .manage(WatcherState(Mutex::new(None)))
        .manage(SyncthingState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            read_file,
            write_file,
            list_dir,
            create_dir,
            remove_dir,
            start_file_watcher,
            stop_file_watcher,
            load_config_command,
            save_config_command,
            ensure_config_command,
            is_first_run,
            get_log_path,
            initialize_vault_command,
            initialize_vault_with_folders_command,
            get_vault_structure_command,
            get_vault_template_command,
            scan_vault_command,
            ensure_vault_config_command,
            load_vault_config_command,
            save_vault_config_command,
            rebuild_index_command,
            get_index_command,
            parse_markdown_file_command,
            parse_yaml_file_command,
            validate_visual_dna_command,
            list_projects_command,
            get_recent_activities_command,
            create_project_command,
            get_dashboard_data_command,
            list_conversations_command,
            load_conversation_command,
            chat_with_hermes_command,
            summarize_conversation_command,
            generate_structured_output_command,
            list_templates_command,
            apply_template_command,
            create_blank_entity_command,
            save_custom_template_command,
            list_npcs_command,
            load_npc_files_command,
            list_entities_command,
            load_entity_files_command,
            find_conflicts_command,
            regenerate_npc_part_command,
            upload_npc_portrait_command,
            is_obsidian_available_command,
            list_obsidian_vaults_command,
            open_in_obsidian_command,
            npc_changed_command,
            start_syncthing_command,
            stop_syncthing_command,
            syncthing_status_command,
            syncthing_health_check_command,
            syncthing_system_status_command,
            syncthing_folders_command,
            configure_vault_folder_command,
            get_vault_folder_status_command,
            get_device_id_command,
            list_devices_command,
            list_pending_devices_command,
            add_device_command,
            accept_device_command,
            list_vault_devices_command,
            share_vault_with_device_command
        ])
        .setup(|app| {
            tracing::info!("app setup started");

            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Abrir", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("NerdForge")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        tracing::info!("tray quit selected");
                        app.exit(0);
                    }
                    "show" => {
                        tracing::info!("tray show selected");
                        let _ = show_window(app);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        let _ = show_window(app);
                    }
                })
                .build(app)?;

            tracing::info!("app setup completed");
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                tracing::info!("window close requested, hiding to tray");
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_and_read_file() {
        let dir = std::env::temp_dir().join("nerdforge_fs_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt").to_string_lossy().to_string();

        write_file(file_path.clone(), "Hello NerdForge".to_string()).unwrap();
        let content = read_file(file_path).unwrap();

        assert_eq!(content, "Hello NerdForge");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_create_dir_and_list_dir() {
        let dir = std::env::temp_dir().join("nerdforge_dir_test");
        let _ = fs::remove_dir_all(&dir);
        let dir_path = dir.to_string_lossy().to_string();

        create_dir(dir_path.clone()).unwrap();
        fs::write(dir.join("file.txt"), "x").unwrap();

        let entries = list_dir(dir_path).unwrap();
        let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();

        assert!(names.contains(&"file.txt".to_string()));
        assert!(entries.iter().any(|e| e.name == "file.txt" && e.is_file));
        let _ = fs::remove_dir_all(&dir);
    }
}
