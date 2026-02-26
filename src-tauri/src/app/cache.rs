use std::error::Error;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};
use crate::app::DbState;

#[derive(serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PreviewData {
    id: i32,
    preview: String,
    created_at: i32,
    last_used_at: i32,
    use_count: i32
}

#[derive(serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ClipboardData {
    id: i32,
    content: String,
    content_len: i32,
    data_type: String,
    created_at: i32,
    last_used_at: i32,
    use_count: i32
}

pub fn create_database(app: &AppHandle) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = app.path().app_data_dir()?.join("clipboard_history.db");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(path)?;

    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;

    // schema
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            short_preview TEXT NOT NULL,
            full_content TEXT NOT NULL,
            content_hash TEXT NOT NULL UNIQUE,
            full_content_len INTEGER NOT NULL,
            data_type TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            last_used_at INTEGER NOT NULL DEFAULT (unixepoch()),
            use_count INTEGER NOT NULL DEFAULT 1
        );
        "#
    )?;

    app.manage(DbState(Mutex::new(conn)));

    Ok(())
}

pub fn add_clipboard_item(conn: &Connection, text: &str) -> Result<i64, rusqlite::Error> {
    let char_count = text.chars().count();
    let preview: String = text.chars().take(80).collect();
    let text_size = char_count as i32;
    let hash = blake3::hash(text.as_bytes()).to_hex().to_string();

    println!("{}", hash);

    let id: i64 = conn.query_row(
        r#"
        INSERT INTO clipboard (
          short_preview, full_content, full_content_len, data_type, content_hash
        )
        VALUES (
          ?1, ?2, ?3, ?4, ?5
        )
        ON CONFLICT(content_hash) DO UPDATE SET
          last_used_at = unixepoch(),
          use_count = clipboard.use_count + 1,
          short_preview = excluded.short_preview,
          full_content_len = excluded.full_content_len
        RETURNING id;
        "#,
        params![preview, text, text_size, "UNICODE", hash], // TODO: Add other types too!
        |row| row.get(0)
    )?;

    //println!("Added clipboard with id {id}, data {text}");

    Ok(id)
}

pub fn load_clipboard_preview_list(conn: &Connection, start_idx: i32) -> Result<Vec<PreviewData>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, short_preview, created_at, last_used_at, use_count FROM clipboard WHERE id >= ?1 ORDER BY last_used_at DESC")?;
    let rows = stmt.query_map([start_idx], |row| {
        Ok(PreviewData {
            id: row.get(0)?,
            preview: row.get(1)?,
            created_at: row.get(2)?,
            last_used_at: row.get(3)?,
            use_count: row.get(4)?,
        })
    })?;

    let mut items = Vec::new();
    for item in rows {
        items.push(item?);
    }
    Ok(items)
}

pub fn load_clipboard_item(conn: &Connection, id: i32) -> Result<ClipboardData, rusqlite::Error> {
    conn.query_row(
        "SELECT full_content, full_content_len, data_type, created_at, last_used_at, use_count FROM clipboard WHERE id = ?1",
        [id],
        |row| {
            Ok(ClipboardData {
                id,
                content: row.get(0)?,
                content_len: row.get(1)?,
                data_type: row.get(2)?,
                created_at: row.get(3)?,
                last_used_at: row.get(4)?,
                use_count: row.get(5)?,
            })
        },
    )
}

pub fn load_clipboard_preview(conn: &Connection, id: i32) -> Result<PreviewData, rusqlite::Error> {
    conn.query_row(
        "SELECT id, short_preview, created_at, last_used_at, use_count FROM clipboard WHERE id = ?1",
        [id],
        |row| {
            Ok(PreviewData {
                id: row.get(0)?,
                preview: row.get(1)?,
                created_at: row.get(2)?,
                last_used_at: row.get(3)?,
                use_count: row.get(4)?,
            })
        },
    )
}

pub fn get_db_size(conn: &Connection) -> Result<i32, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM clipboard;", [], |row| row.get(0))
}