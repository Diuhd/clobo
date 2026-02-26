use windows::core::{Interface, BOOL};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, SAFEARRAY,  CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED,
};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Accessibility::{
    AccessibleObjectFromWindow, IAccessible, IUIAutomation, IUIAutomationElement,
    IUIAutomationTextPattern2, IUIAutomationTextRange, UIA_TextPattern2Id, CUIAutomation8
};
use windows::Win32::UI::WindowsAndMessaging::{OBJID_CARET, CHILDID_SELF};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;

use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetLBound, SafeArrayGetUBound,
    SafeArrayUnaccessData
};

#[cfg(windows)]
pub fn get_caret_rect() -> RECT {
    if let Some(rect) = unsafe { get_caret_uia() } {
        //println!("Returning 1");
        return rect;
    }

    if let Some(rect) = unsafe { get_caret_guithreadinfo() } {
        //println!("Returning 2");
        return rect;
    }

    if let Some(rect) = get_caret_msaa() {
        //println!("Returning 3");
        return rect;
    }

    RECT::default()
}

#[cfg(windows)]
fn get_caret_msaa() -> Option<RECT> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            let _ = CoUninitialize();
            return None;
        }

        let mut pv_object: *mut std::ffi::c_void = std::ptr::null_mut();
        if AccessibleObjectFromWindow(
            hwnd,
            OBJID_CARET.0 as u32,
            &IAccessible::IID,
            &mut pv_object as *mut _ as *mut _,
        )
            .is_err()
        {
            let _ = CoUninitialize();
            return None;
        }

        // SAFETY: pv_object points to an IAccessible COM interface.
        let accessible: IAccessible = IAccessible::from_raw(pv_object);

        // Prepare variables to receive the caret bounds.
        let mut left: i32 = 0;
        let mut top: i32 = 0;
        let mut width: i32 = 0;
        let mut height: i32 = 0;

        // Create a VARIANT representing CHILDID_SELF.
        let mut var_caret: VARIANT = std::mem::zeroed();
        (*var_caret.Anonymous.Anonymous).vt = VT_I4;
        (*var_caret.Anonymous.Anonymous).Anonymous.lVal = CHILDID_SELF as i32;

        // Invoke accLocation to obtain the caret location.
        let hr = accessible.accLocation(&mut left, &mut top, &mut width, &mut height, &var_caret);

        let _ = CoUninitialize();

        if hr.is_ok() && width > 0 && height > 0 {
            Some(RECT {
                left,
                top,
                right: left + width,
                bottom: top + height,
            })
        } else {
            None
        }
    }
}

/// Attempt to obtain the caret bounding rectangle using the modern UI
/// Automation API.  UI Automation is much more reliable than MSAA and
/// works in most modern applications (including browsers and UWP apps).
/// Returns `Some(RECT)` if successful or `None` if the caret could not
/// be located.
unsafe fn get_caret_uia() -> Option<RECT> {
    // Initialize COM for multithreaded apartments.
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

    // Create an instance of the CUIAutomation8 object.
    let automation: IUIAutomation = match CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) {
        Ok(a) => a,
        Err(_) => {
            let _ = CoUninitialize();
            return None;
        }
    };

    // Retrieve the currently focused element.
    let element: IUIAutomationElement = match automation.GetFocusedElement() {
        Ok(e) => e,
        Err(_) => {
            let _ = CoUninitialize();
            return None;
        }
    };

    // Request the IUIAutomationTextPattern2 interface from the element.
    let text_pattern2: IUIAutomationTextPattern2 =
        match element.GetCurrentPatternAs::<IUIAutomationTextPattern2>(UIA_TextPattern2Id) {
            Ok(p) => p,
            Err(_) => {
                let _ = CoUninitialize();
                return None;
            }
        };

    // Retrieve the caret range.  The boolean returned via is_active
    // indicates whether the control has keyboard focus.
    let mut is_active: BOOL = BOOL(0);
    let caret_range: IUIAutomationTextRange = match text_pattern2.GetCaretRange(&mut is_active) {
        Ok(r) => r,
        Err(_) => {
            let _ = CoUninitialize();
            return None;
        }
    };

    // Query the bounding rectangles for the caret range.  UI Automation
    // returns a SAFEARRAY of doubles.
    let psa: *mut SAFEARRAY = match caret_range.GetBoundingRectangles() {
        Ok(a) => a,
        Err(_) => {
            let _ = CoUninitialize();
            return None;
        }
    };
    if psa.is_null() {
        let _ = CoUninitialize();
        return None;
    }

    // Determine the bounds of the SAFEARRAY.
    let get_bounds = || -> windows::core::Result<(i32, i32)> {
        let lbound = SafeArrayGetLBound(psa, 1)?;
        let ubound = SafeArrayGetUBound(psa, 1)?;
        Ok((lbound, ubound))
    };

    let bounds = match get_bounds() {
        Ok(bounds) => bounds,
        Err(_) => {
            let _ = SafeArrayDestroy(psa);
            CoUninitialize();
            return None;
        }
    };

    // Compute the element count.
    let count = bounds.1 - bounds.0 + 1;
    if count < 4 {
        let _ = SafeArrayDestroy(psa);
        let _ = CoUninitialize();
        return None;
    }

    // Access the data in the SAFEARRAY.
    let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    if SafeArrayAccessData(psa, &mut data_ptr).is_err() {
        let _ = SafeArrayDestroy(psa);
        let _ = CoUninitialize();
        return None;
    }
    let rects_slice = std::slice::from_raw_parts(data_ptr as *const f64, count as usize);

    let left = rects_slice[0] as i32;
    let top = rects_slice[1] as i32;
    let width = rects_slice[2] as i32;
    let height = rects_slice[3] as i32;

    // Deconstruction
    let _ = SafeArrayUnaccessData(psa);
    let _ = SafeArrayDestroy(psa);

    let _ = CoUninitialize();

    if width <= 0 || height <= 0 {
        return None;
    }
    Some(RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    })
}

/// Use the classic `GetGUIThreadInfo` API to retrieve the caret
/// rectangle.  This method works for applications that use the system
/// caret (most Win32 edit controls) but does not work with modern
/// applications that draw their own caret.  The returned rectangle is
/// converted from client coordinates to screen coordinates.
unsafe fn get_caret_guithreadinfo() -> Option<RECT> {
    let hwnd = GetForegroundWindow();
    if hwnd.0.is_null() {
        return None;
    }

    let thread_id = GetWindowThreadProcessId(hwnd, None);
    if thread_id == 0 {
        return None;
    }

    let mut gui_info: GUITHREADINFO = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };

    if GetGUIThreadInfo(thread_id, &mut gui_info).is_err() || gui_info.hwndCaret.0.is_null() {
        return None;
    }

    let mut top_left = POINT {
        x: gui_info.rcCaret.left,
        y: gui_info.rcCaret.top,
    };
    let mut bottom_right = POINT {
        x: gui_info.rcCaret.right,
        y: gui_info.rcCaret.bottom,
    };

    if !ClientToScreen(gui_info.hwndCaret, &mut top_left).as_bool()
        || !ClientToScreen(gui_info.hwndCaret, &mut bottom_right).as_bool()
    {
        return None;
    }

    Some(RECT {
        left: top_left.x,
        top: top_left.y,
        right: bottom_right.x,
        bottom: bottom_right.y,
    })
}
