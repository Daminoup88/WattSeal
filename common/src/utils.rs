use std::path::Path;

use pelite::{FileMap, PeFile};

/// Load an icon and a friendly name from `path`.
///
/// On Windows it attempts to read the PE file in `windows_icon_and_name` using the [`pelite`] crate.
///
/// On non-Windows targets this function returns `(None, None)`.
///
/// Parameters:
/// - `path`: [`Path`] to an executable, DLL or other PE-containing file.
///
/// Returns: `(Option<Vec<u8>>, Option<String>)` where the first element is the icon in ICO format (if found) and the second element is the friendly name (if found).
pub fn load_icon_and_name(path: &str) -> (Option<Vec<u8>>, Option<String>) {
    #[cfg(windows)]
    {
        windows_icon_and_name(Path::new(path))
    }
    #[cfg(not(windows))]
    {
        (None, None)
    }
}

/// Windows-specific function that loads an icon and a friendly name from a PE file using the [`pelite`] crate.
fn windows_icon_and_name(path: &Path) -> (Option<Vec<u8>>, Option<String>) {
    // Try to memory-map the file (fast, avoids copying the whole file)
    if let Ok(map) = FileMap::open(path) {
        // Parse as a PE file
        if let Ok(file) = PeFile::from_bytes(&map) {
            // Access the resource directory
            if let Ok(resources) = file.resources() {
                // Read friendly name from VERSIONINFO -> "FileDescription" (first translation)
                let file_description = resources.version_info().ok().and_then(|vi| {
                    let lang = vi.translation().first().copied()?;
                    vi.value(lang, "FileDescription")
                });

                // Find the first icon group and serialize it into bytes (ICO-format group)
                for res in resources.icons() {
                    if let Ok((_, group)) = res {
                        let mut contents = Vec::new();
                        if group.write(&mut contents).is_ok() {
                            return (Some(contents), file_description);
                        }
                    }
                }
                return (None, file_description);
            }
        }
    }
    (None, None)
}

pub fn bytes_to_mb(bytes: f64) -> f64 {
    bytes / (2 << 20) as f64
}
