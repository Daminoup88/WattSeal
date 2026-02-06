use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet, VecDeque},
    fmt::Display,
    rc::Rc,
};

use chrono::{DateTime, TimeDelta, Timelike, Utc};
use common::{CPUData, DatabaseEntry, SensorData, TotalData};
use iced::{
    Alignment, Color, Element, Length, Padding, Renderer, Task, Theme,
    advanced::graphics::text::cosmic_text::skrifa::raw::tables::aat::class,
    alignment::{Horizontal, Vertical},
    time::Duration,
    widget::{
        Column, Container, PickList, Row, Scrollable, Text, Toggler,
        button::{self, Button},
        pick_list, table,
    },
};

use crate::{
    components::{
        self,
        chart::{LineType, SensorChart},
    },
    message::Message,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        picklist::PickListStyle,
        scrollable::ScrollableStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HUGE, FONT_SIZE_SUBTITLE, FONT_SIZE_TITLE, PADDING_LARGE,
            SPACING_LARGE, SPACING_MEDIUM, SPACING_XLARGE,
        },
        text::TextStyle,
        toggler::TogglerStyle,
    },
    themes::AppTheme,
    types::TimeRange,
};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

#[derive(Default, PartialEq)]
pub enum MetricType {
    #[default]
    Power,
    Usage,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Power => write!(f, "Power"),
            MetricType::Usage => write!(f, "Usage"),
        }
    }
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

pub struct ComponentState<'a> {
    table_name: String,
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
            table_name: name,
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
        let _ = state.update_time_range(TimeRange::default());
        state
    }

    fn push_data(&mut self, timestamp: DateTime<Utc>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);

        self.latest_reading = Some(data.clone());

        if !self.time_range.is_real_time() {
            return;
        }

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

        let cutoff = timestamp - self.time_range.duration_seconds();

        let prune_history = |history: &Rc<RefCell<VecDeque<(DateTime<Utc>, f32)>>>| {
            if let Ok(mut h) = history.try_borrow_mut() {
                while let Some(&(ts, _)) = h.front() {
                    if ts < cutoff {
                        h.pop_front();
                    } else {
                        break;
                    }
                }
            }
        };

        prune_history(&self.power_history);
        prune_history(&self.usage_history);

        self.chart.refresh_cache();
    }

    fn push_history(&mut self, timestamp: DateTime<Utc>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);

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
    }

    fn update_time_range(&mut self, time_range: TimeRange) -> Task<Message> {
        if self.time_range == time_range {
            return Task::none();
        }
        self.time_range = time_range;
        let label = "Time";
        let unit = self.time_range.unit();
        let line_type = match self.time_range {
            TimeRange::LastMinute => LineType::Line,
            _ => LineType::Step,
        };
        self.chart.set_all_line_types(line_type);
        self.chart.set_x_axis_label_and_unit(label, unit);
        self.chart.set_x_range(self.time_range.duration_seconds());
        self.clear_data();
        Task::done(Message::FetchChartData(
            self.table_name.clone(),
            self.time_range.clone(),
        ))
    }

    fn switch_metric_type(&mut self) {
        let new_metric = match self.metric_type {
            MetricType::Power => MetricType::Usage,
            MetricType::Usage => MetricType::Power,
        };
        self.update_metric_type(new_metric);
    }

    fn update_metric_type(&mut self, metric_type: MetricType) {
        self.metric_type = metric_type;
        self.chart.clear_all();
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

    fn clear_data(&mut self) {
        if let Ok(mut power_history) = self.power_history.try_borrow_mut() {
            power_history.clear();
        }
        if let Ok(mut usage_history) = self.usage_history.try_borrow_mut() {
            usage_history.clear();
        }
    }

    fn chart_card<'b>(
        &'b self,
        title: &'b str,
        height: f32,
        show_switch_metric: bool,
    ) -> Element<'b, Message, AppTheme> {
        let title = Text::new(title)
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .class(TextStyle::Subtitle)
            .width(Length::Fill);

        let time_range_selector: PickList<'_, _, _, _, _, AppTheme, Renderer> = pick_list(
            [TimeRange::LastMinute, TimeRange::LastHour, TimeRange::Last24Hours],
            Some(self.time_range.clone()),
            |tr| Message::ChangeChartTimeRange(self.table_name.clone(), tr),
        );

        let mut first_row = Row::new()
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(title)
            .push(time_range_selector);

        if show_switch_metric {
            let metric_type_button: Button<'_, _, AppTheme, Renderer> = iced::widget::button(
                Text::new(match self.metric_type {
                    MetricType::Power => MetricType::Usage.to_string(),
                    MetricType::Usage => MetricType::Power.to_string(),
                })
                .size(FONT_SIZE_BODY),
            )
            .on_press(Message::ChangeChartMetricType(self.table_name.clone()));

            first_row = first_row.push(metric_type_button);
        } else {
            first_row = first_row.push(Container::new(Text::new("")).width(Length::Fixed(90.0)));
        }

        let chart_container = Container::new(self.chart.view(height))
            .width(Length::Fill)
            .height(Length::Fill);

        let content = Column::new()
            .spacing(SPACING_MEDIUM)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(first_row)
            .push(chart_container);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::Card)
            .into()
    }

    fn snapshot_card(&self) -> Element<'_, Message, AppTheme> {
        let name_owned = &self.sensor_type;
        let power = self.latest_reading.as_ref().and_then(|data| data.total_power_watts());
        let usage = self.latest_reading.as_ref().and_then(|data| data.usage_percent());
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
}

pub struct DashboardPage<'a> {
    components: BTreeMap<String, ComponentState<'a>>,
}

impl<'a> DashboardPage<'a> {
    pub fn new(theme: AppTheme, components: Vec<String>) -> (Self, Task<Message>) {
        let components = components
            .into_iter()
            .map(|table_name| {
                let sensor_type = SensorData::get_matching_sensor_data(table_name.as_str())
                    .map(|data| data.sensor_type().to_string())
                    .unwrap_or(table_name.clone());
                (table_name.clone(), ComponentState::new(table_name, sensor_type, theme))
            })
            .collect();
        (Self { components }, Task::none())
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
            Message::ChangeChartMetricType(table_name) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    component.switch_metric_type();
                }
            }
            Message::ChangeChartTimeRange(sensor_type, time_range) => {
                if let Some(component) = self.components.get_mut(&sensor_type) {
                    return component.update_time_range(time_range);
                }
            }
            Message::ReplaceChartData(table_name, data) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    for (timestamp, sensor) in data.iter() {
                        component.push_history(*timestamp, sensor);
                    }
                    component.chart.refresh_cache();
                }
            }
            _ => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let chart_card = self
            .components
            .get(TotalData::table_name_static())
            .map(|c| c.chart_card("Total Power Over Time", 300.0, false))
            .unwrap_or_else(|| {
                Text::new("No data available")
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted)
                    .into()
            });

        let chart_card2 = self
            .components
            .get(CPUData::table_name_static())
            .map(|c| c.chart_card("CPU Power Over Time", 300.0, true))
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
            .push(self.view_power_summary());

        let additional_content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(chart_card)
            .push(chart_card2)
            .push(self.view_component_cards());

        // Container::new(content).width(Length::Fill).height(Length::Fill).into()
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

        for (i, (_, component)) in self
            .components
            .iter()
            .filter(|(table_name, _)| *table_name != TotalData::table_name_static())
            .enumerate()
        {
            let card = component.snapshot_card();

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
}
