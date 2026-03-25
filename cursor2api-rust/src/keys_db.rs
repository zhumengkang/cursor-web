use anyhow::Result;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::types::{ApiKey, CreateKeyRequest, UpdateKeyRequest};

pub fn init_keys_db(path: &str) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS api_keys (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            key_value   TEXT NOT NULL,
            note        TEXT,
            enabled     INTEGER NOT NULL DEFAULT 1,
            created_at  INTEGER NOT NULL,
            last_used   INTEGER
        );",
    )?;
    Ok(())
}

fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return "*".repeat(chars.len());
    }
    let prefix: String = chars[..4].iter().collect();
    let suffix: String = chars[chars.len() - 4..].iter().collect();
    let masked = "*".repeat(chars.len().saturating_sub(8).min(12));
    format!("{}{}{}", prefix, masked, suffix)
}

pub fn list_keys(path: &str) -> Result<Vec<ApiKey>> {
    let conn = Connection::open(path)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, key_value, note, enabled, created_at, last_used FROM api_keys ORDER BY created_at DESC",
    )?;
    let keys = stmt
        .query_map([], |row| {
            let key_value: String = row.get(2)?;
            Ok(ApiKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key_preview: mask_key(&key_value),
                note: row.get(3)?,
                enabled: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                last_used: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(keys)
}

pub fn create_key(path: &str, req: &CreateKeyRequest) -> Result<ApiKey> {
    let conn = Connection::open(path)?;
    let id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    conn.execute(
        "INSERT INTO api_keys (id, name, key_value, note, enabled, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        params![id, req.name, req.key_value, req.note, now],
    )?;
    Ok(ApiKey {
        id,
        name: req.name.clone(),
        key_preview: mask_key(&req.key_value),
        note: req.note.clone(),
        enabled: true,
        created_at: now,
        last_used: None,
    })
}

pub fn update_key(path: &str, id: &str, req: &UpdateKeyRequest) -> Result<bool> {
    let conn = Connection::open(path)?;
    let rows = conn.execute(
        "UPDATE api_keys SET name = ?1, note = ?2, enabled = ?3 WHERE id = ?4",
        params![
            req.name,
            req.note,
            if req.enabled { 1i64 } else { 0i64 },
            id
        ],
    )?;
    Ok(rows > 0)
}

pub fn delete_key(path: &str, id: &str) -> Result<bool> {
    let conn = Connection::open(path)?;
    let rows = conn.execute("DELETE FROM api_keys WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}
