use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use chrono::{DateTime, Local, Timelike};
use common::{MetricType, SecondaryValues, SensorData};
use iced::{
    Alignment, Element, Length, Padding, Task,
    widget::{Column, Container, Row, Space, Text, pick_list},
};

use crate::{
    components::chart::{LineType, SensorChart},
    message::Message,
    styles::{
        container::ContainerStyle,
        picklist::PickListStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_SUBTITLE, PADDING_LARGE, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL,
            SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    types::TimeRange,
};

const SNAPSHOT_AREA_HEIGHT: f32 = 34.0;

type HistoryRef = Rc<RefCell<VecDeque<(DateTime<Local>, f32)>>>;

pub struct ComponentState<'a> {
    table_name: String,
    sensor_type: String,
    latest_reading: Option<SensorData>,
    power_history: HistoryRef,
    secondary_histories: Vec<HistoryRef>,
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
            secondary_histories: Vec::new(),
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

    fn current_secondary_values(&self) -> Option<SecondaryValues> {
        self.latest_reading
            .as_ref()
            .and_then(|data| data.secondary_values().map(|metrics| metrics))
    }

    fn extend_secondary_histories(&mut self, required_count: usize) {
        while self.secondary_histories.len() < required_count {
            self.secondary_histories.push(Rc::new(RefCell::new(VecDeque::new())));
        }
    }

    fn append_to_history(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        if let Some(power) = data.total_power_watts() {
            if let Ok(mut history) = self.power_history.try_borrow_mut() {
                history.push_back((timestamp, power as f32));
            }
        }

        if let Some(secondary_values) = data.secondary_values() {
            self.extend_secondary_histories(secondary_values.values.len());
            for (index, labeled_value) in secondary_values.values.into_iter().enumerate() {
                if let Some(value) = labeled_value.value
                    && let Ok(mut history) = self.secondary_histories[index].try_borrow_mut()
                {
                    history.push_back((timestamp, value as f32));
                }
            }
        }
    }

    fn prune_before(&self, cutoff: DateTime<Local>) {
        prune_history(&self.power_history, cutoff);
        for history in &self.secondary_histories {
            prune_history(history, cutoff);
        }
    }

    fn available_metrics(&self) -> Vec<MetricType> {
        let mut metrics = vec![MetricType::Power];
        if let Some(secondary_values) = self.current_secondary_values() {
            metrics.push(secondary_values.metric_type);
        }
        metrics
    }

    pub fn push_data(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        self.latest_reading = Some(data.clone());

        if !self.time_range.is_real_time() {
            return;
        }

        self.append_to_history(timestamp, data);
        self.prune_before(timestamp - self.time_range.duration_seconds());
        self.refresh_chart();
    }

    pub fn push_to_history_only(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        self.append_to_history(timestamp, data);
    }

    pub fn load_history_batch(&mut self, data: &[(DateTime<Local>, SensorData)]) {
        for (timestamp, sensor) in data {
            self.push_to_history_only(*timestamp, sensor);
        }
        self.refresh_chart();
    }

    fn apply_time_range(&mut self, time_range: TimeRange) -> Task<Message> {
        self.time_range = time_range;
        self.line_type = match self.time_range {
            TimeRange::LastMinute => LineType::Line,
            _ => LineType::Step,
        };
        self.chart.set_all_line_types(self.line_type);
        self.chart.set_x_axis_label_and_unit("Time", self.time_range.unit());
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

    pub fn set_metric_type(&mut self, metric_type: MetricType) {
        self.update_metric_type(metric_type);
    }

    pub fn get_latest_reading(&self) -> Option<&SensorData> {
        self.latest_reading.as_ref()
    }

    fn update_metric_type(&mut self, metric_type: MetricType) {
        self.metric_type = metric_type;
        self.chart.clear_all();
        self.chart
            .set_y_axis_label_and_unit(self.metric_type.label(), self.metric_type.unit());
        match self.metric_type {
            MetricType::Power => {
                let legend = self.metric_type.legend(&self.sensor_type);
                self.chart
                    .add_series(&legend, self.line_type, Some(self.metric_type as usize));
                self.chart.set_data(&legend, self.power_history.clone());
            }
            _ => {
                if let Some(secondary_values) = self.current_secondary_values() {
                    for (index, labeled_value) in secondary_values.values.into_iter().enumerate() {
                        let legend = format!("{} {}", self.sensor_type, labeled_value.label);
                        self.chart
                            .add_series(&legend, self.line_type, Some(index.saturating_add(1)));
                        self.chart.set_data(&legend, self.secondary_histories[index].clone());
                    }
                }
            }
        }
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        self.chart.update_style(theme);
    }

    fn clear_data(&mut self) {
        if let Ok(mut power_history) = self.power_history.try_borrow_mut() {
            power_history.clear();
        }
        for history in &self.secondary_histories {
            if let Ok(mut borrowed) = history.try_borrow_mut() {
                borrowed.clear();
            }
        }
    }

    pub fn refresh_chart(&mut self) {
        self.chart.refresh_cache();
    }

    fn snapshot_row(&self) -> Option<Row<'static, Message, AppTheme>> {
        let secondary_values = self.current_secondary_values()?;

        let mut row = Column::new().spacing(SPACING_SMALL);

        for (index, value) in secondary_values.values.into_iter().enumerate() {
            if let Some(metric_value) = value.value {
                let value_style = match index % 3 {
                    0 => TextStyle::Secondary,
                    1 => TextStyle::Tertiary,
                    _ => TextStyle::Subtitle,
                };

                row = row.push(
                    Row::new()
                        .spacing(SPACING_MEDIUM)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(format!("{}:", value.label))
                                .size(FONT_SIZE_BODY)
                                .class(TextStyle::Muted),
                        )
                        .push(
                            Text::new(format!("{:.1} {}", metric_value, secondary_values.metric_type.unit()))
                                .size(FONT_SIZE_BODY)
                                .class(value_style)
                                .font(FONT_BOLD),
                        ),
                );
            }
        }

        Some(Row::new().align_y(Alignment::Center).push(row))
    }

    pub fn chart_card<'b>(
        &'b self,
        title: Option<&'b str>,
        height: f32,
        show_secondary: bool,
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
        .class(PickListStyle::TimeRange)
        .menu_class(PickListStyle::TimeRange);

        let power_style = if power.is_some() {
            TextStyle::Primary
        } else {
            TextStyle::Muted
        };

        let first_row = Row::new()
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(title);

        let power_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(Text::new("Power:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
            .push(
                Text::new(power_text)
                    .size(FONT_SIZE_BODY)
                    .class(power_style)
                    .font(FONT_BOLD),
            );

        let secondary_row_container = if let Some(row) = self.snapshot_row() {
            Container::new(row)
                .height(Length::Fixed(SNAPSHOT_AREA_HEIGHT))
                .align_y(Alignment::Center)
        } else {
            Container::new(Space::new()).height(Length::Fixed(SNAPSHOT_AREA_HEIGHT))
        };

        let second_row_left = Column::new()
            .spacing(SPACING_SMALL)
            .push(power_row)
            .push(secondary_row_container)
            .width(Length::Fill);

        let mut second_row_right = Row::new()
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(time_range_selector);

        if show_secondary {
            let metrics = self.available_metrics();
            if metrics.len() > 1 {
                let metric_selector = pick_list(metrics, Some(self.metric_type), |metric| {
                    Message::ChangeChartMetricType(self.table_name.clone(), metric)
                })
                .class(PickListStyle::TimeRange)
                .menu_class(PickListStyle::TimeRange);
                second_row_right = second_row_right.push(metric_selector);
            }
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

fn prune_history(history: &HistoryRef, cutoff: DateTime<Local>) {
    if let Ok(mut borrowed) = history.try_borrow_mut() {
        while borrowed.front().is_some_and(|&(ts, _)| ts < cutoff) {
            borrowed.pop_front();
        }
    }
}
