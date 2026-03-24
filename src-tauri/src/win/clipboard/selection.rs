use std::io::Error;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::DWORD_BLOB;
use windows::Win32::UI::Controls::EM_GETSEL;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetGUIThreadInfo, GetWindowTextLengthW, GetWindowTextW, SendMessageW, GUITHREADINFO};

fn get_selection() -> Option<String> {
    unsafe {
        let mut gti: GUITHREADINFO = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(0, &mut gti).is_ok() {
            let hwnd = gti.hwndFocus;
            if hwnd.0.is_null() || hwnd.is_invalid() {
                return None;
            }
            hwnd.
        }
    }
}

unsafe fn get_selection_standard(hwnd: HWND) -> Option<String> {
    let mut start: u32 = 0;
    let mut end: u32 = 0;
    SendMessageW(
        hwnd, EM_GETSEL,
        Option::from(WPARAM(&mut start as *mut _ as usize)),
        Option::from(LPARAM(&mut end as *mut _ as isize))
    );
    if start == end { None };
    let len = GetWindowTextLengthW(hwnd);
    let mut buffer = vec![0u16; (len + 1) as usize];
    GetWindowTextW(hwnd, buffer.as_mut_ptr() as _, &mut start);
}