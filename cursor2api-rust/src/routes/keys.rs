use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::collections::HashMap;

use crate::{
    config::{read_config, write_config},
    logger::db_get_requests,
    state::AppState,
    types::{KeyStats, ModelBreakdown, DailyBreakdown},
};

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    since: Option<u64>,
    until: Option<u64>,
    granularity: Option<String>,
    key: Option<String>,
}

fn server_err(msg: impl ToString) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": msg.to_string() })))
}

fn bad_req(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg })))
}

// GET /api/keys  -> [{id, name, keyValue, enabled, createdAt}]
pub async fn get_keys(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cfg = read_config(&state.config.config_path).map_err(|e| server_err(e))?;
    let tokens = cfg.auth_tokens.unwrap_or_default();
    let keys: Vec<Value> = tokens
        .iter()
        .map(|t| json!({
            "id": t,
            "name": t,
            "keyValue": t,
            "enabled": true,
            "createdAt": 0
        }))
        .collect();
    Ok(Json(json!({ "keys": keys })))
}

// POST /api/keys  body: { keyValue: "sk-xxx" }
pub async fn post_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = body["keyValue"].as_str().unwrap_or("").trim().to_string();
    if token.is_empty() {
        return Err(bad_req("keyValue is required"));
    }
    let config_path = state.config.config_path.clone();
    let token_clone = token.clone();
    tokio::task::spawn_blocking(move || {
        let mut cfg = read_config(&config_path)?;
        let mut tokens = cfg.auth_tokens.unwrap_or_default();
        if tokens.contains(&token_clone) {
            return Err(anyhow::anyhow!("token already exists"));
        }
        tokens.push(token_clone.clone());
        cfg.auth_tokens = Some(tokens);
        write_config(&config_path, &cfg)?;
        Ok(())
    })
    .await
    .map_err(|e| server_err(e))?
    .map_err(|e: anyhow::Error| {
        if e.to_string().contains("already exists") {
            bad_req("token already exists")
        } else {
            server_err(e)
        }
    })?;
    Ok(Json(json!({
        "key": {
            "id": token,
            "name": token,
            "keyValue": token,
            "enabled": true,
            "createdAt": 0
        }
    })))
}

// DELETE /api/keys/:id  (id = token value, URL encoded)
pub async fn delete_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let config_path = state.config.config_path.clone();
    tokio::task::spawn_blocking(move || {
        let mut cfg = read_config(&config_path)?;
        let mut tokens = cfg.auth_tokens.unwrap_or_default();
        let before = tokens.len();
        tokens.retain(|t| t != &id);
        if tokens.len() == before {
            return Err(anyhow::anyhow!("not found"));
        }
        cfg.auth_tokens = Some(tokens);
        write_config(&config_path, &cfg)
    })
    .await
    .map_err(|e| server_err(e))?
    .map_err(|e: anyhow::Error| server_err(e))?;
    Ok(Json(json!({ "ok": true })))
}

// PUT /api/keys/:id  - 不支持编辑，仅为兼容前端路由
pub async fn put_key(
    _state: State<Arc<AppState>>,
    _id: Path<String>,
    _body: Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err(bad_req("editing token value is not supported, delete and re-add instead"))
}

