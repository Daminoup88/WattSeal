use chrono::Utc;
use iced::{
    Color, Element, Font, Length, Task, Theme,
    alignment::Alignment,
    font,
    time::Duration,
    widget::{Column, Container, Row, Text, button, stack},
};

use crate::{message::Message, pages::Page};

pub struct Header {
    title: String,
    navigation_buttons: Vec<Page>,
}

impl Header {
    pub fn new(title: &str, navigation_buttons: Vec<Page>) -> Self {
        Self {
            title: String::from(title),
            navigation_buttons,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = Container::new(Text::new(&self.title).size(24).font(Font {
            family: font::Family::Name("Noto Sans"),
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        }))
        .width(Length::Fill)
        .height(Length::Shrink);

        let mut content = Row::new().padding(10).spacing(20).push(content);
        for navigation_button in &self.navigation_buttons {
            content = content.push(
                button(Text::new(navigation_button.to_string()).align_x(Alignment::End))
                    .on_press(Message::NavigateTo(navigation_button.clone())),
            );
        }
        content.into()
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
}
