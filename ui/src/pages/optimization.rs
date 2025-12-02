use iced::{Element, widget::Text};

use crate::message::Message;

pub struct OptimizationPage {}

impl OptimizationPage {
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
        Text::new("Optimization Page Content").into()
    }
}
