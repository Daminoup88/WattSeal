use std::collections::HashMap;

use common::{DatabaseEntry, TotalData, generic_name_for_table};
use iced::{
    Alignment, Element, Length, Padding, Task,
    alignment::{Horizontal, Vertical},
    widget::{Column, Container, Row, Scrollable, Text},
};

use crate::{
    components::component_state::ComponentState,
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
    types::TimeRange,
};

pub struct DashboardPage<'a> {
    components: HashMap<String, ComponentState<'a>>,
}

impl<'a> DashboardPage<'a> {
    pub fn new(theme: AppTheme, components: Vec<String>) -> (Self, Task<Message>) {
        let components = components
            .into_iter()
            .map(|table_name| {
                let sensor_type = generic_name_for_table(table_name.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(table_name.clone());
                (table_name.clone(), ComponentState::new(table_name, sensor_type, theme))
            })
            .collect();
        (
            Self { components },
            Task::done(Message::FetchAllChartsData(TimeRange::default())),
        )
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        for component in self.components.values_mut() {
            component.update_theme(theme);
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateChartData(data) => {
                for (timestamp, sensor) in data.iter() {
                    if let Some(component) = self.components.get_mut(sensor.table_name()) {
                        component.push_data(*timestamp, sensor);
                    }
                }
            }
            Message::ChangeChartMetricType(table_name, metric_type) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    component.set_metric_type(metric_type);
                }
            }
            Message::ChangeChartTimeRange(sensor_type, time_range) => {
                if let Some(component) = self.components.get_mut(&sensor_type) {
                    return component.update_time_range(time_range);
                }
            }
            Message::ReplaceChartData(table_name, data) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    component.load_history_batch(&data);
                }
            }
            _ => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.view_power_summary());

        let additional_content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.chart_or_placeholder(None, TotalData::table_name_static(), 300.0, false))
            .push(self.view_component_cards());

        content
            .push(
                Scrollable::new(additional_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .class(ScrollableStyle::Standard),
            )
            .into()
    }

    fn view_power_summary(&self) -> Element<'_, Message, AppTheme> {
        let power_value = format!(
            "{:.1}",
            self.components
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

    fn view_component_cards(&self) -> Element<'_, Message, AppTheme> {
        let mut column = Column::new().spacing(SPACING_LARGE).width(Length::Fill);

        let mut components: Vec<(&String, &ComponentState<'a>)> = self
            .components
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

        components.sort_by_key(|(name, _)| (priority(name.as_str()), *name));

        let mut row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
        let mut items_in_row = 0usize;

        for (i, (_, component)) in components.into_iter().enumerate() {
            let card = component.chart_card(None, 200.0, true);

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

    fn chart_or_placeholder<'b>(
        &'b self,
        title: Option<&'b str>,
        table_name: &str,
        height: f32,
        show_usage: bool,
    ) -> Element<'b, Message, AppTheme> {
        self.components
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
