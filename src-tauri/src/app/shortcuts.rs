use tauri::{AppHandle};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register(app: &AppHandle) -> tauri::Result<()> {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Backquote);

    let app_handle = app.clone();
    let _ = app_handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, _shortcut, _event| {
                if _shortcut != &shortcut { return; }
                if matches!(_event.state(), ShortcutState::Pressed) {
                    let caret_rect = crate::win::caret::get_caret_rect();
                    let x = caret_rect.left;
                    let y = caret_rect.bottom + 8;

                    let _ = crate::app::window::show_near_point(&_app.clone(), x, y);
                }
            })
            .build()
    );
    let _ = app.global_shortcut().register(shortcut);

    Ok(())
}
