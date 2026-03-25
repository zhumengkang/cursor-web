mod auth;
mod config;
mod logger;
mod node_service;
mod routes;
mod state;
mod types;

use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    middleware,
    response::Response,
    routing::{get, post, put},
    Router,
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::auth_middleware,
    routes::{
        config as config_routes, keys as keys_routes, logs as logs_routes,
        models as models_routes, proxies as proxies_routes, sse as sse_routes,
    },
    state::{AppState, ServerConfig},
};

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

async fn serve_embedded(path: Option<Path<String>>) -> Response<Body> {
    let path = path.map(|p| p.0).unwrap_or_default();
    let path = if path.is_empty() || path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    match StaticAssets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // SPA fallback: 返回 index.html
            match StaticAssets::get("index.html") {
                Some(content) => Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(content.data.into_owned()))
                    .unwrap(),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap(),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 获取 exe 所在目录，启动内嵌的 cursor2api Node.js 服务
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let _node_child = node_service::setup_and_spawn(&exe_dir);

    // 等待 Node.js 启动
    if _node_child.is_some() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    }

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
        .route("/api/keys/stats", get(keys_routes::get_key_stats))
        .route("/api/keys", get(keys_routes::get_keys).post(keys_routes::post_key))
        .route("/api/keys/:id", put(keys_routes::put_key).delete(keys_routes::delete_key))
        .route("/api/proxies", get(proxies_routes::get_proxies).post(proxies_routes::set_proxies))
        .route("/api/proxies/test", post(proxies_routes::test_proxy))
        .route("/api/proxies/test-all", post(proxies_routes::test_all_proxies))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        // 静态文件（嵌入到 exe 内）
        .route("/", get(|| serve_embedded(None)))
        .route("/*path", get(|p: Path<String>| serve_embedded(Some(p))))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("kkproxy 启动，监听 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok", "service": "kkproxy" }))
}
