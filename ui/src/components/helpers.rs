use iced::{
    Element, Length,
    widget::{
        Text, center,
        container::Container,
        mouse_area, opaque,
        rule::horizontal,
        stack,
        text::{self},
    },
};

use crate::{
    message::Message,
    styles::{
        container::ContainerStyle,
        rule::RuleStyle,
        style_constants::{FONT_BOLD, FONT_SIZE_BODY},
        text::TextStyle,
    },
    themes::AppTheme,
    translations::no_data_available,
    types::AppLanguage,
};

/// Creates a styled text element with optional bold font.
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

/// Returns a muted placeholder for empty data sections.
pub fn no_data_placeholder<'a>(language: AppLanguage) -> Element<'a, Message, AppTheme> {
    Text::new(no_data_available(language))
        .size(FONT_SIZE_BODY)
        .class(TextStyle::Muted)
        .into()
}

/// Wraps content in a modal overlay with click-outside dismiss.
pub fn modal<'a>(
    background: Element<'a, Message, AppTheme>,
    content: Element<'a, Message, AppTheme>,
    on_click_outside: Message,
) -> Element<'a, Message, AppTheme> {
    let mouse_area =
        mouse_area(center(opaque(content)).class(ContainerStyle::ModalBackdrop)).on_press(on_click_outside);

    stack![background, opaque(mouse_area)].into()
}

pub fn separator<'a>() -> Element<'a, Message, AppTheme> {
    Container::new(horizontal(1).class(RuleStyle::Subtle))
        .width(Length::Fill)
        .into()
}
