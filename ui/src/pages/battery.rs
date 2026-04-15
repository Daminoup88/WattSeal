use common::Database;
use iced::{
    Alignment, Element, Length, Padding,
    widget::{Column, Container, Row, Scrollable, Text},
};

use crate::{
    message::Message,
    styles::{
        container::ContainerStyle,
        scrollable::ScrollableStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SUBTITLE, PADDING_LARGE, PADDING_MEDIUM,
            SPACING_LARGE, SPACING_MEDIUM,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    types::AppLanguage,
};

/// Battery health page showing current health and trend.
pub struct BatteryPage;

impl BatteryPage {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self, _language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let db = match Database::new() {
            Ok(db) => db,
            Err(_) => {
                return Container::new(Text::new("Database unavailable").size(FONT_SIZE_BODY))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from(PADDING_LARGE))
                    .into();
            }
        };

        let latest = db.select_latest_battery_health().ok().flatten();

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let day_ago = now_ms - 24 * 3600 * 1000;
        let history = db.select_battery_health(day_ago, now_ms).unwrap_or_default();

        let title = Text::new("Battery Health")
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .class(TextStyle::Primary);

        let mut content = Column::new()
            .spacing(SPACING_LARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .push(title);

        // Current health card
        let health_card = if let Some(ref record) = latest {
            let health_text = format!("{:.1}%", record.health_percent);
            let discharge_text = record
                .discharge_rate_watts
                .map(|w| format!("{:.1} W", w))
                .unwrap_or_else(|| "N/A".to_string());
            let cycle_text = record
                .cycle_count
                .map(|c| format!("{}", c))
                .unwrap_or_else(|| "N/A".to_string());

            let health_row = Row::new()
                .spacing(SPACING_MEDIUM)
                .align_y(Alignment::Center)
                .push(
                    Text::new("Health:")
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                )
                .push(
                    Text::new(health_text)
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Primary),
                );

            let discharge_row = Row::new()
                .spacing(SPACING_MEDIUM)
                .align_y(Alignment::Center)
                .push(
                    Text::new("Discharge Rate:")
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                )
                .push(
                    Text::new(discharge_text)
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Secondary),
                );

            let cycle_row = Row::new()
                .spacing(SPACING_MEDIUM)
                .align_y(Alignment::Center)
                .push(
                    Text::new("Cycle Count:")
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                )
                .push(
                    Text::new(cycle_text)
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Tertiary),
                );

            Column::new()
                .spacing(SPACING_MEDIUM)
                .push(health_row)
                .push(discharge_row)
                .push(cycle_row)
        } else {
            Column::new().push(
                Text::new("No battery detected or no health data available")
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted),
            )
        };

        let card_container = Container::new(health_card)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_MEDIUM))
            .class(ContainerStyle::Card);

        content = content.push(card_container);

        // History summary
        if !history.is_empty() {
            let trend_title = Text::new("Health Trend (Last 24h)")
                .size(FONT_SIZE_SUBTITLE)
                .font(FONT_BOLD)
                .class(TextStyle::Subtitle);

            let min_health = history.iter().map(|r| r.health_percent).fold(f64::MAX, f64::min);
            let max_health = history.iter().map(|r| r.health_percent).fold(f64::MIN, f64::max);
            let avg_health = history.iter().map(|r| r.health_percent).sum::<f64>() / history.len() as f64;

            let stats = Column::new()
                .spacing(SPACING_MEDIUM)
                .push(
                    Row::new()
                        .spacing(SPACING_MEDIUM)
                        .push(Text::new("Min:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
                        .push(
                            Text::new(format!("{:.1}%", min_health))
                                .size(FONT_SIZE_BODY)
                                .font(FONT_BOLD),
                        )
                        .push(Text::new("Avg:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
                        .push(
                            Text::new(format!("{:.1}%", avg_health))
                                .size(FONT_SIZE_BODY)
                                .font(FONT_BOLD),
                        )
                        .push(Text::new("Max:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
                        .push(
                            Text::new(format!("{:.1}%", max_health))
                                .size(FONT_SIZE_BODY)
                                .font(FONT_BOLD),
                        ),
                )
                .push(
                    Text::new(format!("Data points: {}", history.len()))
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                );

            let trend_container = Container::new(Column::new().spacing(SPACING_MEDIUM).push(trend_title).push(stats))
                .width(Length::Fill)
                .padding(Padding::from(PADDING_MEDIUM))
                .class(ContainerStyle::Card);

            content = content.push(trend_container);
        }

        Scrollable::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .class(ScrollableStyle::Standard)
            .into()
    }
}
