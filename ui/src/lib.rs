#![allow(dead_code, unused_imports)]

use iced::font::Font;

pub mod app;
pub mod components;
pub mod message;
pub mod pages;
pub mod themes;

use app::App;

pub fn run() -> iced::Result {
    iced::application("Energy Monitor", App::update, App::view)
        .antialiasing(true)
        .default_font(Font::with_name("Roboto"))
        .subscription(App::subscription)
        .theme(App::theme)
        .run_with(App::new)
}
