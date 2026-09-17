//! Native Win32 context menu for the overlay.
//!
//! A real `TrackPopupMenu` popup is used rather than an in-window overlay
//! because the overlay window can be tiny (the horizontal layout is only a few
//! dozen pixels tall): a native menu is drawn by the OS at the cursor, can never
//! be clipped by the window, and behaves like every other Windows app.

/// Nothing selected (or the menu was dismissed).
pub const NONE: u8 = 0;
/// The "Settings" entry.
pub const SETTINGS: u8 = 1;
/// The "Quit" entry.
pub const QUIT: u8 = 2;

/// Shows the context menu at the current cursor position and blocks until it is
/// dismissed, returning the chosen command ([`NONE`] when cancelled).
///
/// `TrackPopupMenu` runs a *modal message loop* and menu input is delivered
/// through the message queue of the thread that **owns** `hwnd`. It must
/// therefore be called from the event loop's own thread — hosting it on a
/// worker thread makes the popup appear and immediately vanish.
pub fn show(hwnd: u64) -> u8 {
    let choice = track(hwnd);
    log::info!("winmenu: hwnd={hwnd} choice={choice}");
    choice
}

#[cfg(target_os = "windows")]
fn track(hwnd: u64) -> u8 {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_SEPARATOR, MF_STRING, PostMessageW,
        SetForegroundWindow, TPM_LEFTALIGN, TPM_RETURNCMD, TrackPopupMenu, WM_NULL,
    };

    let hwnd = hwnd as *mut core::ffi::c_void;
    if hwnd.is_null() {
        log::info!("winmenu: null hwnd, skipping");
        return NONE;
    }

    // SAFETY: this is the documented Win32 popup-menu recipe (`CreatePopupMenu`
    // -> `AppendMenuW` -> `TrackPopupMenu` -> `DestroyMenu`) on our own window.
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            log::info!("winmenu: CreatePopupMenu failed");
            return NONE;
        }

        let settings: Vec<u16> = "Settings\0".encode_utf16().collect();
        let quit: Vec<u16> = "Quit\0".encode_utf16().collect();
        AppendMenuW(menu, MF_STRING, SETTINGS as usize, settings.as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(menu, MF_STRING, QUIT as usize, quit.as_ptr());

        let mut point = POINT { x: 0, y: 0 };
        GetCursorPos(&mut point);

        // Required so the popup is dismissed by clicking elsewhere.
        SetForegroundWindow(hwnd);

        let selected = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_RETURNCMD,
            point.x,
            point.y,
            0,
            hwnd,
            std::ptr::null(),
        );

        // Documented workaround: the menu can linger without a posted message.
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);

        log::info!(
            "winmenu: cursor=({}, {}) selected={selected}",
            point.x,
            point.y
        );

        match selected as u8 {
            SETTINGS => SETTINGS,
            QUIT => QUIT,
            _ => NONE,
        }
    }
}

/// No-op fallback for other platforms.
#[cfg(not(target_os = "windows"))]
fn track(_hwnd: u64) -> u8 {
    NONE
}
