use collector::CollectorApp;

fn main() {
    let _ = common::set_current_dir_to_exe_dir();

    let _singleton = match common::SingletonGuard::acquire(common::DATABASE_PATH) {
        Ok(guard) => guard,
        Err(msg) => {
            eprintln!("{msg}");
            return;
        }
    };

    let mut app = match CollectorApp::new(true, None) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to create CollectorApp: {}", e);
            return;
        }
    };
    if let Err(e) = app.initialize() {
        eprintln!("Failed to initialize CollectorApp: {}", e);
        return;
    }
    app.run();
}
