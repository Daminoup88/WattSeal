use collector::CollectorApp;

fn main() {
    let _singleton = match common::SingletonGuard::acquire(common::DATABASE_PATH) {
        Ok(guard) => guard,
        Err(msg) => {
            eprintln!("{msg}");
            return;
        }
    };

    let mut app = match CollectorApp::new() {
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
