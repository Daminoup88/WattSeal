#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    process::{Child, Command},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use collector::CollectorApp;
use common::WINDOW_ICON_BYTES;
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
#[cfg(not(target_os = "linux"))]
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

/// Spawns the UI subprocess if not already running.
fn spawn_ui(ui_child: &Arc<Mutex<Option<Child>>>) -> Result<(), String> {
    let mut guard = ui_child.lock().map_err(|e| {
        let msg = format!("Failed to lock UI child mutex: {}", e);
        common::clog!("✗ {msg}");
        msg
    })?;
    let already_running = guard.as_mut().is_some_and(|c| matches!(c.try_wait(), Ok(None)));
    if already_running {
        return Ok(());
    }
    if let Ok(exe) = std::env::current_exe() {
        match Command::new(exe).arg("--ui").spawn() {
            Ok(child) => {
                *guard = Some(child);
                Ok(())
            }
            Err(e) => {
                let msg = format!("Failed to spawn UI process: {}", e);
                common::clog!("✗ Failed to spawn UI process: {}", e);
                Err(msg)
            }
        }
    } else {
        let msg = "Failed to determine current executable path".to_string();
        common::clog!("✗ {msg}");
        Err(msg)
    }
}

/// Loads the application icon from the embedded PNG for the system tray.
fn load_tray_icon() -> Option<tray_icon::Icon> {
    let img = image::load_from_memory(WINDOW_ICON_BYTES).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).ok()
}

/// Sets up the tray icon menu, event handlers, and builds the tray icon.
/// Returns `Some(TrayIcon)` on success, `None` if icon loading or tray creation fails.
fn setup_tray(ui_child: &Arc<Mutex<Option<Child>>>) -> Option<tray_icon::TrayIcon> {
    let tray_menu = Menu::new();
    let open_ui_i = MenuItem::new("Open UI", true, None);
    let quit_i = MenuItem::new("Quit", true, None);
    let open_ui_id = open_ui_i.id().to_owned();
    let quit_id = quit_i.id().to_owned();

    tray_menu
        .append_items(&[&open_ui_i, &PredefinedMenuItem::separator(), &quit_i])
        .ok();

    let ui_child_menu = Arc::clone(ui_child);
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_ui_id {
            spawn_ui(&ui_child_menu).ok();
        } else if event.id == quit_id {
            if let Ok(mut child_guard) = ui_child_menu.lock() {
                if let Some(c) = child_guard.as_mut() {
                    let _ = c.kill();
                }
            }
            std::process::exit(0);
        }
    }));

    let ui_child_tray = Arc::clone(ui_child);
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::DoubleClick { .. } = event {
            spawn_ui(&ui_child_tray).ok();
        }
    }));

    let icon = load_tray_icon()?;
    TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("WattSeal")
        .with_icon(icon)
        .build()
        .ok()
}

/// Linux: try to initialise GTK, create the tray icon and run the GTK event loop.
/// Returns `true` if the tray was set up and the GTK loop ran (i.e. the app lifecycle
/// was fully handled). Returns `false` if setup failed so the caller can fall back.
#[cfg(target_os = "linux")]
fn run_linux_tray(ui_child: &Arc<Mutex<Option<Child>>>) -> bool {
    if gtk::init().is_err() {
        return false;
    }

    let _tray_icon = match setup_tray(ui_child) {
        Some(t) => t,
        None => return false,
    };

    // Monitor UI child process via GTK periodic callback
    let ui_child_watcher = Arc::clone(ui_child);
    gtk::glib::timeout_add_local(Duration::from_millis(250), move || {
        if let Ok(mut guard) = ui_child_watcher.lock() {
            if let Some(child) = guard.as_mut() {
                if let Ok(Some(status)) = child.try_wait() {
                    let code = status.code().unwrap_or(0);
                    *guard = None;
                    if code == common::EXIT_CODE_SHUTDOWN_ALL {
                        std::process::exit(0);
                    }
                }
            }
        }
        gtk::glib::ControlFlow::Continue
    });

    gtk::main();
    true
}

fn main() {
    #[cfg(target_os = "windows")]
    if !std::env::args().any(|a| a == "--ui") && !is_admin::is_admin() {
        let exe = std::env::current_exe();
        if let Ok(exe) = exe {
            let args: Vec<String> = std::env::args().skip(1).collect();
            let relaunched = runas::Command::new(&exe).args(&args).gui(true).status();
            match relaunched {
                Ok(status) if status.success() => return,
                _ => {}
            }
        }
    }

    if std::env::args().any(|a| a == "--ui") {
        let _ = ui::run();
        return;
    }

    // Prevent a second collector from writing the same database
    let _singleton = match common::SingletonGuard::acquire(common::DATABASE_PATH) {
        Ok(guard) => guard,
        Err(msg) => {
            common::clog!("✗ {msg}");
            return;
        }
    };

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut app = match CollectorApp::new() {
            Ok(app) => app,
            Err(_) => return,
        };
        if app.initialize().is_err() {
            return;
        }
        tx.send(()).unwrap_or_default();
        app.run();
    });

    // Wait for collector to finish initializing
    let _ = rx.recv();

    let ui_child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    spawn_ui(&ui_child).ok();

    // Windows/macOS: system tray + winit event loop
    #[cfg(not(target_os = "linux"))]
    {
        let event_loop = match EventLoop::new() {
            Ok(loop_handle) => loop_handle,
            Err(_) => return,
        };

        let _tray_icon = setup_tray(&ui_child);

        let ui_child_watcher = Arc::clone(&ui_child);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(250));
                let mut guard = match ui_child_watcher.lock() {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                if let Some(child) = guard.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code().unwrap_or(0);
                            *guard = None;
                            if code == common::EXIT_CODE_SHUTDOWN_ALL {
                                std::process::exit(0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        struct TrayApp;
        impl ApplicationHandler for TrayApp {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
        }
        event_loop.run_app(&mut TrayApp).ok();
    }

    // Linux: try tray with GTK, fall back to simple monitoring
    #[cfg(target_os = "linux")]
    {
        if !run_linux_tray(&ui_child) {
            common::clog!("⚠ System tray unavailable, running without tray icon");
            loop {
                thread::sleep(Duration::from_millis(250));
                let mut guard = match ui_child.lock() {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                if let Some(child) = guard.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code().unwrap_or(0);
                            *guard = None;
                            if code == common::EXIT_CODE_SHUTDOWN_ALL {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
