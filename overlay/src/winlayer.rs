//! Win32 layered-window helper.
//!
//! Gives the overlay a uniformly translucent window **without** relying on the
//! GPU surface exposing an alpha mode — DX12 reports `AlphaMode::Ignore` and
//! therefore drops per-pixel alpha, whereas `SetLayeredWindowAttributes`
//! composites at the DWM level and works with any rendering backend.

/// Applies `WS_EX_LAYERED` + uniform alpha to the window identified by `hwnd`
/// (the raw id reported by `iced::window::raw_id`).
///
/// Returns `true` when the window system accepted the attributes.
#[cfg(target_os = "windows")]
pub fn apply(hwnd: u64, alpha: f32) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, LWA_ALPHA, SetLayeredWindowAttributes, SetWindowLongPtrW, WS_EX_LAYERED,
    };

    if hwnd == 0 {
        return false;
    }

    let hwnd = hwnd as *mut core::ffi::c_void;
    let alpha_byte = (alpha.clamp(0.05, 1.0) * 255.0).round() as u8;

    // SAFETY: `hwnd` comes from iced/winit for our own window; the calls are the
    // documented Win32 pattern for enabling uniform-alpha layered windows.
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if ex_style & (WS_EX_LAYERED as isize) == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | (WS_EX_LAYERED as isize));
        }
        SetLayeredWindowAttributes(hwnd, 0, alpha_byte, LWA_ALPHA) != 0
    }
}

/// No-op on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
pub fn apply(_hwnd: u64, _alpha: f32) -> bool {
    false
}

/// Enables or disables mouse pass-through (`WS_EX_TRANSPARENT`).
///
/// With it enabled the window is skipped by hit-testing, so every click lands
/// on whatever is underneath — this is what "pin mode" uses so the overlay can
/// be watched without ever being grabbed. `WS_EX_NOACTIVATE` is applied too, so
/// the window cannot steal focus.
#[cfg(target_os = "windows")]
pub fn set_click_through(hwnd: u64, enabled: bool) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
    };

    if hwnd == 0 {
        return false;
    }

    let hwnd = hwnd as *mut core::ffi::c_void;
    let flags = (WS_EX_TRANSPARENT | WS_EX_NOACTIVATE) as isize;

    // SAFETY: `hwnd` is our own window and these are the documented extended
    // styles for a click-through overlay.
    unsafe {
        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if enabled {
            ex_style |= flags;
        } else {
            ex_style &= !flags;
        }
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
    }
    true
}

/// No-op on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
pub fn set_click_through(_hwnd: u64, _enabled: bool) -> bool {
    false
}

/// Whether mouse pass-through can actually be applied here.
///
/// Only Windows is implemented; macOS (`setIgnoresMouseEvents`) and X11 (input
/// shape) would each need their own branch, and Wayland has no protocol for it
/// at all. Where it is unavailable, pin mode still locks the position and the
/// right-click menu remains reachable.
#[cfg(target_os = "windows")]
pub const fn click_through_supported() -> bool {
    true
}

/// See the Windows variant.
#[cfg(not(target_os = "windows"))]
pub const fn click_through_supported() -> bool {
    false
}
