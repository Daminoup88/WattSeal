use std::collections::HashMap;

use common::{DatabaseEntry, TotalData};
use iced::{
    Alignment, Element, Length, Padding,
    alignment::{Horizontal, Vertical},
    widget::{Column, Container, Row, Scrollable, Text},
};

use crate::{
    components::sensor_state::SensorState,
    message::Message,
    styles::{
        container::ContainerStyle,
        scrollable::ScrollableStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HUGE, FONT_SIZE_SUBTITLE, FONT_SIZE_TITLE, PADDING_LARGE,
            SPACING_LARGE, SPACING_MEDIUM, SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
};

pub struct DashboardPage;

impl DashboardPage {
    pub fn view<'a>(&'a self, sensors: &'a HashMap<String, SensorState>) -> Element<'a, Message, AppTheme> {
        let content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.view_power_summary(sensors));

        let additional_content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.chart_or_placeholder(sensors, None, TotalData::table_name_static(), 300.0, false))
            .push(self.view_component_cards(sensors));

        content
            .push(
                Scrollable::new(additional_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .class(ScrollableStyle::Standard),
            )
            .into()
    }

    fn view_power_summary<'a>(&'a self, sensors: &'a HashMap<String, SensorState>) -> Element<'a, Message, AppTheme> {
        let power_value = format!(
            "{:.1}",
            sensors
                .get(TotalData::table_name_static())
                .and_then(|c| c.get_latest_reading())
                .and_then(|data| data.total_power_watts())
                .unwrap_or(0.0)
        );
        let power_unit = "W";

        let title = Text::new("Total Power Consumption")
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .class(TextStyle::Subtitle);

        let power_display = Row::new()
            .align_y(Alignment::End)
            .spacing(4)
            .push(
                Text::new(power_value)
                    .size(FONT_SIZE_HUGE)
                    .font(FONT_BOLD)
                    .class(TextStyle::Primary),
            )
            .push(Text::new(power_unit).size(FONT_SIZE_TITLE).class(TextStyle::Muted));

        let content = Column::new()
            .spacing(SPACING_MEDIUM)
            .align_x(Alignment::Center)
            .push(title)
            .push(power_display);

        Container::new(content)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .class(ContainerStyle::PowerCard)
            .into()
    }

    fn view_component_cards<'a>(&'a self, sensors: &'a HashMap<String, SensorState>) -> Element<'a, Message, AppTheme> {
        let mut column = Column::new().spacing(SPACING_LARGE).width(Length::Fill);

        let mut sensors: Vec<(&String, &SensorState)> = sensors
            .iter()
            .filter(|(table_name, _)| *table_name != TotalData::table_name_static())
            .collect();

        fn priority(name: &str) -> usize {
            let lower = name.to_lowercase();
            if lower.contains("cpu") {
                0
            } else if lower.contains("gpu") {
                1
            } else if lower.contains("ram") {
                2
            } else if lower.contains("disk") {
                3
            } else if lower.contains("network") {
                4
            } else {
                5
            }
        }

        sensors.sort_by_key(|(name, _)| (priority(name.as_str()), *name));

        let mut row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
        let mut items_in_row = 0usize;

        for (i, (_, sensor)) in sensors.into_iter().enumerate() {
            let card = sensor.chart_card(None, 200.0, true);

            row = row.push(card);
            items_in_row += 1;

            if i % 2 == 1 {
                column = column.push(row);
                row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
                items_in_row = 0;
            }
        }

        if items_in_row > 0 {
            column = column.push(row);
        }

        Container::new(column)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::Card)
            .into()
    }

    fn chart_or_placeholder<'a>(
        &'a self,
        sensors: &'a HashMap<String, SensorState>,
        title: Option<&'static str>,
        table_name: &str,
        height: f32,
        show_usage: bool,
    ) -> Element<'a, Message, AppTheme> {
        sensors
            .get(table_name)
            .map(|c| c.chart_card(title, height, show_usage))
            .unwrap_or_else(|| {
                Text::new("No data available")
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted)
                    .into()
            })
    }
}
