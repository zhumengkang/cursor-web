use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{
    logger::{
        db_get_logs, db_get_payload, db_get_requests, db_get_stats, db_clear,
        jsonl_get_logs, jsonl_get_requests, jsonl_clear,
    },
    state::AppState,
    types::RequestsPage,
};

fn use_db(state: &AppState) -> bool {
    std::path::Path::new(&state.config.db_path).exists()
}

// GET /api/requests
#[derive(Debug, Deserialize)]
pub struct RequestsQuery {
    limit: Option<usize>,
    since: Option<u64>,
    status: Option<String>,
    keyword: Option<String>,
}

pub async fn get_requests(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RequestsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = q.limit.unwrap_or(50).min(500);
    let result = if use_db(&state) {
        db_get_requests(
            &state.config.db_path,
            limit,
            None,
            q.since,
            q.status.as_deref(),
            q.keyword.as_deref(),
        )
    } else {
        jsonl_get_requests(
            &state.config.log_dir,
            limit,
            None,
            q.since,
            q.status.as_deref(),
            q.keyword.as_deref(),
        )
    };
    match result {
        Ok((requests, has_more)) => Ok(Json(json!({
            "requests": requests,
            "hasMore": has_more,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

// GET /api/requests/more
#[derive(Debug, Deserialize)]
pub struct MoreQuery {
    before: Option<u64>,
    limit: Option<usize>,
    since: Option<u64>,
    status: Option<String>,
    keyword: Option<String>,
}

pub async fn get_requests_more(
    State(state): State<Arc<AppState>>,
    Query(q): Query<MoreQuery>,
) -> Result<Json<RequestsPage>, (StatusCode, Json<Value>)> {
    let limit = q.limit.unwrap_or(50).min(500);
    let result = if use_db(&state) {
        db_get_requests(
            &state.config.db_path,
            limit,
            q.before,
            q.since,
            q.status.as_deref(),
            q.keyword.as_deref(),
        )
    } else {
        jsonl_get_requests(
            &state.config.log_dir,
            limit,
            q.before,
            q.since,
            q.status.as_deref(),
            q.keyword.as_deref(),
        )
    };
    match result {
        Ok((requests, has_more)) => Ok(Json(RequestsPage {
            requests,
            has_more,
            next_cursor: None,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

// GET /api/logs
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    level: Option<String>,
    source: Option<String>,
    limit: Option<usize>,
    since: Option<u64>,
}

pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LogsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = q.limit.unwrap_or(200).min(1000);
    let result = if use_db(&state) {
        db_get_logs(
            &state.config.db_path,
            q.request_id.as_deref(),
            q.level.as_deref(),
            q.source.as_deref(),
            limit,
            q.since,
        )
    } else {
        jsonl_get_logs(
            &state.config.log_dir,
            q.request_id.as_deref(),
            q.level.as_deref(),
            q.source.as_deref(),
            limit,
            q.since,
        )
    };
    match result {
        Ok(logs) => Ok(Json(json!({ "logs": logs }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

// GET /api/vue/stats
#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    since: Option<u64>,
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = db_get_stats(&state.config.db_path, q.since);
    match result {
        Ok(stats) => Ok(Json(json!({ "stats": stats }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

// GET /api/payload/:requestId
pub async fn get_payload(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match db_get_payload(&state.config.db_path, &request_id) {
        Ok(Some(payload)) => Ok(Json(json!({ "payload": payload }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "payload not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

// POST /api/logs/clear
pub async fn clear_logs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db_result = db_clear(&state.config.db_path);
    let jsonl_result = jsonl_clear(&state.config.log_dir);
    match (db_result, jsonl_result) {
        (Ok(_), Ok(_)) => Ok(Json(json!({ "ok": true }))),
        (Err(e), _) | (_, Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
