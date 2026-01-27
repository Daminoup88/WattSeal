use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use chrono::{DateTime, Utc};
use common::{CPUData, DatabaseEntry, SensorData, TotalData};
use iced::{
    Alignment, Color, Element, Length, Padding, Task,
    alignment::{Horizontal, Vertical},
    time::Duration,
    widget::{Column, Container, Row, Text},
};

use crate::{
    components::{
        self,
        chart::{LineType, SensorChart},
    },
    message::Message,
    styles::{
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HUGE, FONT_SIZE_SUBTITLE, FONT_SIZE_TITLE, PADDING_LARGE,
            SPACING_LARGE, SPACING_MEDIUM, SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

#[derive(Default)]
pub enum MetricType {
    #[default]
    Power,
    Usage,
}

impl MetricType {
    pub fn label(&self) -> &'static str {
        match self {
            MetricType::Power => "Power",
            MetricType::Usage => "Usage",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            MetricType::Power => "W",
            MetricType::Usage => "%",
        }
    }

    pub fn legend(&self, component_name: &str) -> String {
        format!("{} {}", component_name, self.label())
    }
}

#[derive(Default, Clone)]
pub enum TimeRange {
    #[default]
    LastMinute = 60,
    LastHour = 3600,
    Last24Hours = 86400,
}

pub struct ComponentState<'a> {
    name: String,
    sensor_type: String,
    latest_reading: Option<SensorData>,
    power_history: Rc<RefCell<VecDeque<(DateTime<Utc>, f32)>>>,
    usage_history: Rc<RefCell<VecDeque<(DateTime<Utc>, f32)>>>,
    chart: SensorChart<'a>,
    line_type: LineType,
    time_range: TimeRange,
    metric_type: MetricType,
    show_in_total: bool,
}

impl<'a> ComponentState<'a> {
    fn new(name: String, sensor_type: String, theme: AppTheme) -> Self {
        let chart = SensorChart::new(theme);
        let mut state = Self {
            name,
            sensor_type,
            latest_reading: None,
            chart,
            power_history: Rc::new(RefCell::new(VecDeque::new())),
            usage_history: Rc::new(RefCell::new(VecDeque::new())),
            time_range: TimeRange::default(),
            metric_type: MetricType::default(),
            show_in_total: false,
            line_type: LineType::default(),
        };
        state.update_metric_type(MetricType::default());
        state.update_time_range(TimeRange::default());
        state
    }

