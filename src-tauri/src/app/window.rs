use tauri::{AppHandle, Manager, PhysicalPosition};

pub fn show_near_point(app: &AppHandle, x: i32, y: i32) -> tauri::Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tauri::Error::WindowNotFound)?;

    window.set_position(PhysicalPosition::new(x, y))?;
    window.show()?;
    window.set_focus()?;

    Ok(())
}

pub fn hide(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    window.hide()?;
    Ok(())
}
