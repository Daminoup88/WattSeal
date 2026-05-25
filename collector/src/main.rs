use collector::CollectorApp;

fn main() {
    if let Err(e) = common::set_current_dir_to_exe_dir() {
        common::clog!("⚠ Failed to set working directory to executable directory: {}", e);
    }

    let _singleton = match common::SingletonGuard::acquire(common::DATABASE_PATH) {
        Ok(guard) => guard,
        Err(msg) => {
            common::clog!("✗ {msg}");
            return;
        }
    };

    let mut app = match CollectorApp::new(true, None) {
        Ok(app) => app,
        Err(e) => {
            common::clog!("✗ Failed to create CollectorApp: {}", e);
            return;
        }
    };
    if let Err(e) = app.initialize() {
        common::clog!("✗ Failed to initialize CollectorApp: {}", e);
        return;
    }
    app.run();
}
