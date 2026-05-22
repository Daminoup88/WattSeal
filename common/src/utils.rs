#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
use pelite::{FileMap, PeFile};

use crate::database::types::IconData;

/// Load an icon and a friendly name from `path`.
///
/// Icons are retrieved cross-platform via [`file_icon_provider`].
/// On Windows the friendly name is read from the PE version info using [`pelite`].
///
/// Parameters:
/// - `path`: Path string to an executable or other file.
///
/// Returns: `(Option<IconData>, Option<String>)` — the icon as raw RGBA pixel
/// data (if available) and the friendly name (if available).
pub fn load_icon_and_name(path: &str) -> (Option<IconData>, Option<String>) {
    let icon = std::panic::catch_unwind(|| load_icon(path)).unwrap_or_else(|_| {
        crate::clog!("⚠ Icon extraction panicked for path: {path}");
        None
    });

    #[cfg(windows)]
    let name = {
        let fd = std::panic::catch_unwind(|| pe_file_description(Path::new(path))).unwrap_or_else(|_| {
            crate::clog!("⚠ PE metadata parsing panicked for path: {path}");
            None
        });
        if let Some(fd) = fd {
            if fd == "" { None } else { Some(fd) }
        } else {
            None
        }
    };
    #[cfg(not(windows))]
    let name: Option<String> = None;

    (icon, name)
}

/// Retrieve the system icon for the file at `path` on any platform using [`file_icon_provider`].
fn load_icon(path: &str) -> Option<IconData> {
    let icon = file_icon_provider::get_file_icon(path, 32).ok()?;
    Some(IconData {
        width: icon.width,
        height: icon.height,
        pixels: icon.pixels,
    })
}

/// Windows-only: extract the `FileDescription` from the PE version-info resource of the file at `path` using [`pelite`].
#[cfg(windows)]
fn pe_file_description(path: &Path) -> Option<String> {
    // Fast skip for non-PE files to avoid unnecessary parser work and edge-case crashes.
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    let is_pe_like = matches!(ext.as_deref(), Some("exe" | "dll" | "scr" | "cpl" | "ocx"));
    if !is_pe_like {
        return None;
    }

    let map = FileMap::open(path).ok()?;
    let file = PeFile::from_bytes(&map).ok()?;
    let resources = file.resources().ok()?;
    let vi = resources.version_info().ok()?;
    let lang = vi.translation().first().copied()?;
    let value = vi.value(lang, "FileDescription");
    if value.is_none() {
        crate::clog!("⚠ Missing FileDescription in PE metadata for path: {}", path.display());
    }
    value
}

/// Converts a byte count to megabytes.
pub fn bytes_to_mb(bytes: f64) -> f64 {
    bytes / (2 << 20) as f64
}

/// Set current working directory to the executable's parent directory.
pub fn set_current_dir_to_exe_dir() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let Some(exe_dir) = exe.parent() else {
        return Ok(());
    };
    std::env::set_current_dir(exe_dir)
}
