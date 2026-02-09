use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use chrono::{DateTime, Local, Timelike};
use common::SensorData;
use iced::{
    Alignment, Element, Length, Padding, Task,
    widget::{Column, Container, Row, Text, button, pick_list},
};

use crate::{
    components::chart::{LineType, SensorChart},
    message::Message,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        picklist::PickListStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_SUBTITLE, PADDING_LARGE, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL,
            SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    types::{MetricType, TimeRange},
};

pub struct ComponentState<'a> {
    table_name: String,
    sensor_type: String,
    latest_reading: Option<SensorData>,
    power_history: Rc<RefCell<VecDeque<(DateTime<Local>, f32)>>>,
    usage_history: Rc<RefCell<VecDeque<(DateTime<Local>, f32)>>>,
    chart: SensorChart<'a>,
    line_type: LineType,
    time_range: TimeRange,
    metric_type: MetricType,
    show_in_total: bool,
}

impl<'a> ComponentState<'a> {
    pub fn new(name: String, sensor_type: String, theme: AppTheme) -> Self {
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
        let _ = state.apply_time_range(TimeRange::default());
        state
    }

    pub fn name(&self) -> &str {
        &self.sensor_type
    }

    fn append_to_history(&self, timestamp: DateTime<Local>, data: &SensorData) {
        if let Some(power) = data.total_power_watts() {
            if let Ok(mut h) = self.power_history.try_borrow_mut() {
                h.push_back((timestamp, power as f32));
            }
        }
        if let Some(usage) = data.usage_percent() {
            if let Ok(mut h) = self.usage_history.try_borrow_mut() {
                h.push_back((timestamp, usage as f32));
            }
        }
    }

    fn prune_before(&self, cutoff: DateTime<Local>) {
        for history in [&self.power_history, &self.usage_history] {
            if let Ok(mut h) = history.try_borrow_mut() {
                while h.front().is_some_and(|&(ts, _)| ts < cutoff) {
                    h.pop_front();
                }
            }
        }
    }

    pub fn push_data(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        self.latest_reading = Some(data.clone());

        if !self.time_range.is_real_time() {
            return;
        }

        self.append_to_history(timestamp, data);
        self.prune_before(timestamp - self.time_range.duration_seconds());
        self.chart.refresh_cache();
    }

    pub fn push_history(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        self.append_to_history(timestamp, data);
    }

    pub fn load_history_batch(&mut self, data: &[(DateTime<Local>, SensorData)]) {
        for (timestamp, sensor) in data {
            self.push_history(*timestamp, sensor);
        }
        self.refresh_chart();
    }

    fn apply_time_range(&mut self, time_range: TimeRange) -> Task<Message> {
        self.time_range = time_range;
        let label = "Time";
        let unit = self.time_range.unit();
        self.line_type = match self.time_range {
            TimeRange::LastMinute => LineType::Line,
            _ => LineType::Step,
        };
        self.chart.set_all_line_types(self.line_type);
        self.chart.set_x_axis_label_and_unit(label, unit);
        self.chart.set_x_range(self.time_range.duration_seconds());
        self.clear_data();
        Task::done(Message::FetchChartData(
            self.table_name.clone(),
            self.time_range.clone(),
        ))
    }

    pub fn update_time_range(&mut self, time_range: TimeRange) -> Task<Message> {
        if self.time_range == time_range {
            return Task::none();
        }
        self.apply_time_range(time_range)
    }

    pub fn switch_metric_type(&mut self) {
        self.update_metric_type(self.metric_type.toggled());
    }

    pub fn get_latest_reading(&self) -> Option<&SensorData> {
        self.latest_reading.as_ref()
    }

    fn update_metric_type(&mut self, metric_type: MetricType) {
        self.metric_type = metric_type;
        self.chart.clear_all();
        let legend = self.metric_type.legend(&self.sensor_type);
        self.chart
            .add_series(&legend, self.line_type, Some(self.metric_type as usize));
        self.chart
            .set_y_axis_label_and_unit(self.metric_type.label(), self.metric_type.unit());
        self.chart.set_data(
            &legend,
            match self.metric_type {
                MetricType::Power => self.power_history.clone(),
                MetricType::Usage => self.usage_history.clone(),
            },
        );
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
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

    pub fn refresh_chart(&mut self) {
        self.chart.refresh_cache();
    }

    pub fn chart_card<'b>(
        &'b self,
        title: Option<&'b str>,
        height: f32,
        show_usage: bool,
    ) -> Element<'b, Message, AppTheme> {
        let chart_container = Container::new(self.chart.view(height)).width(Length::Fill);

        let power = self.latest_reading.as_ref().and_then(|data| data.total_power_watts());
        let power_text = power
            .map(|p| format!("{:.1} W", p))
            .unwrap_or_else(|| "N/A".to_string());

        let title = Text::new(title.unwrap_or(&self.sensor_type))
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD);

        let time_range_selector = pick_list(
            [TimeRange::LastMinute, TimeRange::LastHour, TimeRange::Last24Hours],
            Some(self.time_range.clone()),
            |tr| Message::ChangeChartTimeRange(self.table_name.clone(), tr),
        )
        .class(PickListStyle::TimeRange);

        let power_style = if power.is_some() {
            TextStyle::Primary
        } else {
            TextStyle::Muted
        };

        let first_row = Row::new()
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(title);

        let mut second_row_left = Column::new()
            .spacing(SPACING_SMALL)
            .push(
                Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(Text::new("Power:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
                    .push(
                        Text::new(power_text)
                            .size(FONT_SIZE_BODY)
                            .class(power_style)
                            .font(FONT_BOLD),
                    ),
            )
            .width(Length::Fill);

        let mut second_row_right = Row::new()
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(time_range_selector);

        if show_usage {
            let usage = self.latest_reading.as_ref().and_then(|data| data.usage_percent());
            let usage_text = usage
                .map(|u| format!("{:.1} %", u))
                .unwrap_or_else(|| "N/A".to_string());
            let usage_style = if usage.is_some() {
                TextStyle::Success
            } else {
                TextStyle::Muted
            };
            let metric_type_button = button(Text::new(self.metric_type.toggled().to_string()).size(FONT_SIZE_BODY))
                .on_press(Message::ChangeChartMetricType(self.table_name.clone()))
                .class(ButtonStyle::Toggle(self.metric_type.into()));

            second_row_left = second_row_left.push(
                Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(Text::new("Usage:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
                    .push(
                        Text::new(usage_text)
                            .size(FONT_SIZE_BODY)
                            .class(usage_style)
                            .font(FONT_BOLD),
                    ),
            );
            second_row_right = second_row_right.push(metric_type_button);
        }

        let content = Column::new().spacing(SPACING_LARGE).push(first_row).push(
            Column::new()
                .push(
                    Row::new()
                        .spacing(SPACING_LARGE)
                        .align_y(Alignment::Center)
                        .push(second_row_left)
                        .push(second_row_right),
                )
                .push(chart_container),
        );

        Container::new(content)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::ComponentCard)
            .into()
    }
}
