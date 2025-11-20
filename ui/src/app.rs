use iced::{Element, Task};

use crate::message::Message;
use crate::pages::HomePage;

pub struct App {
    home_page: HomePage,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                home_page: HomePage::new(),
            },
            Task::done(Message::Tick),
        )
    }

    pub fn update(&mut self, message: Message) {
        // Update the current page
        self.home_page.update(&message);
    }

    pub fn view(&self) -> Element<'_, Message> {
        // For now, just show the home page
        // This structure allows for adding multiple pages and overlays in the future
        self.home_page.view()
    }
}
