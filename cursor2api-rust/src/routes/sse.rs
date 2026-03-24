use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    convert::Infallible,
    path::Path,
    sync::Arc,
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::state::AppState;

pub async fn sse_logs(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rx = state.sse_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| {
            msg.ok().map(|data| {
                Ok::<Event, Infallible>(Event::default().data(data))
            })
        });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    )
}

/// 启动文件监听，变更时向 SSE 广播
pub fn start_file_watcher(
    log_dir: String,
    db_path: String,
    tx: broadcast::Sender<String>,
) {
    tokio::spawn(async move {
        let (watcher_tx, mut watcher_rx) = tokio::sync::mpsc::channel(64);

        let mut watcher = match RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = watcher_tx.blocking_send(event);
                }
            },
            Config::default(),
        ) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("创建文件监听失败: {}", e);
                return;
            }
        };

        let log_path = Path::new(&log_dir);
        if log_path.exists() {
            if let Err(e) = watcher.watch(log_path, RecursiveMode::NonRecursive) {
                tracing::warn!("监听日志目录失败: {}", e);
            }
        }

        let db = Path::new(&db_path);
        if db.exists() {
            if let Some(parent) = db.parent() {
                if parent.exists() {
                    let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
                }
            }
        }

        while let Some(event) = watcher_rx.recv().await {
            use notify::EventKind::*;
            match event.kind {
                Modify(_) | Create(_) => {
                    let msg = serde_json::json!({ "type": "refresh" }).to_string();
                    let _ = tx.send(msg);
                    // 短暂去抖，避免密集发送
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                _ => {}
            }
        }
    });
}
