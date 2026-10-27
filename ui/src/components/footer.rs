use iced::{
    Alignment, Element, Length, Padding,
    widget::{Container, Row, Space, Text, button},
};

use crate::{
    icons::Icon,
    message::Message,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{FONT_SIZE_SMALL, PADDING_LARGE, PADDING_MEDIUM, SPACING_MEDIUM, SPACING_SMALL},
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{app_name, report_issue, star_on_github},
    types::AppLanguage,
};

const GITHUB_URL: &str = "https://github.com/Daminoup88/WattSeal/";
const ISSUES_URL: &str = "https://github.com/Daminoup88/WattSeal/issues/new/choose";

/// Minimal footer with version info and external links.
pub struct Footer;

impl Footer {
    pub fn view(&self, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let version = Text::new(format!("{} v{}", app_name(language), env!("CARGO_PKG_VERSION")))
            .size(FONT_SIZE_SMALL)
            .class(TextStyle::Muted);

        let github_icon = Icon::GitHub.to_text().size(FONT_SIZE_SMALL);

        let star_button = button(
            Row::new()
                .spacing(SPACING_SMALL)
                .align_y(Alignment::Center)
                .push(github_icon)
                .push(Text::new(star_on_github(language)).size(FONT_SIZE_SMALL)),
        )
        .padding(Padding::from([4.0, 12.0]))
        .class(ButtonStyle::FooterPrimary)
        .on_press(Message::OpenUrl(GITHUB_URL.to_string()));

        let issue_button = button(
            Row::new()
                .spacing(SPACING_SMALL)
                .align_y(Alignment::Center)
                .push(Text::new(report_issue(language)).size(FONT_SIZE_SMALL)),
        )
        .padding(Padding::from([4.0, 12.0]))
        .class(ButtonStyle::Footer)
        .on_press(Message::OpenUrl(ISSUES_URL.to_string()));

        let right_section = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(star_button)
            .push(issue_button);

        let content = Row::new()
            .padding(Padding::from([PADDING_MEDIUM, PADDING_LARGE]))
            .align_y(Alignment::Center)
            .push(version)
            .push(Space::new().width(Length::Fill))
            .push(right_section);

        Container::new(content)
            .width(Length::Fill)
            .class(ContainerStyle::Footer)
            .into()
    }
}
