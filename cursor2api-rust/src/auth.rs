use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
};
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // 无 token 配置则全放行
    if state.config.auth_tokens.is_empty() {
        return Ok(next.run(req).await);
    }

    // 从 query param 取 token
    let query_token = req.uri().query().and_then(|q| {
        url_query_token(q)
    });

    // 从 Authorization header 取 token
    let header_token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(|t| t.to_string()));

    let provided = query_token.or(header_token);

    if let Some(token) = provided {
        if state.config.auth_tokens.iter().any(|t| t == &token) {
            return Ok(next.run(req).await);
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Unauthorized" })),
    ))
}

fn url_query_token(query: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next()?;
        let val = parts.next()?;
        if key == "token" {
            Some(urlencoding_decode(val))
        } else {
            None
        }
    })
}

fn urlencoding_decode(s: &str) -> String {
    // 简单 percent-decode
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            if let Ok(b) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                result.push(b as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}
