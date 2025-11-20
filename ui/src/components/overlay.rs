use iced::{Color, Element, Length};
use iced::widget::{button, center, column, container, text, Container};
use iced::alignment::Alignment;

use crate::message::Message;

/// A reusable overlay component that can be displayed on top of any page
pub struct Overlay {
    pub visible: bool,
}

impl Overlay {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Render the overlay content
    pub fn view(&self) -> Element<'_, Message> {
        if !self.visible {
            return Container::new(column![]).into();
        }

        // Semi-transparent dark background
        let backdrop = Container::new(
            center(
                column![
                    text("Example Overlay").size(24),
                    text("This overlay appears on top of all pages"),
                    button("Close").on_press(Message::ToggleOverlay),
                ]
                .spacing(20)
                .padding(40)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
            ..Default::default()
        });

        backdrop.into()
    }
}
