use std::thread;

use collector::CollectorApp;

fn main() {
    thread::spawn(|| {
        println!("Starting collector...");
        let mut app = CollectorApp::new().expect("Failed to create CollectorApp");

        if let Err(e) = app.initialize() {
            eprintln!("Failed to initialize collector: {}", e);
            return;
        }

        app.run();
    });

    println!("Starting UI...");
    if let Err(e) = ui::run() {
        eprintln!("UI error: {}", e);
    }
}
