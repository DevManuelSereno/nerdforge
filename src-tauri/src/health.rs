use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::obsidian::{is_obsidian_available, list_obsidian_vaults};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub syncthing: IntegrationStatus,
    pub hermes: IntegrationStatus,
    pub obsidian: IntegrationStatus,
    pub overall_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub available: bool,
    pub label: String,
    pub detail: String,
}

pub fn health_check() -> Result<HealthStatus, String> {
    let syncthing = check_syncthing();
    let hermes = check_hermes();
    let obsidian = check_obsidian()?;

    let overall_healthy = syncthing.available || hermes.available || obsidian.available;

    Ok(HealthStatus {
        syncthing,
        hermes,
        obsidian,
        overall_healthy,
    })
}

fn check_syncthing() -> IntegrationStatus {
    let program = if cfg!(target_os = "windows") {
        "syncthing.exe"
    } else {
        "syncthing"
    };

    match Command::new(program).arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_line = version.lines().next().unwrap_or("unknown");
                IntegrationStatus {
                    available: true,
                    label: "Syncthing".to_string(),
                    detail: version_line.trim().to_string(),
                }
            } else {
                IntegrationStatus {
                    available: false,
                    label: "Syncthing".to_string(),
                    detail: "Encontrado mas não responde".to_string(),
                }
            }
        }
        Err(_) => IntegrationStatus {
            available: false,
            label: "Syncthing".to_string(),
            detail: "Não encontrado no PATH".to_string(),
        },
    }
}

fn check_hermes() -> IntegrationStatus {
    let program = if cfg!(target_os = "windows") {
        "hermes.exe"
    } else {
        "hermes"
    };

    match Command::new(program).arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_line = version.lines().next().unwrap_or("unknown");
                IntegrationStatus {
                    available: true,
                    label: "Hermes CLI".to_string(),
                    detail: version_line.trim().to_string(),
                }
            } else {
                IntegrationStatus {
                    available: false,
                    label: "Hermes CLI".to_string(),
                    detail: "Encontrado mas não responde".to_string(),
                }
            }
        }
        Err(_) => IntegrationStatus {
            available: false,
            label: "Hermes CLI".to_string(),
            detail: "Não encontrado no PATH".to_string(),
        },
    }
}

fn check_obsidian() -> Result<IntegrationStatus, String> {
    if is_obsidian_available() {
        let vaults = list_obsidian_vaults().unwrap_or_default();
        let detail = if vaults.is_empty() {
            "Detectado, nenhum vault registrado".to_string()
        } else {
            format!("Detectado, {} vault(s)", vaults.len())
        };
        Ok(IntegrationStatus {
            available: true,
            label: "Obsidian".to_string(),
            detail,
        })
    } else {
        Ok(IntegrationStatus {
            available: false,
            label: "Obsidian".to_string(),
            detail: "Não detectado".to_string(),
        })
    }
}

#[tauri::command]
pub fn health_check_command() -> Result<HealthStatus, String> {
    tracing::info!("running health check");
    health_check()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_returns_result() {
        // Should not error regardless of environment.
        let status = health_check().unwrap();
        // At least the struct should be populated.
        assert!(!status.syncthing.label.is_empty());
        assert!(!status.hermes.label.is_empty());
        assert!(!status.obsidian.label.is_empty());
    }

    #[test]
    fn test_check_hermes_returns_status() {
        let status = check_hermes();
        assert_eq!(status.label, "Hermes CLI");
        // available may be true or false depending on environment.
        let _ = status.available;
    }

    #[test]
    fn test_check_syncthing_returns_status() {
        let status = check_syncthing();
        assert_eq!(status.label, "Syncthing");
        let _ = status.available;
    }
}
