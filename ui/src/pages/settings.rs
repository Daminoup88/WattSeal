use iced::{Element, widget::Text};

use crate::message::Message;

pub struct SettingsPage {}

impl SettingsPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) {
        match message {
            _ => {
                todo!()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        Text::new("Settings Page Content").into()
    }
}
