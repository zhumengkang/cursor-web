use anyhow::Result;
use rusqlite::{Connection, params};
use serde_json;
use std::fs;
use std::path::Path;

use crate::types::{LogEntry, RequestSummary, Stats};

// ==================== SQLite ====================

pub fn db_get_requests(
    db_path: &str,
    limit: usize,
    before: Option<u64>,
    since: Option<u64>,
    status: Option<&str>,
    keyword: Option<&str>,
) -> Result<(Vec<RequestSummary>, bool)> {
    if !Path::new(db_path).exists() {
        return Ok((vec![], false));
    }
    let conn = Connection::open(db_path)?;
    let fetch_limit = limit + 1;

    let mut conditions = vec!["1=1".to_string()];
    if let Some(b) = before {
        conditions.push(format!("timestamp < {}", b));
    }
    if let Some(s) = since {
        conditions.push(format!("timestamp >= {}", s));
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT summary_json FROM requests WHERE {} ORDER BY timestamp DESC LIMIT {}",
        where_clause, fetch_limit
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let has_more = rows.len() > limit;
    let rows = &rows[..rows.len().min(limit)];

    let mut summaries: Vec<RequestSummary> = rows
        .iter()
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect();

    // 应用 status / keyword 过滤（内存过滤，数据量小时可接受）
    if let Some(st) = status {
        summaries.retain(|s| s.status.as_deref() == Some(st));
    }
    if let Some(kw) = keyword {
        let kw_lower = kw.to_lowercase();
        summaries.retain(|s| {
            s.model.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower)
                || s.request_id.to_lowercase().contains(&kw_lower)
                || s.title.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower)
        });
    }

    Ok((summaries, has_more))
}

pub fn db_get_logs(
    db_path: &str,
    request_id: Option<&str>,
    level: Option<&str>,
    source: Option<&str>,
    limit: usize,
    since: Option<u64>,
) -> Result<Vec<LogEntry>> {
    // 日志存在 JSONL，SQLite 中仅有 summary
    // 此函数从 JSONL 读取
    jsonl_get_logs(
        &format!("{}", db_path.replace(".db", ".jsonl")),
        request_id,
        level,
        source,
        limit,
        since,
    )
}

pub fn db_get_payload(db_path: &str, request_id: &str) -> Result<Option<serde_json::Value>> {
    if !Path::new(db_path).exists() {
        return Ok(None);
    }
    let conn = Connection::open(db_path)?;
    let result: Option<String> = conn
        .query_row(
            "SELECT payload_json FROM requests WHERE request_id = ?",
            params![request_id],
            |row| row.get(0),
        )
        .ok();
    match result {
        Some(s) => Ok(serde_json::from_str(&s).ok()),
        None => Ok(None),
    }
}

pub fn db_get_stats(db_path: &str, since: Option<u64>) -> Result<Stats> {
    if !Path::new(db_path).exists() {
        return Ok(Stats {
            total_requests: 0,
            success_count: 0,
            degraded_count: 0,
            error_count: 0,
            avg_response_time: 0.0,
            avg_ttft: 0.0,
        });
    }
    let conn = Connection::open(db_path)?;

    let since_clause = since
        .map(|s| format!(" WHERE timestamp >= {}", s))
        .unwrap_or_default();

    let sql = format!("SELECT summary_json FROM requests{}", since_clause);
    let mut stmt = conn.prepare(&sql)?;
    let summaries: Vec<RequestSummary> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .filter_map(|s| serde_json::from_str(&s).ok())
        .collect();

    let total = summaries.len() as u64;
    let success = summaries.iter().filter(|s| s.status.as_deref() == Some("success")).count() as u64;
    let degraded = summaries.iter().filter(|s| s.status.as_deref() == Some("degraded")).count() as u64;
    let error = summaries.iter().filter(|s| s.status.as_deref() == Some("error")).count() as u64;

    let durations: Vec<f64> = summaries
        .iter()
        .filter_map(|s| {
            if let (Some(end), start) = (s.end_time, s.start_time) {
                Some((end - start) as f64)
            } else {
                None
            }
        })
        .collect();
    let avg_rt = if durations.is_empty() {
        0.0
    } else {
        durations.iter().sum::<f64>() / durations.len() as f64
    };

    let ttfts: Vec<f64> = summaries
        .iter()
        .filter_map(|s| s.ttft.map(|t| t as f64))
        .collect();
    let avg_ttft = if ttfts.is_empty() {
        0.0
    } else {
        ttfts.iter().sum::<f64>() / ttfts.len() as f64
    };

    Ok(Stats {
        total_requests: total,
        success_count: success,
        degraded_count: degraded,
        error_count: error,
        avg_response_time: avg_rt,
        avg_ttft,
    })
}

