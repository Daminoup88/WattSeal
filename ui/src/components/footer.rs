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
    translations::app_name,
    types::AppLanguage,
};

const GITHUB_URL: &str = "https://github.com/Daminoup88/WattSeal/";
const ISSUES_URL: &str = "https://github.com/Daminoup88/WattSeal/issues/new/choose";

/// Minimal footer with version info and external links.
pub struct Footer;

impl Footer {
    pub fn view(&self, language: AppLanguage, overlay_on: bool) -> Element<'_, Message, AppTheme> {
        let version = Text::new(format!("{} v{}", app_name(language), env!("CARGO_PKG_VERSION")))
            .size(FONT_SIZE_SMALL)
            .class(TextStyle::Muted);

        let github_icon = Icon::GitHub.to_text().size(FONT_SIZE_SMALL);

        let star_button = button(
            Row::new()
                .spacing(SPACING_SMALL)
                .align_y(Alignment::Center)
                .push(github_icon)
                .push(Text::new("Star on GitHub").size(FONT_SIZE_SMALL)),
        )
        .padding(Padding::from([4.0, 12.0]))
        .class(ButtonStyle::FooterPrimary)
        .on_press(Message::OpenUrl(GITHUB_URL.to_string()));

        let issue_button = button(
            Row::new()
                .spacing(SPACING_SMALL)
                .align_y(Alignment::Center)
                .push(Text::new("Report Issue").size(FONT_SIZE_SMALL)),
        )
        .padding(Padding::from([4.0, 12.0]))
        .class(ButtonStyle::Footer)
        .on_press(Message::OpenUrl(ISSUES_URL.to_string()));

        // Also the only entry point on systems without a tray (e.g. GNOME
        // without the AppIndicator extension), so it lives in the footer rather
        // than behind the tray menu.
        let overlay_button = button(
            Row::new()
                .spacing(SPACING_SMALL)
                .align_y(Alignment::Center)
                .push(Icon::Display.to_text().size(FONT_SIZE_SMALL))
                .push(
                    Text::new(if overlay_on { "Hide overlay" } else { "Show overlay" })
                        .size(FONT_SIZE_SMALL),
                ),
        )
        .padding(Padding::from([4.0, 12.0]))
        .class(if overlay_on {
            ButtonStyle::FooterPrimary
        } else {
            ButtonStyle::Footer
        })
        .on_press(Message::ToggleOverlay(!overlay_on));

        let right_section = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(overlay_button)
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
