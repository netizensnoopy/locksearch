/// Windows-specific platform code for frameless window with resize support.
///
/// Strategy: Start with `decorations: true` (gives native WS_THICKFRAME resize
/// borders), then strip `WS_CAPTION` to remove the title bar while keeping
/// resize borders functional. This is the proven approach used by Chrome/Electron.

#[cfg(target_os = "windows")]
pub fn setup_frameless_resize() {
    use std::thread;
    use std::time::Duration;

    thread::spawn(|| {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;

            // Retry to handle timing — winit may not have created the window yet
            for attempt in 0..15 {
                thread::sleep(Duration::from_millis(if attempt == 0 { 400 } else { 200 }));

                // Find our window by its title
                let title: Vec<u16> = "LockSearch\0".encode_utf16().collect();
                let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());

                if hwnd.is_null() {
                    continue;
                }

                let style = GetWindowLongW(hwnd, GWL_STYLE);

                // If WS_CAPTION is already removed, we're done
                if (style & WS_CAPTION as i32) == 0 {
                    return;
                }

                // Remove WS_CAPTION (title bar + border chrome) but keep
                // WS_THICKFRAME (resize borders), WS_MINIMIZEBOX, WS_MAXIMIZEBOX
                let new_style = style & !(WS_CAPTION as i32);
                SetWindowLongW(hwnd, GWL_STYLE, new_style);

                // Force Windows to recalculate the frame
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );

                // Verify style change stuck
                let check = GetWindowLongW(hwnd, GWL_STYLE);
                if (check & WS_CAPTION as i32) == 0 {
                    return; // Success
                }
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub fn setup_frameless_resize() {
    // No-op on non-Windows platforms
}
