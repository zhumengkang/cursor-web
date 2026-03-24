use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub config_path: String,
    pub db_path: String,
    pub log_dir: String,
    pub auth_tokens: Vec<String>,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "../cursor2api/config.yaml".to_string());
        let db_path = std::env::var("DB_PATH")
            .unwrap_or_else(|_| "../cursor2api/logs/cursor2api.db".to_string());
        let log_dir = std::env::var("LOG_DIR")
            .unwrap_or_else(|_| "../cursor2api/logs".to_string());
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
            auth_tokens,
        }
    }
}

pub struct AppState {
    pub config: ServerConfig,
    pub sse_tx: broadcast::Sender<String>,
}
