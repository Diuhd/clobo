pub mod commands;
pub mod shortcuts;
pub mod window;
pub mod event;
pub mod cache;

use std::sync::Mutex;
use rusqlite::Connection;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
use crate::app::event::SystemEvent;
use crate::win;

pub struct DbState(pub Mutex<Connection>);
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::load_data_preview_list,
            commands::load_data,
            commands::get_clipboard_size,
            commands::load_preview
        ])
        .typ::<SystemEvent>();

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            win::focus::setup_focus_hide(app);
            cache::create_database(app.handle()).expect("Failed to create database.");
            event::init(app.handle());

            shortcuts::register(app.handle())?;

            win::clipboard::start_listener_thread();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
