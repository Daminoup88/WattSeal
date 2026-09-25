#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    net::SocketAddr,
    process::{Child, Command},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use bpaf::{OptionParser, Parser, construct, long, short};
use collector::{CollectorApp, ConsumptionUnit, MQTTInfo};
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

/// Windows process creation flag: spawn the child without a console window.
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Configuration options for the application.
#[derive(Debug, Clone)]
struct Options {
    ui_mode: bool,
    overlay_mode: bool,
    background_mode: bool,
    headless_mode: bool,
    mqtt_id: Option<String>,
    mqtt_addr: Option<SocketAddr>,
    mqtt_unit: Option<ConsumptionUnit>,
    mqtt_raw: bool,
    db_mode: bool,
    #[cfg(target_os = "windows")]
    install_cpu_driver: bool,
    #[cfg(target_os = "windows")]
    uninstall_cpu_driver: bool,
}

/// Returns options parser to run
fn options() -> OptionParser<Options> {
    let ui_mode = long("ui")
        .help("Launch Wattseal with the graphical user interface.")
        .switch();

    let overlay_mode = long("overlay")
        .help("Launch the always-on-top overlay widget.")
        .switch();

    let background_mode = short('b')
        .long("background")
        .long("bg")
        .help(
            "Runs Wattseal in the background. It's possible to return to the
            UI mode from the tray icons",
        )
        .switch();

    let headless_mode = long("headless")
        .help("Runs only Wattseal sensors, without UI and tray icon.")
        .switch();

    let mqtt_id = long("mqtt-id")
        .help("Identifier used as the root of MQTT topics (e.g. my-machine/cpu, my-machine/ram). Requires --mqtt-addr to be set. Defaults to \"wattseal_collector\".")
        .argument::<String>("ID")
        .optional();

    let mqtt_addr = long("mqtt-addr")
        .help("Specify MQTT broker address to send sensors data.")
        .argument::<SocketAddr>("ADDRESS")
        .optional();

    let mqtt_unit = long("mqtt-unit")
        .help(
            "Unit for collector consumption values published via MQTT. \
       One of: uj (microjoules), wh (watt-hours). \
       If omitted, returns raw collector values with their original unit (uj).",
        )
        .argument::<String>("UNIT")
        .parse(|s| match s.as_str() {
            "uj" => Ok(ConsumptionUnit::UJoul),
            "wh" => Ok(ConsumptionUnit::WattHour),
            other => Err(format!("Unknown returns unit '{}' for MQTT: expected uj or wh.", other)),
        })
        .optional();

    let mqtt_raw = long("mqtt-raw")
        .help("Publish directly raw (non-computed) sensor data.")
        .switch();

    let db_mode = long("no-db")
        .help("Do not save sensors metrics in local database.")
        .flag(false, true);

    let description = "WattSeal - Per-app power monitoring tool";

    #[cfg(target_os = "windows")]
    {
        let install_cpu_driver = long("install-cpu-driver")
            .help("Install the Windows CPU MSR driver (requires Administrator privileges).")
            .switch();

        let uninstall_cpu_driver = long("uninstall-cpu-driver")
            .help("Uninstall the Windows CPU MSR driver (requires Administrator privileges).")
            .switch();

        return construct!(Options {
            ui_mode,
            overlay_mode,
            background_mode,
            headless_mode,
            mqtt_id,
            mqtt_addr,
            mqtt_unit,
            mqtt_raw,
            db_mode,
            install_cpu_driver,
            uninstall_cpu_driver,
        })
        .to_options()
        .descr(description);
    }

    #[cfg(not(target_os = "windows"))]
    {
        return construct!(Options {
            ui_mode,
            overlay_mode,
            background_mode,
            headless_mode,
            mqtt_id,
            mqtt_addr,
            mqtt_unit,
            mqtt_raw,
            db_mode,
        })
        .to_options()
        .descr(description);
    }
}

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
                common::clog!("✗ {msg}");
                #[cfg(target_os = "windows")]
                if std::env::var("ICED_BACKEND").as_deref() != Ok("tiny-skia") {
                    // Safe on Windows even in multi-threaded
                    unsafe {
                        std::env::set_var("ICED_BACKEND", "tiny-skia");
                    }
                    drop(guard);
                    return spawn_ui(ui_child);
                }
                Err(msg)
            }
        }
    } else {
        let msg = "Failed to determine current executable path".to_string();
        common::clog!("✗ {msg}");
        Err(msg)
    }
}

