use iced::{
    Element, Length,
    widget::{
        Column, Container, Row, Text, center, mouse_area, opaque, stack,
        text::{self, Catalog},
    },
};

use crate::{
    message::Message,
    styles::{
        container::ContainerStyle,
        style_constants::{FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_SMALL},
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{self, no_data_available},
    types::AppLanguage,
};

pub fn text_widget<'a>(
    text: impl text::IntoFragment<'a>,
    size: f32,
    class: TextStyle,
    width: Length,
    is_bold: bool,
) -> Element<'a, Message, AppTheme> {
    let mut text = Text::new(text).size(size).class(class).width(width);
    if is_bold {
        text = text.font(FONT_BOLD);
    }
    text.into()
}

pub fn no_data_placeholder<'a>(language: AppLanguage) -> Element<'a, Message, AppTheme> {
    Text::new(no_data_available(language))
        .size(FONT_SIZE_BODY)
        .class(TextStyle::Muted)
        .into()
}

pub fn modal<'a>(
    background: Element<'a, Message, AppTheme>,
    content: Element<'a, Message, AppTheme>,
    on_click_outside: Message,
) -> Element<'a, Message, AppTheme> {
    let mouse_area =
        mouse_area(center(opaque(content)).class(ContainerStyle::ModalBackdrop)).on_press(on_click_outside);

    stack![background, opaque(mouse_area)].into()
}
