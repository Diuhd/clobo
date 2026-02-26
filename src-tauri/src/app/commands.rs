use crate::app::{cache, DbState};
use crate::app::cache::{ClipboardData, PreviewData};

#[tauri::command]
#[specta::specta]
pub fn get_clipboard_size(state: tauri::State<'_, DbState>) -> Result<i32, String> {
    let conn = state.0.lock().unwrap();
    let db_size = cache::get_db_size(&conn).map_err(|e| e.to_string())?;
    Ok(db_size)
}

#[tauri::command]
#[specta::specta]
pub fn load_data_preview_list(state: tauri::State<'_, DbState>, start_idx: i32) -> Result<Vec<PreviewData>, String> {
    let conn = state.0.lock().unwrap();
    let items = cache::load_clipboard_preview_list(&conn, start_idx)
        .map_err(|e| e.to_string())?;
    Ok(items)
}

#[tauri::command]
#[specta::specta]
pub fn load_data(state: tauri::State<'_, DbState>, id: i32) -> Result<ClipboardData, String> {
    let conn = state.0.lock().unwrap();
    let item = cache::load_clipboard_item(&conn, id).map_err(|e| e.to_string())?;
    Ok(item)
}

#[tauri::command]
#[specta::specta]
pub fn load_preview(state: tauri::State<'_, DbState>, id: i32) -> Result<PreviewData, String> {
    let conn = state.0.lock().unwrap();
    let item = cache::load_clipboard_preview(&conn, id).map_err(|e| e.to_string())?;
    Ok(item)
}