/// Spawns the overlay subprocess (`--overlay`) if it is not already running.
/// The overlay runs in its own process so it never blocks the main thread.
fn spawn_overlay(overlay_child: &Arc<Mutex<Option<Child>>>) -> Result<(), String> {
    let mut guard = overlay_child
        .lock()
        .map_err(|e| format!("Failed to lock overlay child mutex: {e}"))?;
    let already_running = guard.as_mut().is_some_and(|c| matches!(c.try_wait(), Ok(None)));
    if already_running {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| format!("Failed to determine executable path: {e}"))?;
    let mut command = Command::new(exe);
    command.arg("--overlay");
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    match command.spawn() {
        Ok(child) => {
            *guard = Some(child);
            Ok(())
        }
        Err(e) => Err(format!("Failed to spawn overlay process: {e}")),
    }
}

/// Toggles the overlay subprocess: launches it when off, terminates it on.
fn toggle_overlay(overlay_child: &Arc<Mutex<Option<Child>>>) {
    let mut guard = match overlay_child.lock() {
        Ok(g) => g,
        Err(e) => {
            common::clog!("✗ Failed to lock overlay child mutex: {e}");
            return;
        }
    };
    let running = guard.as_mut().is_some_and(|c| matches!(c.try_wait(), Ok(None)));
    if running {
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
        *guard = None;
    } else {
        drop(guard);
        if let Err(e) = spawn_overlay(overlay_child) {
            common::clog!("✗ {e}");
        }
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
fn setup_tray(
    ui_child: &Arc<Mutex<Option<Child>>>,
    overlay_child: &Arc<Mutex<Option<Child>>>,
) -> Option<tray_icon::TrayIcon> {
    let tray_menu = Menu::new();
    let open_ui_i = MenuItem::new("Open UI", true, None);
    let toggle_overlay_i = MenuItem::new("Toggle Overlay", true, None);
    // Only offered where pinning can actually hide the overlay from the mouse,
    // because there the tray is the sole escape hatch. Where click-through is
    // unavailable the overlay can always unlock itself from its own right-click
    // menu, so the entry would just be noise — and the tray itself may not even
    // exist (GNOME without an AppIndicator extension).
    //
    // A plain item rather than a `CheckMenuItem`: muda's check items hold `Rc`
    // and are therefore not `Send`, so they cannot be moved into the event
    // handler to keep their tick in sync.
    let pin_overlay_i = overlay::click_through_supported().then(|| MenuItem::new("Pin / Unpin Overlay", true, None));
    let quit_i = MenuItem::new("Quit", true, None);
    let open_ui_id = open_ui_i.id().to_owned();
    let toggle_overlay_id = toggle_overlay_i.id().to_owned();
    let pin_overlay_id = pin_overlay_i.as_ref().map(|item| item.id().to_owned());
    let quit_id = quit_i.id().to_owned();

    let separator = PredefinedMenuItem::separator();
    tray_menu.append_items(&[&open_ui_i, &toggle_overlay_i]).ok();
    if let Some(pin_overlay_i) = &pin_overlay_i {
        tray_menu.append_items(&[pin_overlay_i]).ok();
    }
    tray_menu.append_items(&[&separator, &quit_i]).ok();

    let ui_child_menu = Arc::clone(ui_child);
    let overlay_child_menu = Arc::clone(overlay_child);
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_ui_id {
            spawn_ui(&ui_child_menu).ok();
        } else if event.id == toggle_overlay_id {
            toggle_overlay(&overlay_child_menu);
        } else if pin_overlay_id.as_ref() == Some(&event.id) {
            // Pin mode lives in the overlay's own config file, which the overlay
            // polls once a second — so writing it is enough, no IPC required.
            //
            // Deliberately does not start the overlay: pinning also makes the
            // window ignore the mouse, so launching one from here would hand the
            // user a window they never asked for and cannot grab.
            let pinned = overlay::toggle_pin();
            if pinned {
                common::clog!("✓ Overlay pinned: release it from this menu or from the dashboard");
            } else {
                common::clog!("✓ Overlay unpinned: mouse input restored");
            }
        } else if event.id == quit_id {
            // Ask first: the overlay may have been started from the main window,
            // so this process may hold no handle for it and `kill` would miss it.
            overlay::request_close();
            if let Ok(mut child_guard) = ui_child_menu.lock() {
                if let Some(c) = child_guard.as_mut() {
                    let _ = c.kill();
                }
            }
            if let Ok(mut overlay_guard) = overlay_child_menu.lock() {
                if let Some(c) = overlay_guard.as_mut() {
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
fn run_linux_tray(ui_child: &Arc<Mutex<Option<Child>>>, overlay_child: &Arc<Mutex<Option<Child>>>) -> bool {
    if gtk::init().is_err() {
        return false;
    }

    let _tray_icon = match setup_tray(ui_child, overlay_child) {
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

/// Initializes the collector
fn start_collector(enable_save_db: bool, mqtt_info: Option<MQTTInfo>) -> Result<CollectorApp, String> {
    let mut app =
        CollectorApp::new(enable_save_db, mqtt_info).map_err(|e| format!("Failed to create CollectorApp: {e}"))?;
    app.initialize()
        .map_err(|e| format!("Failed to initialize CollectorApp: {e}"))?;
    Ok(app)
}

fn main() {
    if let Err(e) = common::set_current_dir_to_exe_dir() {
        common::clog!("⚠ Failed to set working directory to executable directory: {}", e);
    }

    let options = options().run();

    #[cfg(target_os = "windows")]
    {
        if options.install_cpu_driver {
            collector::sensors::cpu::windows_cpu::install();
            return;
        }

        if options.uninstall_cpu_driver {
            collector::sensors::cpu::windows_cpu::uninstall();
            return;
        }

        // Prevent the UI / overlay subprocesses from setting up the driver again
        if !options.ui_mode && !options.overlay_mode {
            collector::sensors::cpu::windows_cpu::setup();
        }
    }

    if !options.db_mode && options.mqtt_addr.is_none() {
        let msg = format!("Impossible to run without both local data storage and an MQTT broker.");
        common::clog!("✗ {msg}");
        return;
    }

    if options.mqtt_addr.is_none() && options.mqtt_id.is_some() {
        let msg = format!("An MQTT broker address must be entered in order to specify the collector's MQTT topic.");
        common::clog!("✗ {msg}");
        return;
    }

    if options.headless_mode && (options.ui_mode || options.background_mode) {
        let msg = format!("Impossible to run headless mode if UI or background mode is enabled");
        common::clog!("✗ {msg}");
        return;
    }

    if options.ui_mode {
        if let Err(err) = ui::run() {
            common::clog!("✗ UI failed to start: {err}");
        }
        return;
    }

    if options.overlay_mode {
        if let Err(err) = overlay::run() {
            common::clog!("✗ Overlay failed to start: {err}");
        }
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

    let mqtt_info = if let Some(mqtt_addr) = options.mqtt_addr {
        let id = options.mqtt_id.unwrap_or("wattseal_collector".to_string());
        let unit = options.mqtt_unit;
        Some(MQTTInfo::new(&id, &mqtt_addr, unit, options.mqtt_raw))
    } else {
        None
    };

    if options.headless_mode {
        match start_collector(options.db_mode, mqtt_info) {
            Ok(mut app) => app.run(),
            Err(e) => common::clog!("✗ {e}"),
        }
        return;
    }

    // Doesn't run in headless mode
    let (tx, rx) = mpsc::channel::<Result<(), String>>();

    thread::spawn(move || {
        let mut app = match start_collector(options.db_mode, mqtt_info) {
            Ok(app) => app,
            Err(e) => {
                common::clog!("✗ {e}");
                let _ = tx.send(Err(e));
                return;
            }
        };
        let _ = tx.send(Ok(()));
        app.run();
    });

    // Wait for collector to finish initializing
    match rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => {
            common::clog!("✗ {msg}");
            return;
        }
        Err(e) => {
            common::clog!("✗ Collector thread ended before signaling readiness: {}", e);
            return;
        }
    }

    let ui_child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let overlay_child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    if !options.background_mode {
        spawn_ui(&ui_child).ok();
    }

    // Windows/macOS: system tray + winit event loop
    #[cfg(not(target_os = "linux"))]
    {
        let event_loop = match EventLoop::new() {
            Ok(loop_handle) => loop_handle,
            Err(e) => {
                common::clog!("✗ Failed to create event loop: {e}");
                return;
            }
        };

        let _tray_icon = setup_tray(&ui_child, &overlay_child);

        let ui_child_watcher = Arc::clone(&ui_child);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(250));
                let mut guard = match ui_child_watcher.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        common::clog!("✗ Failed to lock UI child mutex: {}", e);
                        continue;
                    }
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

        // The overlay is a fire-and-forget subprocess: clean up its handle when
        // it exits so the tray toggle reflects the real state.
        let overlay_child_watcher = Arc::clone(&overlay_child);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(250));
                let mut guard = match overlay_child_watcher.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        common::clog!("✗ Failed to lock overlay child mutex: {}", e);
                        continue;
                    }
                };
                if let Some(child) = guard.as_mut() {
                    if child.try_wait().is_ok_and(|status| status.is_some()) {
                        *guard = None;
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
        if !run_linux_tray(&ui_child, &overlay_child) {
            common::clog!("⚠ System tray unavailable, running without tray icon");
            loop {
                thread::sleep(Duration::from_millis(250));
                let mut guard = match ui_child.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        common::clog!("✗ Failed to lock UI child mutex: {}", e);
                        continue;
                    }
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
