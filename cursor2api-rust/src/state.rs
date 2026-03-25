use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub config_path: String,
    pub db_path: String,
    pub log_dir: String,
    pub keys_db_path: String,
    pub auth_tokens: Vec<String>,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        // 默认路径相对于 exe 所在目录（生产模式）
        // 开发时可通过环境变量覆盖
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| exe_dir.join("config.yaml").to_string_lossy().to_string());
        let db_path = std::env::var("DB_PATH")
            .unwrap_or_else(|_| exe_dir.join("logs").join("cursor2api.db").to_string_lossy().to_string());
        let log_dir = std::env::var("LOG_DIR")
            .unwrap_or_else(|_| exe_dir.join("logs").to_string_lossy().to_string());
        let keys_db_path = std::env::var("KEYS_DB_PATH")
            .unwrap_or_else(|_| exe_dir.join("keys.db").to_string_lossy().to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);
        let auth_tokens = std::env::var("AUTH_TOKENS")
            .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
            .unwrap_or_default();
        Self {
            port,
            config_path,
            db_path,
            log_dir,
            keys_db_path,
            auth_tokens,
        }
    }
}

pub struct AppState {
    pub config: ServerConfig,
    pub sse_tx: broadcast::Sender<String>,
}
