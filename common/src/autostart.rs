//! Registers/unregisters WattSeal to launch automatically when the user logs in.
//!
//! Windows is backed by the per-user `Run` registry key (no admin rights required).
//! Other platforms are not yet supported and report the feature as unavailable.

#[cfg(target_os = "windows")]
mod platform {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};

    const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "WattSeal";

    /// Returns `true` if a startup entry for WattSeal exists in the registry.
    pub fn is_enabled() -> bool {
        let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(RUN_KEY_PATH) else {
            return false;
        };
        hkcu.get_value::<String, _>(VALUE_NAME).is_ok()
    }

    /// Adds or removes the WattSeal startup registry entry.
    ///
    /// When enabling, the entry launches the current executable in background
    /// mode (tray icon only), matching how the app behaves when auto-started.
    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(RUN_KEY_PATH).map_err(|e| e.to_string())?;

        if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let command = format!("\"{}\" --background", exe.display());
            key.set_value(VALUE_NAME, &command).map_err(|e| e.to_string())?;
            crate::clog!("\u{2713} Registered launch-on-startup entry {RUN_KEY_PATH}\\{VALUE_NAME} = {command}");
            Ok(())
        } else {
            match key.delete_value(VALUE_NAME) {
                Ok(()) => {
                    crate::clog!("\u{2713} Removed launch-on-startup entry {RUN_KEY_PATH}\\{VALUE_NAME}");
                    Ok(())
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    crate::clog!("\u{2713} Launch-on-startup entry {RUN_KEY_PATH}\\{VALUE_NAME} already absent");
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    pub fn is_enabled() -> bool {
        false
    }

    pub fn set_enabled(_enabled: bool) -> Result<(), String> {
        Err("Launch on startup is currently only supported on Windows".to_string())
    }
}

pub use platform::{is_enabled, set_enabled};

/// Returns `true` if launch-on-startup is supported on this platform.
pub const fn is_supported() -> bool {
    cfg!(target_os = "windows")
}
