use iced::{
    Alignment, Element, Length,
    widget::{
        Button, Column, Container, Row, Text, button, pick_list,
        text::Wrapping,
        text_input, toggler,
    },
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
        toggler::TogglerStyle,
    },
    themes::AppTheme,
    translations::{
        TranslatedCarbonIntensity, TranslatedElectricityCost, TranslatedTheme, custom_carbon_invalid,
        custom_carbon_placeholder, custom_kwh_cost_placeholder, kwh_cost_invalid, modal_close,
        settings_carbon_intensity, settings_electricity_cost, settings_general, settings_install_location,
        settings_language, settings_launch_on_startup, settings_open_folder, settings_theme, settings_title,
    },
    types::{AppLanguage, CarbonIntensity, Currency, ElectricityCost},
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
        launch_on_startup: bool,
        install_dir: &'a str,
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
            pick_list(
                TranslatedTheme::all(language),
                Some(TranslatedTheme::new(theme, language)),
                |tt| Message::ChangeTheme(tt.theme),
            )
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

        let mut content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(top_row)
            .push(subtitle)
            .push(theme_row)
            .push(language_row)
            .push(install_location_row(language, install_dir));

        if common::autostart::is_supported() {
            content = content.push(launch_on_startup_row(language, launch_on_startup));
        }

        let content = content.push(carbon_row).push(kwh_row);

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
        .push(Text::new(label).size(FONT_SIZE_BODY).width(Length::FillPortion(2)))
        .push(control)
        .into()
}

/// Longest install path we'll display before middle-truncating it. Kept at roughly
/// single-line capacity for this row: the text shaper treats `\` as a break point,
/// so a path that doesn't fit on one line wraps right after the drive letter
/// (e.g. "C:\" alone on line 1, everything else crammed onto line 2) rather than
/// wrapping cleanly - so we truncate before that point instead of past it.
const INSTALL_PATH_MAX_CHARS: usize = 42;

/// Middle-truncates a long path, keeping the start (drive/root) and end (innermost
/// folder) visible since those are the most useful parts to recognize at a glance.
///
/// Only ever drops whole path components (never cuts through the middle of a
/// folder name) by growing the kept prefix/suffix one component at a time,
/// replacing whatever's dropped in between with a lone "..." segment.
fn truncate_path_display(path: &str, max_chars: usize) -> String {
    if path.chars().count() <= max_chars {
        return path.to_string();
    }

    let separator = if path.contains('\\') { '\\' } else { '/' };
    let components: Vec<&str> = path.split(separator).collect();

    let mut front_end = 1;
    let mut back_start = components.len().saturating_sub(1);
    if front_end >= back_start {
        return path.to_string();
    }

    const ELLIPSIS: &str = "...";
    let segment_len = |range: std::ops::Range<usize>| -> usize {
        components[range].iter().map(|c| c.chars().count() + 1).sum()
    };

    let mut grow_front = true;
    loop {
        let (candidate_front, candidate_back) = if grow_front {
            (front_end + 1, back_start)
        } else {
            (front_end, back_start - 1)
        };
        if candidate_front >= candidate_back {
            break;
        }
        let total = segment_len(0..candidate_front) + ELLIPSIS.len() + segment_len(candidate_back..components.len());
        if total > max_chars {
            break;
        }
        front_end = candidate_front;
        back_start = candidate_back;
        grow_front = !grow_front;
    }

    let front = components[..front_end].join(&separator.to_string());
    let back = components[back_start..].join(&separator.to_string());
    format!("{front}{separator}{ELLIPSIS}{separator}{back}")
}

