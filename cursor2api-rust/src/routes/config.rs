use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::{
    config::{read_config, write_config},
    state::AppState,
    types::HotConfig,
};

pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let config_path = &state.config.config_path;
    match read_config(config_path) {
        Ok(cfg) => Ok(Json(json!({ "config": cfg }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn post_config(
    State(state): State<Arc<AppState>>,
    Json(body): Json<HotConfig>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let config_path = &state.config.config_path;

    // 读取现有配置后合并
    let mut current = read_config(config_path).unwrap_or_default();
    let mut changes: Vec<String> = Vec::new();

    macro_rules! merge_field {
        ($field:ident, $label:expr) => {
            if let Some(val) = body.$field {
                let old = format!("{:?}", current.$field);
                let new = format!("{:?}", &val);
                if old != new {
                    changes.push(format!("{}: {} → {}", $label, old, new));
                    current.$field = Some(val);
                }
            }
        };
    }

    merge_field!(cursor_model, "cursor_model");
    merge_field!(timeout, "timeout");
    merge_field!(max_auto_continue, "max_auto_continue");
    merge_field!(max_history_messages, "max_history_messages");
    merge_field!(max_history_tokens, "max_history_tokens");
    merge_field!(sanitize_response, "sanitize_response");
    merge_field!(thinking, "thinking");
    merge_field!(compression, "compression");
    merge_field!(tools, "tools");
    merge_field!(logging, "logging");

    match write_config(config_path, &current) {
        Ok(()) => Ok(Json(json!({ "ok": true, "changes": changes }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
