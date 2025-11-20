use iced::{Font, time::Duration};

mod app;
mod components;
mod message;
mod pages;

use app::App;
use message::Message;

fn main() {
    iced::application("CPU Monitor Example", App::update, App::view)
        .antialiasing(true)
        .default_font(Font::with_name("Roboto"))
        .subscription(|_| {
            const FPS: u64 = 1;
            iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
        })
        .run_with(App::new)
        .unwrap();
}