//! WattSeal always-on-top overlay widget.
//!
//! This crate is a self-contained library that the **main binary** runs in its
//! own process via `WattSeal.exe --overlay` (launched from the tray). It never
//! blocks the main application thread and owns its own state, theme and config.

pub mod app;
pub mod config;
pub mod message;
pub mod theme;
pub mod winlayer;

/// Runs the overlay window.
///
/// On Windows, per-pixel window transparency only composites correctly when the
/// GPU surface exposes a pre/post-multiplied alpha mode — provided by Vulkan
/// but usually not by DX12 (`AlphaMode::Ignore`). When the Vulkan loader is
/// present we therefore prefer the Vulkan backend so the default `Auto`
/// transparency mode stays translucent; the `Layered` mode works on any backend
/// via a Win32 layered window.
pub fn run() -> iced::Result {
    init_logging();

    // The layered path is GPU-independent, so only steer towards Vulkan when
    // the per-pixel surface path is selected and a Vulkan loader exists.
    #[cfg(target_os = "windows")]
    if !config::OverlayConfig::load()
        .unwrap_or_default()
        .transparency
        .uses_layered()
        && std::env::var_os("WGPU_BACKEND").is_none()
        && vulkan_loader_present()
    {
        // SAFETY: called once at startup, before the event loop spawns threads.
        unsafe { std::env::set_var("WGPU_BACKEND", "vulkan") };
    }

    app::run()
}

/// Installs a tiny file logger so the renderer diagnostics — selected adapter,
/// surface format and **alpha mode** — end up in `overlay.log`. This is the key
/// data needed to explain why a window may still render opaque.
fn init_logging() {
    use std::io::Write;

    struct FileLogger;

    impl log::Log for FileLogger {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("overlay.log")
            {
                let _ = writeln!(file, "[{}] {}", record.level(), record.args());
            }
        }

        fn flush(&self) {}
    }

    static LOGGER: FileLogger = FileLogger;

    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}

/// Heuristic: the Vulkan loader (`vulkan-1.dll`) is installed in System32.
#[cfg(target_os = "windows")]
fn vulkan_loader_present() -> bool {
    std::env::var("SystemRoot")
        .map(|root| std::path::Path::new(&root).join("System32").join("vulkan-1.dll").exists())
        .unwrap_or(false)
}