/// Renders as a labeled block (not a `settings_row`): the label sits on its own
/// line, with the path (left-aligned, wrapping up to ~2 lines) and the "Open
/// folder" button (right-aligned, inline with the path) directly beneath it.
fn install_location_row<'a>(language: AppLanguage, install_dir: &'a str) -> Element<'a, Message, AppTheme> {
    let label = Text::new(settings_install_location(language)).size(FONT_SIZE_BODY);

    let path_text = Text::new(truncate_path_display(install_dir, INSTALL_PATH_MAX_CHARS))
        .size(FONT_SIZE_BODY)
        .class(TextStyle::Muted)
        .wrapping(Wrapping::WordOrGlyph)
        .width(Length::Fill);

    let open_button: Button<'_, Message, AppTheme> =
        button(Text::new(settings_open_folder(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::OpenInstallFolder);

    let path_row = Row::new()
        .spacing(SPACING_LARGE)
        .align_y(Alignment::Start)
        .push(path_text)
        .push(open_button);

    Column::new().spacing(4).push(label).push(path_row).into()
}

fn launch_on_startup_row<'a>(language: AppLanguage, launch_on_startup: bool) -> Element<'a, Message, AppTheme> {
    settings_row(
        settings_launch_on_startup(language),
        toggler(launch_on_startup)
            .on_toggle(Message::ToggleLaunchOnStartup)
            .class(TogglerStyle::Standard)
            .into(),
    )
}

fn carbon_intensity_row<'a>(
    language: AppLanguage,
    carbon_intensity: CarbonIntensity,
    custom_carbon_input: &'a str,
) -> Element<'a, Message, AppTheme> {
    let custom_valid = custom_carbon_input.parse::<f64>().ok().filter(|&v| v > 0.0).is_some();

    let picker = pick_list(
        TranslatedCarbonIntensity::all(language),
        Some(TranslatedCarbonIntensity::new(carbon_intensity, language)),
        |tci| Message::ChangeCarbonIntensity(tci.intensity),
    )
    .width(Length::FillPortion(3))
    .padding(PADDING_MEDIUM);

    let right_col: Element<'_, Message, AppTheme> = if carbon_intensity.is_custom() {
        let input = text_input(custom_carbon_placeholder(language), custom_carbon_input)
            .on_input(Message::CustomCarbonInput)
            .width(Length::FillPortion(3))
            .padding(PADDING_MEDIUM);
        let mut col = Column::new()
            .width(Length::FillPortion(3))
            .spacing(4)
            .push(picker)
            .push(input);
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
        .align_y(Alignment::Center)
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
    let custom_valid = custom_kwh_cost_input
        .parse::<f64>()
        .ok()
        .filter(|&v| v >= 0.0)
        .is_some();

    let picker = pick_list(
        TranslatedElectricityCost::all(language),
        Some(TranslatedElectricityCost::new(electricity_cost, language)),
        |tec| Message::ChangeElectricityCost(tec.cost),
    )
    .width(Length::FillPortion(3))
    .padding(PADDING_MEDIUM);

    let right_col: Element<'_, Message, AppTheme> = if electricity_cost.is_custom() {
        let input = text_input(custom_kwh_cost_placeholder(language), custom_kwh_cost_input)
            .on_input(Message::CustomKwhCostInput)
            .width(Length::Fill)
            .padding(PADDING_MEDIUM);
        let currency_picker = pick_list(
            Currency::ALL,
            Some(electricity_cost.currency()),
            Message::ChangeCustomCurrency,
        )
        .padding(PADDING_MEDIUM);
        let input_row = Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(input.width(Length::FillPortion(2)))
            .push(currency_picker.width(Length::FillPortion(2)))
            .push(Text::new("/kWh").size(FONT_SIZE_BODY).class(TextStyle::Muted));
        let mut col = Column::new()
            .width(Length::FillPortion(3))
            .spacing(4)
            .push(picker)
            .push(input_row);
        if !custom_kwh_cost_input.is_empty() && !custom_valid {
            col = col.push(
                Text::new(kwh_cost_invalid(language, electricity_cost.currency_symbol))
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
        .align_y(Alignment::Center)
        .push(
            Text::new(settings_electricity_cost(language))
                .size(FONT_SIZE_BODY)
                .width(Length::FillPortion(2)),
        )
        .push(right_col)
        .into()
}
