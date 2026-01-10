use collector::CollectorApp;

fn main() {
    let mut app = CollectorApp::new().expect("Failed to create CollectorApp");
    app.initialize().expect("Failed to initialize CollectorApp");

    app.run();
}
