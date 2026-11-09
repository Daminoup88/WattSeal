#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
use pelite::{FileMap, PeFile};

/// Load the Windows PE `FileDescription` from `path`, if available.
pub fn load_process_name(_path: &str) -> Option<String> {
    #[cfg(windows)]
    {
        let fd = std::panic::catch_unwind(|| pe_file_description(Path::new(_path))).unwrap_or_else(|_| {
            crate::clog!("⚠ PE metadata parsing panicked for path: {_path}");
            None
        });
        if let Some(fd) = fd {
            if fd == "" { None } else { Some(fd) }
        } else {
            None
        }
    }

    #[cfg(not(windows))]
    {
        None
    }
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
