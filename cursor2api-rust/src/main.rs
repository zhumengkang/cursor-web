mod auth;
mod config;
mod logger;
mod routes;
mod state;
mod types;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::auth_middleware,
    routes::{config as config_routes, logs as logs_routes, models as models_routes, sse as sse_routes},
    state::{AppState, ServerConfig},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_config = ServerConfig::from_env();
    let port = server_config.port;
    let log_dir = server_config.log_dir.clone();
    let db_path = server_config.db_path.clone();

    let (sse_tx, _) = broadcast::channel::<String>(256);

    // 启动文件监听
    sse_routes::start_file_watcher(log_dir, db_path, sse_tx.clone());

    let state = Arc::new(AppState {
        config: server_config,
        sse_tx,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 需要鉴权的 API 路由
    let api_routes = Router::new()
        .route("/api/models", get(models_routes::get_models))
        .route("/api/config", get(config_routes::get_config).post(config_routes::post_config))
        .route("/api/logs", get(logs_routes::get_logs))
        .route("/api/logs/clear", post(logs_routes::clear_logs))
        .route("/api/logs/stream", get(sse_routes::sse_logs))
        .route("/api/requests", get(logs_routes::get_requests))
        .route("/api/requests/more", get(logs_routes::get_requests_more))
        .route("/api/vue/stats", get(logs_routes::get_stats))
        .route("/api/payload/:request_id", get(logs_routes::get_payload))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        // 静态文件服务（Vue UI）
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("cursor2api-rust 启动，监听 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok", "service": "cursor2api-rust" }))
}
