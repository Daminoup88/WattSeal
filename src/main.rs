use std::{sync::mpsc, thread};

use collector::CollectorApp;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        println!("Starting collector...");
        let mut app = CollectorApp::new().expect("Failed to create CollectorApp");

        if let Err(e) = app.initialize() {
            eprintln!("Failed to initialize collector: {}", e);
            return;
        }
        tx.send(()).unwrap_or_default();
        app.run();
    });

    let _ = rx.recv();
    println!("Starting UI...");
    if let Err(e) = ui::run() {
        eprintln!("UI error: {}", e);
    }
}
