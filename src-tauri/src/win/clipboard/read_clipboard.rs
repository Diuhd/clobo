use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::{CF_UNICODETEXT};

unsafe fn wait_open_clipboard(timeout_ms: u64) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms) {
        if !OpenClipboard(None).is_err() { return true; }
        thread::sleep(Duration::from_millis(5));
    }
    false
}


pub fn read_text() -> Option<String> {
    unsafe {
        wait_open_clipboard(1000);

        let handle = GetClipboardData(CF_UNICODETEXT.0 as u32);

        if handle.is_err() {
            let _ = CloseClipboard();
            return None;
        }

        let handle_unwrap = handle.unwrap();

        if handle_unwrap.is_invalid() {
            let _ = CloseClipboard();
            return None;
        }

        let ptr = GlobalLock(HGLOBAL(handle_unwrap.0)) as *mut u16;
        if ptr.is_null() {
            let _ = CloseClipboard();
            return None;
        }

        // CF_UNICODETEXT is a null-terminated UTF-16 string
        let mut len = 0usize;
        let mut p = ptr as *const u16;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }

        let slice = core::slice::from_raw_parts(ptr as *const u16, len);
        let s = String::from_utf16_lossy(slice);

        let _ = GlobalUnlock(HGLOBAL(handle_unwrap.0));
        let _ = CloseClipboard();

        if s.is_empty() {
            None
        } else { Some(s) }
    }
}