    fn push_data(&mut self, timestamp: DateTime<Utc>, data: &SensorData) {
        self.latest_reading = Some(data.clone());

        let power = data.total_power_watts();
        let usage = data.usage_percent();
        if let Some(p) = power {
            if let Ok(mut history) = self.power_history.try_borrow_mut() {
                history.push_back((timestamp, p as f32));
            }
        }
        if let Some(u) = usage {
            if let Ok(mut history) = self.usage_history.try_borrow_mut() {
                history.push_back((timestamp, u as f32));
            }
        }

        let cutoff = timestamp - chrono::Duration::seconds(self.time_range.clone() as i64);
        if let Ok(mut history) = self.usage_history.try_borrow_mut() {
            while let Some(&(ts, _)) = history.front() {
                if ts < cutoff {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        self.chart.refresh_cache();
    }

    fn update_time_range(&mut self, time_range: TimeRange) {
        self.time_range = time_range;
        let label = "Time";
        let unit = match self.time_range {
            TimeRange::LastMinute => "s",
            TimeRange::LastHour => "min",
            TimeRange::Last24Hours => "h",
        };
        self.chart.set_x_axis_label_and_unit(label, unit);
        self.chart
            .set_x_range(chrono::Duration::seconds(self.time_range.clone() as i64));
        // TODO: fetch data
    }

    fn update_metric_type(&mut self, metric_type: MetricType) {
        self.metric_type = metric_type;
        self.chart.clear();
        let (label, unit) = match self.metric_type {
            MetricType::Power => ("Power", "W"),
            MetricType::Usage => ("Usage", "%"),
        };
        let legend = self.metric_type.legend(&self.sensor_type);
        self.chart.add_series(&legend, self.line_type);
        self.chart.set_y_axis_label_and_unit(label, unit);
        self.chart.set_data(
            &legend,
            match self.metric_type {
                MetricType::Power => self.power_history.clone(),
                MetricType::Usage => self.usage_history.clone(),
            },
        );
    }

    fn update_theme(&mut self, theme: AppTheme) {
        self.chart.update_style(theme);
    }

    fn chart_card<'b>(&'b self, title: &'b str) -> Element<'b, Message, AppTheme> {
        let title = Text::new(title)
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .class(TextStyle::Subtitle);

        let chart_container = Container::new(self.chart.view())
            .width(Length::Fill)
            .height(Length::Fill);

        let content = Column::new()
            .spacing(SPACING_MEDIUM)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(title)
            .push(chart_container);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::Card)
            .into()
    }
}

pub struct DashboardPage<'a> {
    components: HashMap<String, ComponentState<'a>>,
}

impl<'a> DashboardPage<'a> {
    pub fn new(theme: AppTheme, components: Vec<String>) -> (Self, Task<Message>) {
        let components = components
            .into_iter()
            .map(|name| {
                let sensor_type = SensorData::get_matching_sensor_data(name.as_str())
                    .map(|data| data.sensor_type().to_string())
                    .unwrap_or(name.clone());
                (sensor_type.clone(), ComponentState::new(name, sensor_type, theme))
            })
            .collect();
        (Self { components }, Task::none())
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        for component in self.components.values_mut() {
            component.update_theme(theme);
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateChartData(data) => {
                for (timestamp, sensor) in data.iter() {
                    if let Some(component) = self.components.get_mut(sensor.sensor_type()) {
                        component.push_data(*timestamp, sensor);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let chart_card = self
            .components
            .get("Total")
            .map(|c| c.chart_card("Total Power Over Time"))
            .unwrap_or_else(|| {
                Text::new("No data available")
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted)
                    .into()
            });
        let content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.view_power_summary())
            .push(self.view_component_cards())
            .push(chart_card);

        Container::new(content).width(Length::Fill).height(Length::Fill).into()
    }

    fn view_power_summary(&self) -> Element<'_, Message, AppTheme> {
        let power_value = format!(
            "{:.1}",
            self.components
                .get("Total")
                .and_then(|c| c.latest_reading.as_ref())
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
        let mut row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
        let mut items_in_row = 0;

        for (i, (name, component)) in self.components.iter().filter(|(name, _)| *name != "Total").enumerate() {
            let power = component
                .latest_reading
                .as_ref()
                .and_then(|data| data.total_power_watts());
            let usage = component.latest_reading.as_ref().and_then(|data| data.usage_percent());
            let card = self.component_snapshot_card(name, power, usage);

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

    fn component_snapshot_card(
        &self,
        name: &str,
        power: Option<f64>,
        usage: Option<f64>,
    ) -> Element<'_, Message, AppTheme> {
        let name_owned = name.to_string();
        let power_text = power
            .map(|p| format!("{:.1} W", p))
            .unwrap_or_else(|| "N/A".to_string());

        let usage_text = usage.map(|u| format!("{:.1}%", u)).unwrap_or_else(|| "N/A".to_string());

        let title = Text::new(name_owned).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD);

        let power_style = if power.is_some() {
            TextStyle::Primary
        } else {
            TextStyle::Muted
        };

        let usage_style = if usage.is_some() {
            TextStyle::Success
        } else {
            TextStyle::Muted
        };

        let power_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(Text::new("Power:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
            .push(
                Text::new(power_text)
                    .size(FONT_SIZE_BODY)
                    .font(FONT_BOLD)
                    .class(power_style),
            );

        let usage_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(Text::new("Usage:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
            .push(
                Text::new(usage_text)
                    .size(FONT_SIZE_BODY)
                    .font(FONT_BOLD)
                    .class(usage_style),
            );

        let content = Column::new()
            .spacing(SPACING_LARGE)
            .push(title)
            .push(power_row)
            .push(usage_row);

        Container::new(content)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::ComponentCard)
            .into()
    }

    fn chart_card<'b>(&'b self, title: &'b str) -> Element<'b, Message, AppTheme> {
        let title = Text::new(title)
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .class(TextStyle::Subtitle);

        let chart = self.components.get("Total").map(|c| c.chart.view()).unwrap_or_else(|| {
            Text::new("No data available")
                .size(FONT_SIZE_BODY)
                .class(TextStyle::Muted)
                .into()
        });

        let chart_container = Container::new(chart).width(Length::Fill).height(Length::Fill);

        let content = Column::new()
            .spacing(SPACING_MEDIUM)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(title)
            .push(chart_container);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::Card)
            .into()
    }
}
