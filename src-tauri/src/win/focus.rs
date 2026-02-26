use tauri::{Manager, WindowEvent};

pub fn setup_focus_hide(app: &tauri::App) {
    let window = app.get_webview_window("main").unwrap();
    let w = window.clone();

    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Focused(false)) {
            let _ = w.hide();
        }
    });
}