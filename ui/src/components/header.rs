use iced::{
    Element, Font, Length,
    alignment::Alignment,
    font,
    widget::{Container, Row, Text, button},
};

use crate::{message::Message, pages::Page};

const HEADER_FONT: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

pub struct Header {
    title: String,
    nav_pages: Vec<Page>,
}

impl Header {
    pub fn new(title: &str, nav_pages: Vec<Page>) -> Self {
        Self { title: title.into(), nav_pages }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.into();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = Container::new(
            Text::new(&self.title).size(24).font(HEADER_FONT)
        ).width(Length::Fill);

        self.nav_pages.iter()
            .fold(Row::new().padding(10).spacing(20).push(title), |row, page| {
                row.push(
                    button(Text::new(page.to_string()).align_x(Alignment::End))
                        .on_press(Message::NavigateTo(*page))
                )
            })
            .into()
    }
}
