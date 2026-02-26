use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{AddClipboardFormatListener, RemoveClipboardFormatListener};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::app::event::SystemEvent;

const CLASS_NAME: PCWSTR = w!("CloboClipboardListener");

pub fn run_clipboard_window_loop() {
    unsafe {
        let hinstance = HINSTANCE::from(GetModuleHandleW(None).unwrap());

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };

        RegisterClassW(&wc);

        // Create a message-only window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0), CLASS_NAME, w!(""),
            WS_OVERLAPPED,
            0, 0, 0, 0,
            Option::from(HWND_MESSAGE), None, Option::from(hinstance), None
        ).expect("Cannot create window!");

        if hwnd.0.is_null() { return; }

        let _ = AddClipboardFormatListener(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            if let Some(tx) = crate::app::event::EVENT.get() {
                tx.send(SystemEvent::ClipboardUpdate).expect("Failed to send SystemEvent::ClipboardUpdate");
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            let _ = RemoveClipboardFormatListener(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
