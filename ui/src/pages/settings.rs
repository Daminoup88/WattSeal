use iced::{
    Alignment, Element, Length,
    widget::{Button, Column, Container, Row, Text, button, pick_list, text_input},
};

use crate::{
    message::Message,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SUBTITLE, PADDING_LARGE, PADDING_MEDIUM,
            PADDING_XLARGE, SPACING_LARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{
        self, custom_carbon_invalid, custom_carbon_placeholder, modal_close, settings_carbon_intensity,
        settings_general, settings_language, settings_theme, settings_title,
    },
    types::{AppLanguage, CarbonIntensity},
};

pub struct SettingsPage {}

impl SettingsPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(
        &self,
        theme: AppTheme,
        language: AppLanguage,
        carbon_intensity: CarbonIntensity,
        custom_carbon_input: &str,
    ) -> Element<'_, Message, AppTheme> {
        let title = Text::new(settings_title(language))
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let subtitle = Text::new(settings_general(language))
            .size(FONT_SIZE_SUBTITLE)
            .class(TextStyle::Muted);

        let theme_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(
                Text::new(settings_theme(language))
                    .size(FONT_SIZE_BODY)
                    .width(Length::FillPortion(2)),
            )
            .push(
                pick_list(AppTheme::all(), Some(theme), Message::ChangeTheme)
                    .width(Length::FillPortion(3))
                    .padding(PADDING_MEDIUM),
            );

        let language_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(
                Text::new(settings_language(language))
                    .size(FONT_SIZE_BODY)
                    .width(Length::FillPortion(2)),
            )
            .push(
                pick_list(AppLanguage::all(), Some(language), Message::ChangeLanguage)
                    .width(Length::FillPortion(3))
                    .padding(PADDING_MEDIUM),
            );

        let custom_input_valid = custom_carbon_input.parse::<f64>().ok().filter(|&v| v > 0.0).is_some();

        let ci_picker = pick_list(
            CarbonIntensity::PRESETS.to_vec(),
            Some(carbon_intensity),
            Message::ChangeCarbonIntensity,
        )
        .width(Length::FillPortion(3))
        .padding(PADDING_MEDIUM);

        let carbon_row: Element<'_, Message, AppTheme> = if carbon_intensity.is_custom() {
            let input = text_input(custom_carbon_placeholder(language), custom_carbon_input)
                .on_input(Message::CustomCarbonInput)
                .width(Length::FillPortion(3))
                .padding(PADDING_MEDIUM);
            let mut right_col = Column::new().spacing(4).push(ci_picker).push(input);
            if !custom_carbon_input.is_empty() && !custom_input_valid {
                right_col = right_col.push(
                    Text::new(custom_carbon_invalid(language))
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                );
            }
            Row::new()
                .spacing(SPACING_LARGE)
                .align_y(Alignment::Start)
                .push(
                    Text::new(settings_carbon_intensity(language))
                        .size(FONT_SIZE_BODY)
                        .width(Length::FillPortion(2)),
                )
                .push(right_col)
                .into()
        } else {
            Row::new()
                .spacing(SPACING_LARGE)
                .align_y(Alignment::Center)
                .push(
                    Text::new(settings_carbon_intensity(language))
                        .size(FONT_SIZE_BODY)
                        .width(Length::FillPortion(2)),
                )
                .push(ci_picker)
                .into()
        };

        let close_button: Button<'_, Message, AppTheme> = button(Text::new(modal_close(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::CloseSettings);

        let top_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(title)
            .push(close_button);

        let content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(top_row)
            .push(subtitle)
            .push(theme_row)
            .push(language_row)
            .push(carbon_row);

        Container::new(content)
            .width(Length::Fixed(520.0))
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }
}
