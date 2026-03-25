use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

use crate::{
    config::{read_config, write_config},
    state::AppState,
};

#[derive(Serialize)]
pub struct ProxiesResponse {
    pub proxies: Vec<String>,
    pub proxy: Option<String>,
}

#[derive(Deserialize)]
pub struct SetProxiesRequest {
    pub proxies: Vec<String>,
}

#[derive(Serialize)]
pub struct TestAllProxiesResponse {
    pub results: Vec<ProxyTestResult>,
    pub round_robin_order: Vec<String>,
}

#[derive(Serialize)]
pub struct ProxyTestResult {
    pub proxy: String,
    pub ok: bool,
    pub latency: Option<u64>,
    pub status: Option<u16>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct TestAllProxiesRequest {
    pub target: Option<String>,
}

#[derive(Deserialize)]
pub struct TestProxyRequest {
    pub proxy: String,
    /// 测试目标 URL，默认 https://www.gstatic.com/generate_204
    pub target: Option<String>,
}

#[derive(Serialize)]
pub struct TestProxyResponse {
    pub ok: bool,
    pub latency: Option<u64>,  // ms
    pub status: Option<u16>,
    pub error: Option<String>,
}

/// GET /api/proxies - 读取代理列表
pub async fn get_proxies(State(state): State<Arc<AppState>>) -> Json<ProxiesResponse> {
    let cfg = read_config(&state.config.config_path).unwrap_or_default();
    Json(ProxiesResponse {
        proxies: cfg.proxies.unwrap_or_default(),
        proxy: cfg.proxy,
    })
}

/// POST /api/proxies - 保存代理列表
pub async fn set_proxies(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetProxiesRequest>,
) -> Result<Json<ProxiesResponse>, (axum::http::StatusCode, String)> {
    let mut cfg = read_config(&state.config.config_path).map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // 去重 + 过滤空行
    let mut proxies: Vec<String> = req
        .proxies
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    proxies.dedup();

    cfg.proxies = if proxies.is_empty() { None } else { Some(proxies.clone()) };

    write_config(&state.config.config_path, &cfg).map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(ProxiesResponse {
        proxies,
        proxy: cfg.proxy,
    }))
}

/// POST /api/proxies/test - 服务端通过指定代理测试连通性和延迟
pub async fn test_proxy(
    Json(req): Json<TestProxyRequest>,
) -> Json<TestProxyResponse> {
    let proxy_url = req.proxy.trim().to_string();
    let target = req
        .target
        .unwrap_or_else(|| "https://www.gstatic.com/generate_204".to_string());

    if proxy_url.is_empty() {
        return Json(TestProxyResponse {
            ok: false,
            latency: None,
            status: None,
            error: Some("代理地址不能为空".to_string()),
        });
    }

    let client = match reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).unwrap_or_else(|_| reqwest::Proxy::all("http://127.0.0.1:1").unwrap()))
        .timeout(Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return Json(TestProxyResponse {
                ok: false,
                latency: None,
                status: None,
                error: Some(format!("构建客户端失败: {}", e)),
            });
        }
    };

    let start = Instant::now();
    match client.get(&target).send().await {
        Ok(res) => {
            let latency = start.elapsed().as_millis() as u64;
            let status = res.status().as_u16();
            let ok = res.status().is_success() || status == 204;
            Json(TestProxyResponse {
                ok,
                latency: Some(latency),
                status: Some(status),
                error: if ok { None } else { Some(format!("HTTP {}", status)) },
            })
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;
            Json(TestProxyResponse {
                ok: false,
                latency: Some(latency),
                status: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// POST /api/proxies/test-all - 并发测试所有代理（轮询负载均衡验证）
pub async fn test_all_proxies(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TestAllProxiesRequest>,
) -> Json<TestAllProxiesResponse> {
    let target = req
        .target
        .unwrap_or_else(|| "https://www.gstatic.com/generate_204".to_string());

    let cfg = read_config(&state.config.config_path).unwrap_or_default();
    let proxies: Vec<String> = {
        let mut list = cfg.proxies.unwrap_or_default();
        if let Some(p) = cfg.proxy {
            if !list.contains(&p) {
                list.push(p);
            }
        }
        list
    };

    if proxies.is_empty() {
        return Json(TestAllProxiesResponse {
            results: vec![],
            round_robin_order: vec![],
        });
    }

    // 并发测试所有代理
    let mut join_set: JoinSet<ProxyTestResult> = JoinSet::new();
    for proxy_url in proxies.clone() {
        let target = target.clone();
        join_set.spawn(async move {
            let client = match reqwest::Client::builder()
                .proxy(
                    reqwest::Proxy::all(&proxy_url)
                        .unwrap_or_else(|_| reqwest::Proxy::all("http://127.0.0.1:1").unwrap()),
                )
                .timeout(Duration::from_secs(8))
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    return ProxyTestResult {
                        proxy: proxy_url,
                        ok: false,
                        latency: None,
                        status: None,
                        error: Some(format!("构建客户端失败: {}", e)),
                    };
                }
            };
            let start = Instant::now();
            match client.get(&target).send().await {
                Ok(res) => {
                    let latency = start.elapsed().as_millis() as u64;
                    let status = res.status().as_u16();
                    let ok = res.status().is_success() || status == 204;
                    ProxyTestResult {
                        proxy: proxy_url,
                        ok,
                        latency: Some(latency),
                        status: Some(status),
                        error: if ok { None } else { Some(format!("HTTP {}", status)) },
                    }
                }
                Err(e) => {
                    let latency = start.elapsed().as_millis() as u64;
                    ProxyTestResult {
                        proxy: proxy_url,
                        ok: false,
                        latency: Some(latency),
                        status: None,
                        error: Some(e.to_string()),
                    }
                }
            }
        });
    }

    let mut results: Vec<ProxyTestResult> = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok(r) = res {
            results.push(r);
        }
    }

    // 按原始 proxies 顺序排序，方便对照轮询顺序
    let order = proxies.clone();
    results.sort_by_key(|r| order.iter().position(|p| p == &r.proxy).unwrap_or(usize::MAX));

    Json(TestAllProxiesResponse {
        round_robin_order: proxies,
        results,
    })
}
