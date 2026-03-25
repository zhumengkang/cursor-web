use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use std::sync::Arc;
use crate::state::AppState;

/// 将 /v1/* 请求反向代理到 Node.js cursor2api（端口 3010）
pub async fn proxy_v1(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response<Body>, (StatusCode, String)> {
    let node_port = std::env::var("NODE_PORT").unwrap_or_else(|_| "3010".to_string());
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    let target_url = format!("http://127.0.0.1:{}{}", node_port, path_and_query);

    let method = req.method().clone();
    let headers = req.headers().clone();
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let client = reqwest::Client::new();
    let mut builder = client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap(),
        &target_url,
    );

    // 转发请求头（排除 host）
    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        if name_str == "host" || name_str == "transfer-encoding" {
            continue;
        }
        if let Ok(v) = value.to_str() {
            builder = builder.header(name_str, v);
        }
    }

    let resp = builder
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("转发请求失败: {}", e)))?;

    let status = resp.status();
    let resp_headers = resp.headers().clone();

    // 流式转发响应体
    let resp_body = resp.bytes_stream();
    let stream_body = Body::from_stream(resp_body);

    let mut response = Response::builder().status(status.as_u16());
    for (name, value) in resp_headers.iter() {
        let name_str = name.as_str();
        if name_str == "transfer-encoding" {
            continue;
        }
        response = response.header(name_str, value);
    }

    response
        .body(stream_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
