use std::sync::mpsc::Sender;
use std::sync::{mpsc, OnceLock};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager};
use crate::app::{cache, DbState};

#[derive(serde::Serialize, serde::Deserialize, Type)]
pub enum SystemEvent {
    ClipboardUpdate
}

pub static EVENT: OnceLock<Sender<SystemEvent>> = OnceLock::new();

pub fn init(app: &AppHandle) {
    let (tx, rx) = mpsc::channel();
    EVENT.set(tx).unwrap();
    let handle = app.clone();
    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            match event {
                SystemEvent::ClipboardUpdate => {
                    if let Some(clipboard_data) = crate::win::clipboard::read_clipboard::read_text() {
                        let id = cache::add_clipboard_item(&*handle.state::<DbState>().0.lock().unwrap(), &clipboard_data).expect("Failed to add clipboard item");
                        handle.emit("clipboard_update", id).expect("Failed to emit clipboard item");
                        // I know. It's faster to make a PreviewData object after truncating the string and directly pushing it to the frontend.
                        // I'm using this approach to sync the db with the window at all cases.
                    } else {
                        println!("read_text = None (first update likely busy / not text yet)");
                    }
                }
            }
        }
    });
}