pub mod listener;
pub mod read_clipboard;

pub fn start_listener_thread() {
    std::thread::spawn(move || {
        listener::run_clipboard_window_loop();
    });
}
