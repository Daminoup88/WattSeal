#![allow(dead_code, unused_imports)]

use iced::font::Font;

pub mod app;
pub mod components;
pub mod message;
pub mod pages;
pub mod styles;
pub mod themes;
pub mod types;

use app::App;

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Energy Monitor")
        .antialiasing(true)
        .default_font(Font::with_name("Roboto"))
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}
