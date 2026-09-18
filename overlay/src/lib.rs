//! WattSeal always-on-top overlay widget.
//!
//! This crate is a self-contained library that the **main binary** runs in its
//! own process via `WattSeal.exe --overlay` (started from the tray or from the
//! main window's footer). It never blocks the main application thread and owns
//! its own state, theme and config.

pub mod app;
pub mod config;
pub mod message;
pub mod theme;
pub mod translations;
pub mod winlayer;

pub use winlayer::click_through_supported;

/// Runs the overlay window.
///
/// Transparency is delegated to the platform: [`config::Transparency::Auto`]
/// resolves to a Win32 layered window on Windows (where the GPU surface exposes
/// no alpha-capable composite mode) and to per-pixel surface alpha elsewhere.
pub fn run() -> iced::Result {
    // Match the collector and the dashboard: resolve the database — and the
    // config file written next to it — against the executable's directory
    // instead of whatever directory this process happened to start in.
    if let Err(err) = common::set_current_dir_to_exe_dir() {
        log::warn!("overlay: could not switch to the executable directory: {err}");
    }
    init_logging();
    app::run()
}

/// Whether the overlay is currently asked to be running.
///
/// The overlay and its callers talk through the config file rather than through
/// IPC, so this is the read side of that channel.
pub fn is_requested() -> bool {
    config::OverlayConfig::load()
        .map(|config| config.overlay_requested)
        .unwrap_or(false)
}

/// Asks the overlay to open, clearing pin mode with it: a pinned overlay
/// ignores the mouse, so it must not come back locked once the user has just
/// asked for it.
///
/// Callers say what they want instead of editing the file, which keeps the
/// layout of that file in one place — the tray, the dashboard and the overlay
/// itself all go through here.
pub fn request_open() {
    let mut config = config::OverlayConfig::load().unwrap_or_default();

    // Nothing to write when it is already open and unpinned.
    if config.overlay_requested && !config.pin_mode {
        return;
    }

    config.overlay_requested = true;
    config.pin_mode = false;
    config.save();
}

/// Asks the overlay to close.
pub fn request_close() {
    let mut config = config::OverlayConfig::load().unwrap_or_default();

    if !config.overlay_requested {
        return;
    }

    config.overlay_requested = false;
    config.save();
}

/// Flips "pin mode" and returns the new state.
///
/// The overlay polls the file, so this needs no IPC. It is also how a pinned,
/// click-through overlay is released, since that window ignores the mouse.
pub fn toggle_pin() -> bool {
    let mut config = config::OverlayConfig::load().unwrap_or_default();
    config.pin_mode = !config.pin_mode;
    config.save();
    config.pin_mode
}

/// Where the opt-in log is written: `overlay.log` next to the executable.
static LOG_PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Installs a tiny file logger, but only when `WATTSEAL_OVERLAY_LOG` is set.
///
/// The renderer diagnostics (selected adapter, surface format and **alpha mode**)
/// are the only way to explain why a window renders opaque on a machine we cannot
/// inspect, which is the kind of report an overlay attracts. It is opt-in so a
/// normal run never writes to the user's disk.
fn init_logging() {
    use std::io::Write;

    if std::env::var_os("WATTSEAL_OVERLAY_LOG").is_none() {
        return;
    }
    LOG_PATH
        .set(config::OverlayConfig::path().with_file_name("overlay.log"))
        .ok();

    struct FileLogger;

    impl log::Log for FileLogger {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            let Some(path) = LOG_PATH.get() else {
                return;
            };
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "[{}] {}", record.level(), record.args());
            }
        }

        fn flush(&self) {}
    }

    static LOGGER: FileLogger = FileLogger;

    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}