// GET /api/keys/stats?since=&until=&granularity=
pub async fn get_key_stats(
    State(state): State<Arc<AppState>>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db_path = state.config.db_path.clone();
    let since = q.since;
    let until = q.until;
    let granularity = q.granularity.unwrap_or_else(|| "day".to_string());
    let key_filter = q.key.clone();

    let result = tokio::task::spawn_blocking(move || {
        db_get_requests(&db_path, 10000, None, since, None, None)
    })
    .await
    .map_err(|e| server_err(e))?;

    let summaries = match result {
        Ok((reqs, _)) => reqs,
        Err(e) => return Err(server_err(e)),
    };

    // 时间 + key 过滤
    let summaries: Vec<_> = summaries.into_iter().filter(|s| {
        if let Some(from) = since { if s.start_time < from { return false; } }
        if let Some(to) = until { if s.start_time > to { return false; } }
        if let Some(ref k) = key_filter {
            if s.auth_token.as_deref() != Some(k.as_str()) { return false; }
        }
        true
    }).collect();

    let total = summaries.len() as u64;
    let success = summaries.iter().filter(|s| s.status.as_deref() == Some("success")).count() as u64;
    let error = summaries.iter().filter(|s| s.status.as_deref() == Some("error")).count() as u64;
    let degraded = summaries.iter().filter(|s| s.status.as_deref() == Some("degraded")).count() as u64;
    let success_rate = if total > 0 { success as f64 / total as f64 * 100.0 } else { 0.0 };

    let total_input: u64 = summaries.iter().filter_map(|s| s.input_tokens).sum();
    let total_output: u64 = summaries.iter().filter_map(|s| s.output_tokens).sum();

    let durations: Vec<f64> = summaries.iter().filter_map(|s| {
        s.end_time.map(|e| (e - s.start_time) as f64)
    }).collect();
    let avg_rt = if durations.is_empty() { 0.0 } else { durations.iter().sum::<f64>() / durations.len() as f64 };

    let avg_in = if total > 0 { total_input as f64 / total as f64 } else { 0.0 };
    let avg_out = if total > 0 { total_output as f64 / total as f64 } else { 0.0 };

    // 模型分布
    let mut model_map: HashMap<String, (u64, u64, u64)> = HashMap::new();
    for s in &summaries {
        let model = s.model.clone().unwrap_or_else(|| "unknown".to_string());
        let e = model_map.entry(model).or_insert((0, 0, 0));
        e.0 += 1;
        e.1 += s.input_tokens.unwrap_or(0);
        e.2 += s.output_tokens.unwrap_or(0);
    }
    let mut models_breakdown: Vec<ModelBreakdown> = model_map.into_iter().map(|(model, (count, input, output))| {
        ModelBreakdown { model, count, input_tokens: input, output_tokens: output }
    }).collect();
    models_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

    // 时间分组
    let mut period_map: HashMap<String, (u64, u64, u64)> = HashMap::new();
    for s in &summaries {
        let ts = s.start_time;
        let key = match granularity.as_str() {
            "hour" => {
                let secs = ts / 1000;
                let h = secs - secs % 3600;
                format!("{}", chrono_fmt_hour(h))
            }
            _ => {
                let secs = ts / 1000;
                let d = secs - secs % 86400;
                format!("{}", chrono_fmt_day(d))
            }
        };
        let e = period_map.entry(key).or_insert((0, 0, 0));
        e.0 += 1;
        e.1 += s.input_tokens.unwrap_or(0);
        e.2 += s.output_tokens.unwrap_or(0);
    }
    let mut period_breakdown: Vec<DailyBreakdown> = period_map.into_iter().map(|(date, (count, input, output))| {
        DailyBreakdown { date, count, input_tokens: input, output_tokens: output }
    }).collect();
    period_breakdown.sort_by(|a, b| a.date.cmp(&b.date));

    let stats = KeyStats {
        total_requests: total,
        success_count: success,
        error_count: error,
        degraded_count: degraded,
        success_rate,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_tokens: total_input + total_output,
        avg_input_tokens: avg_in,
        avg_output_tokens: avg_out,
        avg_response_time: avg_rt,
        models_breakdown,
        period_breakdown,
        granularity,
    };

    Ok(Json(json!({ "stats": stats })))
}

fn chrono_fmt_day(secs: u64) -> String {
    let d = secs / 86400;
    let y = 1970 + d / 365;
    let rem = d % 365;
    let m = rem / 30 + 1;
    let day = rem % 30 + 1;
    format!("{:04}-{:02}-{:02}", y, m.min(12), day.min(31))
}

fn chrono_fmt_hour(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let y = 1970 + d / 365;
    let rem = d % 365;
    let m = rem / 30 + 1;
    let day = rem % 30 + 1;
    format!("{:04}-{:02}-{:02} {:02}:00", y, m.min(12), day.min(31), h)
}
