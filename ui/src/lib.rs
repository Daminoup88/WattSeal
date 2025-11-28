use iced::{font::Font, theme::Theme, time::Duration};

pub mod app;
pub mod components;
pub mod message;
pub mod pages;

use app::App;
use message::Message;

pub fn run() -> iced::Result {
    iced::application("CPU Monitor Example", App::update, App::view)
        .antialiasing(true)
        .default_font(Font::with_name("Roboto"))
        .subscription(App::subscription)
        .theme(|_| Theme::Dracula)
        .run_with(App::new)
}
