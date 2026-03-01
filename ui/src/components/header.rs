use iced::{
    Alignment, Element, Length, Padding,
    widget::{Column, Container, Row, Text, button},
};

use crate::{
    message::Message,
    pages::Page,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, PADDING_LARGE, PADDING_MEDIUM, SPACING_MEDIUM},
    },
    themes::AppTheme,
    translations::settings_title,
    types::AppLanguage,
};

pub struct Header {
    nav_pages: Vec<Page>,
    active_page: Page,
}

impl Header {
    pub fn new(nav_pages: Vec<Page>, active_page: Page) -> Self {
        Self { nav_pages, active_page }
    }

    pub fn change_page(&mut self, new_page: Page) {
        self.active_page = new_page;
    }

    pub fn view(&self, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let title = Container::new(
            Text::new(self.active_page.translated_name(language))
                .size(FONT_SIZE_HEADER)
                .font(FONT_BOLD),
        )
        .width(Length::Fill);

        let settings_button = button(Text::new(settings_title(language)).size(FONT_SIZE_BODY))
            .padding(Padding::from([8.0, 16.0]))
            .class(ButtonStyle::Standard)
            .on_press(Message::OpenSettings);

        let nav_buttons = self.nav_pages.iter().fold(Row::new().spacing(8), |row, page| {
            let is_active = self.active_page == *page;
            let button_style = if is_active {
                ButtonStyle::NavActive
            } else {
                ButtonStyle::Nav
            };

            row.push(
                button(Text::new(page.translated_name(language)).size(FONT_SIZE_BODY))
                    .padding(Padding::from([8.0, 16.0]))
                    .class(button_style)
                    .on_press(Message::NavigateTo(*page)),
            )
        });

        let content = Row::new()
            .padding(Padding::from([PADDING_MEDIUM, PADDING_LARGE]))
            .spacing(20)
            .align_y(Alignment::Center)
            .push(title)
            .push(nav_buttons)
            .push(settings_button);

        Container::new(content)
            .width(Length::Fill)
            .class(ContainerStyle::Header)
            .into()
    }
}
