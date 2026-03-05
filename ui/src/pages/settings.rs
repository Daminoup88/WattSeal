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
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SUBTITLE, PADDING_MEDIUM, PADDING_XLARGE,
            SPACING_LARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{
        custom_carbon_invalid, custom_carbon_placeholder, custom_kwh_cost_placeholder, kwh_cost_invalid, modal_close,
        settings_carbon_intensity, settings_electricity_cost, settings_general, settings_language, settings_theme,
        settings_title,
    },
    types::{AppLanguage, CarbonIntensity, ElectricityCost},
};

/// Settings modal for theme, language, carbon intensity, and electricity cost.
pub struct SettingsPage {}

impl SettingsPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view<'a>(
        &'a self,
        theme: AppTheme,
        language: AppLanguage,
        carbon_intensity: CarbonIntensity,
        custom_carbon_input: &'a str,
        electricity_cost: ElectricityCost,
        custom_kwh_cost_input: &'a str,
    ) -> Element<'a, Message, AppTheme> {
        let title = Text::new(settings_title(language))
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let subtitle = Text::new(settings_general(language))
            .size(FONT_SIZE_SUBTITLE)
            .class(TextStyle::Muted);

        let theme_row = settings_row(
            settings_theme(language),
            pick_list(AppTheme::all(), Some(theme), Message::ChangeTheme)
                .width(Length::FillPortion(3))
                .padding(PADDING_MEDIUM)
                .into(),
        );

        let language_row = settings_row(
            settings_language(language),
            pick_list(AppLanguage::all(), Some(language), Message::ChangeLanguage)
                .width(Length::FillPortion(3))
                .padding(PADDING_MEDIUM)
                .into(),
        );

        let carbon_row = carbon_intensity_row(language, carbon_intensity, custom_carbon_input);
        let kwh_row = electricity_cost_row(language, electricity_cost, custom_kwh_cost_input);

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
            .push(carbon_row)
            .push(kwh_row);

        Container::new(content)
            .width(Length::Fixed(520.0))
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }
}

/// Renders a label + single widget row, consistent with all settings rows.
fn settings_row<'a>(label: &'a str, control: Element<'a, Message, AppTheme>) -> Element<'a, Message, AppTheme> {
    Row::new()
        .spacing(SPACING_LARGE)
        .align_y(Alignment::Center)
        .push(
            Text::new(label)
                .size(FONT_SIZE_BODY)
                .width(Length::FillPortion(2)),
        )
        .push(control)
        .into()
}

fn carbon_intensity_row<'a>(
    language: AppLanguage,
    carbon_intensity: CarbonIntensity,
    custom_carbon_input: &'a str,
) -> Element<'a, Message, AppTheme> {
    let custom_valid = custom_carbon_input.parse::<f64>().ok().filter(|&v| v > 0.0).is_some();

    let picker = pick_list(
        CarbonIntensity::PRESETS.to_vec(),
        Some(carbon_intensity),
        Message::ChangeCarbonIntensity,
    )
    .width(Length::FillPortion(3))
    .padding(PADDING_MEDIUM);

    let right_col: Element<'_, Message, AppTheme> = if carbon_intensity.is_custom() {
        let input = text_input(custom_carbon_placeholder(language), custom_carbon_input)
            .on_input(Message::CustomCarbonInput)
            .width(Length::FillPortion(3))
            .padding(PADDING_MEDIUM);
        let mut col = Column::new().width(Length::FillPortion(3)).spacing(4).push(picker).push(input);
        if !custom_carbon_input.is_empty() && !custom_valid {
            col = col.push(
                Text::new(custom_carbon_invalid(language))
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted),
            );
        }
        col.into()
    } else {
        picker.into()
    };

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
}

fn electricity_cost_row<'a>(
    language: AppLanguage,
    electricity_cost: ElectricityCost,
    custom_kwh_cost_input: &'a str,
) -> Element<'a, Message, AppTheme> {
    let custom_valid = custom_kwh_cost_input.parse::<f64>().ok().filter(|&v| v >= 0.0).is_some();

    let picker = pick_list(
        ElectricityCost::PRESETS.to_vec(),
        Some(electricity_cost),
        Message::ChangeElectricityCost,
    )
    .width(Length::FillPortion(3))
    .padding(PADDING_MEDIUM);

    let right_col: Element<'_, Message, AppTheme> = if electricity_cost.is_custom() {
        let input = text_input(custom_kwh_cost_placeholder(language), custom_kwh_cost_input)
            .on_input(Message::CustomKwhCostInput)
            .width(Length::Fill)
            .padding(PADDING_MEDIUM);
        let input_row = Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(input)
            .push(Text::new("$/kWh").size(FONT_SIZE_BODY).class(TextStyle::Muted));
        let mut col = Column::new().width(Length::FillPortion(3)).spacing(4).push(picker).push(input_row);
        if !custom_kwh_cost_input.is_empty() && !custom_valid {
            col = col.push(
                Text::new(kwh_cost_invalid(language))
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted),
            );
        }
        col.into()
    } else {
        picker.into()
    };

    Row::new()
        .spacing(SPACING_LARGE)
        .align_y(Alignment::Start)
        .push(
            Text::new(settings_electricity_cost(language))
                .size(FONT_SIZE_BODY)
                .width(Length::FillPortion(2)),
        )
        .push(right_col)
        .into()
}
