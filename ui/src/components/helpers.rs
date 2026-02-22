use iced::{
    Element, Length,
    widget::{
        Column, Container, Row, Text,
        text::{self, Catalog},
    },
};

use crate::{
    message::Message,
    styles::{style_constants::FONT_BOLD, text::TextStyle},
    themes::AppTheme,
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
