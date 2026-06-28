use std::path::PathBuf;
use tracing_subscriber::{
    fmt::writer::MakeWriterExt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging(log_dir: PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("Failed to create log directory: {}", e))?;

    let log_file = tracing_appender::rolling::daily(&log_dir, "nerdforge.log");

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file.with_max_level(tracing::Level::INFO))
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout.with_max_level(tracing::Level::DEBUG))
        .with_ansi(false)
        .with_target(true);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}

pub fn default_log_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("NerdForge")
        .join("logs")
}

#[tauri::command]
pub fn get_log_path() -> Result<String, String> {
    let path = default_log_dir().join("nerdforge.log");
    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_log_dir_is_not_empty() {
        let dir = default_log_dir();
        assert!(!dir.to_string_lossy().is_empty());
    }

    #[test]
    fn test_init_logging_creates_dir() {
        let dir = std::env::temp_dir().join("nerdforge_log_test");
        let _ = fs::remove_dir_all(&dir);
        init_logging(dir.clone()).unwrap();
        assert!(dir.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
