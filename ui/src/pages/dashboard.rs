use std::collections::HashMap;

use common::{AllTimeData, DatabaseEntry, ProcessData, TotalData};
use iced::{
    Alignment, Element, Length, Padding,
    alignment::{Horizontal, Vertical},
    widget::{Column, Container, Row, Scrollable, Text, grid, rule},
};

use crate::{
    components::{helpers::no_data_placeholder, sensor_state::SensorState},
    message::Message,
    styles::{
        container::ContainerStyle,
        scrollable::ScrollableStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_LARGE, FONT_SIZE_SUBTITLE, FONT_SIZE_TITLE, PADDING_LARGE,
            PADDING_MEDIUM, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL, SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{self, all_time, current_power_consumption, emissions},
    types::AppLanguage,
};

const MAX_COMPONENT_CARD_WIDTH: f32 = 600.0;
const CARBON_INTENSITY_G_PER_KWH: f64 = 475.0;

pub struct DashboardPage;

impl DashboardPage {
    pub fn view<'a>(
        &'a self,
        sensors: &'a HashMap<String, SensorState>,
        all_time_data: &'a AllTimeData,
        language: AppLanguage,
    ) -> Element<'a, Message, AppTheme> {
        let content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.view_power_summary(sensors, all_time_data, language));

        let additional_content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.chart_or_placeholder(sensors, None, TotalData::table_name_static(), 300.0, false, language))
            .push(self.view_process_summary(sensors, language))
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

    fn view_power_summary<'a>(
        &'a self,
        sensors: &'a HashMap<String, SensorState>,
        all_time_data: &'a AllTimeData,
        language: AppLanguage,
    ) -> Element<'a, Message, AppTheme> {
        let power_value = format!(
            "{:.1}",
            sensors
                .get(TotalData::table_name_static())
                .and_then(|c| c.get_latest_reading())
                .and_then(|data| data.total_power_watts())
                .unwrap_or(0.0)
        );

        let main = Column::new()
            .width(Length::FillPortion(1))
            .spacing(SPACING_SMALL)
            .align_x(Alignment::Center)
            .push(
                Text::new(current_power_consumption(language))
                    .size(FONT_SIZE_SUBTITLE)
                    .font(FONT_BOLD)
                    .class(TextStyle::Subtitle),
            )
            .push(
                Row::new()
                    .align_y(Alignment::End)
                    .spacing(4)
                    .push(
                        Text::new(power_value)
                            .size(FONT_SIZE_LARGE)
                            .font(FONT_BOLD)
                            .class(TextStyle::Primary),
                    )
                    .push(Text::new("W").size(FONT_SIZE_TITLE).class(TextStyle::Muted)),
            );

        let total_energy_wh = all_time_data.total_energy_wh;
        let carbon_grams = wh_to_co2_grams(total_energy_wh);

        let side = Column::new()
            .width(Length::FillPortion(1))
            .spacing(SPACING_SMALL)
            .align_x(Alignment::Center)
            .push(metric_tile(
                all_time(language),
                format_wh(total_energy_wh),
                "Wh",
                TextStyle::Secondary,
            ))
            .push(metric_tile(
                emissions(language),
                format_grams(carbon_grams),
                "g CO₂",
                TextStyle::Tertiary,
            ));

        let content = Row::new()
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .push(main)
            .push(rule::vertical(1))
            .push(side);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(Padding::from(PADDING_MEDIUM))
            .class(ContainerStyle::PowerCard)
            .into()
    }

    fn view_process_summary<'a>(
        &'a self,
        sensors: &'a HashMap<String, SensorState>,
        language: AppLanguage,
    ) -> Element<'a, Message, AppTheme> {
        let process_data = sensors.get(ProcessData::table_name_static());

        if let Some(process_card) = process_data.and_then(|p| Some(p.sensor_visual_card(None, 300.0, false))) {
            process_card
        } else {
            no_data_placeholder(language)
        }
    }

    fn view_component_cards<'a>(&'a self, sensors: &'a HashMap<String, SensorState>) -> Element<'a, Message, AppTheme> {
        let mut column = Column::new().spacing(SPACING_LARGE).width(Length::Fill);

        let mut sensors: Vec<(&String, &SensorState)> = sensors
            .iter()
            .filter(|(table_name, _)| {
                *table_name != TotalData::table_name_static() && *table_name != ProcessData::table_name_static()
            })
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
            let card = sensor.sensor_visual_card(None, 200.0, true);

            row = row.push(card);
            items_in_row += 1;

            if i % 2 == 1 {
                column = column.push(row);
                row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
                items_in_row = 0;
            }
        }

        if items_in_row % 2 == 1 {
            row = row.push(Row::new().spacing(SPACING_LARGE).width(Length::Fill));
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
        language: AppLanguage,
    ) -> Element<'a, Message, AppTheme> {
        sensors
            .get(table_name)
            .map(|c| c.sensor_visual_card(title, height, show_usage))
            .unwrap_or_else(|| no_data_placeholder(language))
    }
}

fn metric_tile<'a>(
    label: &'a str,
    value: String,
    unit: &'a str,
    value_style: TextStyle,
) -> Element<'a, Message, AppTheme> {
    let value_row = Row::new()
        .spacing(4)
        .align_y(Alignment::End)
        .push(
            Text::new(value)
                .size(FONT_SIZE_SUBTITLE)
                .font(FONT_BOLD)
                .class(value_style),
        )
        .push(Text::new(unit).size(FONT_SIZE_BODY).class(TextStyle::Muted));

    Container::new(
        Column::new()
            .padding(Padding::from(PADDING_MEDIUM))
            .spacing(2)
            .align_x(Alignment::Center)
            .push(
                Text::new(label)
                    .size(FONT_SIZE_BODY)
                    .font(FONT_BOLD)
                    .class(TextStyle::Subtitle),
            )
            .push(value_row),
    )
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}

fn wh_to_co2_grams(energy_wh: f64) -> f64 {
    (energy_wh / 1000.0) * CARBON_INTENSITY_G_PER_KWH
}

fn format_wh(energy_wh: f64) -> String {
    format!("{:.1}", energy_wh.max(0.0))
}

fn format_grams(co2_grams: f64) -> String {
    format!("{:.1}", co2_grams.max(0.0))
}
