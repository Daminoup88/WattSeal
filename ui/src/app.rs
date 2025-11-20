use iced::{Element, Task};
use iced::widget::stack;

use crate::components::Overlay;
use crate::message::{Message, Page};
use crate::pages::HomePage;

pub struct App {
    /// Current active page
    current_page: Page,
    /// Home page state
    home_page: HomePage,
    /// Common overlay that appears on top of all pages
    overlay: Overlay,
    // Future pages can be added here as the app grows
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_page: Page::Home,
                home_page: HomePage::new(),
                overlay: Overlay::new(),
            },
            Task::done(Message::Tick),
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::NavigateTo(page) => {
                self.current_page = page;
            }
            Message::ToggleOverlay => {
                self.overlay.toggle();
            }
            // Forward other messages to the current page
            _ => {
                match self.current_page {
                    Page::Home => self.home_page.update(&message),
                    // Future pages will handle their own updates here
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Render the current page
        let page_content = match self.current_page {
            Page::Home => self.home_page.view(),
            // Future pages can be rendered here:
            // Page::Settings => self.settings_page.view(),
        };

        // Layer the overlay on top of the page content when visible
        // The Stack widget allows overlaying elements
        stack![
            page_content,
            self.overlay.view(),
        ].into()
    }
}