pub fn db_clear(db_path: &str) -> Result<()> {
    if !Path::new(db_path).exists() {
        return Ok(());
    }
    let conn = Connection::open(db_path)?;
    conn.execute("DELETE FROM requests", [])?;
    Ok(())
}

// ==================== JSONL ====================

pub fn jsonl_get_logs(
    log_path: &str,
    request_id: Option<&str>,
    level: Option<&str>,
    source: Option<&str>,
    limit: usize,
    since: Option<u64>,
) -> Result<Vec<LogEntry>> {
    // 尝试在 log_dir 下查找 *.jsonl 文件
    let path = Path::new(log_path);
    let mut entries: Vec<LogEntry> = Vec::new();

    let files: Vec<std::path::PathBuf> = if path.is_dir() {
        fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .collect()
    } else if path.exists() {
        vec![path.to_path_buf()]
    } else {
        vec![]
    };

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                // 仅处理 type=log 的条目
                if entry.get("type").and_then(|v| v.as_str()) != Some("log") {
                    continue;
                }
                let data = entry.get("data").unwrap_or(&entry);
                let log: LogEntry = match serde_json::from_value(data.clone()) {
                    Ok(l) => l,
                    Err(_) => continue,
                };
                if let Some(rid) = request_id {
                    if log.request_id != rid {
                        continue;
                    }
                }
                if let Some(lv) = level {
                    if log.level != lv {
                        continue;
                    }
                }
                if let Some(src) = source {
                    if log.source != src {
                        continue;
                    }
                }
                if let Some(s) = since {
                    if log.timestamp < s {
                        continue;
                    }
                }
                entries.push(log);
            }
        }
    }

    // 按时间降序，取前 limit 条
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(limit);
    Ok(entries)
}

pub fn jsonl_get_requests(
    log_dir: &str,
    limit: usize,
    before: Option<u64>,
    since: Option<u64>,
    status: Option<&str>,
    keyword: Option<&str>,
) -> Result<(Vec<RequestSummary>, bool)> {
    let path = Path::new(log_dir);
    let mut summaries: Vec<RequestSummary> = Vec::new();

    let files: Vec<std::path::PathBuf> = if path.is_dir() {
        fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .collect()
    } else {
        vec![]
    };

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                if entry.get("type").and_then(|v| v.as_str()) != Some("summary") {
                    continue;
                }
                let data = entry.get("data").unwrap_or(&entry);
                let Ok(summary) = serde_json::from_value::<RequestSummary>(data.clone()) else {
                    continue;
                };
                if let Some(b) = before {
                    if summary.start_time >= b {
                        continue;
                    }
                }
                if let Some(s) = since {
                    if summary.start_time < s {
                        continue;
                    }
                }
                if let Some(st) = status {
                    if summary.status.as_deref() != Some(st) {
                        continue;
                    }
                }
                if let Some(kw) = keyword {
                    let kw_lower = kw.to_lowercase();
                    let matches = summary.model.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower)
                        || summary.request_id.to_lowercase().contains(&kw_lower)
                        || summary.title.as_deref().unwrap_or("").to_lowercase().contains(&kw_lower);
                    if !matches {
                        continue;
                    }
                }
                summaries.push(summary);
            }
        }
    }

    summaries.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    let has_more = summaries.len() > limit;
    summaries.truncate(limit);
    Ok((summaries, has_more))
}

pub fn jsonl_clear(log_dir: &str) -> Result<()> {
    let path = Path::new(log_dir);
    if !path.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            fs::write(&p, "")?;
        }
    }
    Ok(())
}
