use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use chrono::{DateTime, Local, Timelike};
use common::{DatabaseEntry, MetricType, ProcessData, SecondaryValues, SensorData, TotalData, utils::bytes_to_mb};
use iced::{
    Alignment, ContentFit, Element, Length, Padding, Task,
    widget::{
        Column, Container, PickList, Row, Scrollable, Space, Text, image, pick_list,
        scrollable::{Direction, Scrollbar},
    },
};

use crate::{
    components::{
        chart::{LineType, SensorChart},
        helpers::text_widget,
    },
    message::Message,
    styles::{
        container::ContainerStyle,
        picklist::PickListStyle,
        scrollable::ScrollableStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_SMALL, FONT_SIZE_SUBTITLE, PADDING_LARGE, SPACING_LARGE,
            SPACING_MEDIUM, SPACING_SMALL, SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    types::TimeRange,
};

const SNAPSHOT_AREA_HEIGHT: f32 = 34.0;
const PROCESS_APP_WIDTH: f32 = 180.0;
const PROCESS_ICON_COLUMN_WIDTH: f32 = 24.0;
const PROCESS_ICON_SIZE: f32 = 16.0;
const PROCESS_POWER_WIDTH: f32 = 55.0;
const PROCESS_CPU_WIDTH: f32 = 48.0;
const PROCESS_GPU_WIDTH: f32 = 48.0;
const PROCESS_RAM_WIDTH: f32 = 55.0;
const PROCESS_DISK_READ_WIDTH: f32 = 80.0;
const PROCESS_DISK_WRITE_WIDTH: f32 = 80.0;

type HistoryRef = Rc<RefCell<VecDeque<(DateTime<Local>, f32)>>>;
type TooltipHistoryRef = Rc<RefCell<VecDeque<(DateTime<Local>, String, Vec<u8>)>>>;

struct PowerChartState {
    power_history: HistoryRef,
    chart: SensorChart,
    line_type: LineType,
}

impl PowerChartState {
    fn new(theme: AppTheme) -> Self {
        Self {
            power_history: Rc::new(RefCell::new(VecDeque::new())),
            chart: SensorChart::new(theme),
            line_type: LineType::default(),
        }
    }

    fn append_power(&self, timestamp: DateTime<Local>, power: f32) {
        if let Ok(mut h) = self.power_history.try_borrow_mut() {
            h.push_back((timestamp, power));
        }
    }

    fn prune_before(&self, cutoff: DateTime<Local>) {
        prune_history(&self.power_history, cutoff);
    }

    fn clear(&self) {
        if let Ok(mut h) = self.power_history.try_borrow_mut() {
            h.clear();
        }
    }
}

struct ComponentState {
    power_graph: PowerChartState,
    secondary_histories: Vec<HistoryRef>,
    show_in_total: bool,
    metric_type: MetricType,
}

impl ComponentState {
    fn new(theme: AppTheme, display_name: &str) -> Self {
        let mut power_graph = PowerChartState::new(theme);
        let metric_type = MetricType::default();
        power_graph
            .chart
            .set_y_axis_label_and_unit(metric_type.label(), metric_type.unit());
        let legend = metric_type.legend(display_name);
        power_graph
            .chart
            .add_series(&legend, power_graph.line_type, Some(metric_type as usize));
        power_graph.chart.set_data(&legend, power_graph.power_history.clone());
        Self {
            power_graph,
            secondary_histories: Vec::new(),
            show_in_total: true,
            metric_type,
        }
    }

    fn extend_secondary_histories(&mut self, count: usize) {
        while self.secondary_histories.len() < count {
            self.secondary_histories.push(Rc::new(RefCell::new(VecDeque::new())));
        }
    }

    fn append(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        if let Some(power) = data.total_power_watts() {
            self.power_graph.append_power(timestamp, power as f32);
        }
        if let Some(secondary_values) = data.secondary_values() {
            self.extend_secondary_histories(secondary_values.values.len());
            for (i, labeled_value) in secondary_values.values.into_iter().enumerate() {
                if let Some(value) = labeled_value.value
                    && let Ok(mut history) = self.secondary_histories[i].try_borrow_mut()
                {
                    history.push_back((timestamp, value as f32));
                }
            }
        }
    }

    fn prune_before(&self, cutoff: DateTime<Local>) {
        self.power_graph.prune_before(cutoff);
        for history in &self.secondary_histories {
            prune_history(history, cutoff);
        }
    }

    fn clear(&self) {
        self.power_graph.clear();
        for history in &self.secondary_histories {
            if let Ok(mut borrowed) = history.try_borrow_mut() {
                borrowed.clear();
            }
        }
    }

    fn update_metric_type(
        &mut self,
        metric_type: MetricType,
        display_name: &str,
        secondary_values: Option<SecondaryValues>,
    ) {
        self.metric_type = metric_type;
        self.power_graph.chart.clear_all();
        self.power_graph
            .chart
            .set_y_axis_label_and_unit(metric_type.label(), metric_type.unit());
        match metric_type {
            MetricType::Power => {
                let legend = metric_type.legend(display_name);
                self.power_graph
                    .chart
                    .add_series(&legend, self.power_graph.line_type, Some(metric_type as usize));
                self.power_graph
                    .chart
                    .set_data(&legend, self.power_graph.power_history.clone());
            }
            _ => {
                if let Some(secondary_values) = secondary_values {
                    for (i, labeled_value) in secondary_values.values.into_iter().enumerate() {
                        let legend = format!("{} {}", display_name, labeled_value.label);
                        self.power_graph.chart.add_series(
                            &legend,
                            self.power_graph.line_type,
                            Some(i.saturating_add(1)),
                        );
                        self.power_graph
                            .chart
                            .set_data(&legend, self.secondary_histories[i].clone());
                    }
                }
            }
        }
    }

    fn available_metrics(&self, latest: Option<&SensorData>) -> Vec<MetricType> {
        let mut metrics = vec![MetricType::default()];
        if let Some(secondary_values) = latest.and_then(|d| d.secondary_values()) {
            metrics.push(secondary_values.metric_type);
        }
        metrics
    }

    fn snapshot_row(&self, latest: Option<&SensorData>) -> Option<Row<'static, Message, AppTheme>> {
        let secondary_values = latest?.secondary_values()?;
        let mut col = Column::new().spacing(SPACING_SMALL);
        for (i, labeled_value) in secondary_values.values.into_iter().enumerate() {
            if let Some(value) = labeled_value.value {
                let style = match i % 3 {
                    0 => TextStyle::Secondary,
                    1 => TextStyle::Tertiary,
                    _ => TextStyle::Subtitle,
                };
                col = col.push(
                    Row::new()
                        .spacing(SPACING_MEDIUM)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(format!("{}:", labeled_value.label))
                                .size(FONT_SIZE_BODY)
                                .class(TextStyle::Muted),
                        )
                        .push(
                            Text::new(format!("{:.1} {}", value, secondary_values.metric_type.unit()))
                                .size(FONT_SIZE_BODY)
                                .class(style)
                                .font(FONT_BOLD),
                        ),
                );
            }
        }
        Some(Row::new().align_y(Alignment::Center).push(col))
    }
}

struct TotalState {
    power_graph: PowerChartState,
    tooltip_history: TooltipHistoryRef,
}

impl TotalState {
    fn new(theme: AppTheme, display_name: &str) -> Self {
        let mut power_graph = PowerChartState::new(theme);
        let metric_type = MetricType::default();
        power_graph
            .chart
            .set_y_axis_label_and_unit(metric_type.label(), metric_type.unit());
        let legend = metric_type.legend(display_name);
        power_graph
            .chart
            .add_series(&legend, power_graph.line_type, Some(metric_type as usize));
        power_graph.chart.set_data(&legend, power_graph.power_history.clone());
        Self {
            power_graph,
            tooltip_history: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    fn append(&self, timestamp: DateTime<Local>, data: &SensorData) {
        if let Some(power) = data.total_power_watts() {
            self.power_graph.append_power(timestamp, power as f32);
        }
    }

    fn prune_before(&self, cutoff: DateTime<Local>) {
        self.power_graph.prune_before(cutoff);
    }

    fn clear(&self) {
        self.power_graph.clear();
    }
}

struct ProcessesState {
    top_processes: Vec<ProcessData>,
}

impl ProcessesState {
    fn new() -> Self {
        Self {
            top_processes: Vec::new(),
        }
    }

    fn update_from_snapshot(&mut self, processes: &[ProcessData]) {
        self.top_processes = processes.into();
    }

    fn clear(&mut self) {
        self.top_processes.clear();
    }
}

enum SensorCategory {
    Component(ComponentState),
    Total(TotalState),
    Processes(ProcessesState),
}

pub struct SensorState {
    table_name: String,
    display_name: String,
    sensor_category: SensorCategory,
    latest_reading: Option<SensorData>,
    time_range: TimeRange,
}

impl SensorState {
    pub fn new(table_name: String, display_name: String, theme: AppTheme) -> Self {
        let sensor_category = if table_name == TotalData::table_name_static() {
            SensorCategory::Total(TotalState::new(theme, &display_name))
        } else if table_name == ProcessData::table_name_static() {
            SensorCategory::Processes(ProcessesState::new())
        } else {
            SensorCategory::Component(ComponentState::new(theme, &display_name))
        };

        let mut state = Self {
            table_name,
            display_name,
            sensor_category,
            latest_reading: None,
            time_range: TimeRange::default(),
        };
        let _ = state.apply_time_range(TimeRange::default());
        state
    }

    pub fn name(&self) -> &str {
        &self.display_name
    }

    pub fn get_latest_reading(&self) -> Option<&SensorData> {
        self.latest_reading.as_ref()
    }

    fn apply_time_range(&mut self, time_range: TimeRange) -> Task<Message> {
        self.time_range = time_range;
        let line_type = match &self.time_range {
            TimeRange::LastMinute => LineType::Line,
            _ => LineType::Step,
        };
        let unit = self.time_range.unit();
        let x_label = "Time";
        let duration = self.time_range.duration_seconds();
        match &mut self.sensor_category {
            SensorCategory::Component(s) => {
                s.power_graph.line_type = line_type;
                s.power_graph.chart.set_all_line_types(line_type);
                s.power_graph.chart.set_x_axis_label_and_unit(x_label, unit);
                s.power_graph.chart.set_x_range(duration);
            }
            SensorCategory::Total(s) => {
                s.power_graph.line_type = line_type;
                s.power_graph.chart.set_all_line_types(line_type);
                s.power_graph.chart.set_x_axis_label_and_unit(x_label, unit);
                s.power_graph.chart.set_x_range(duration);
            }
            SensorCategory::Processes(_) => {}
        }
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
        if let SensorCategory::Component(state) = &mut self.sensor_category {
            let secondary_values = self.latest_reading.as_ref().and_then(|d| d.secondary_values());
            state.update_metric_type(metric_type, &self.display_name, secondary_values);
        }
    }

    pub fn push_data(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        self.latest_reading = Some(data.clone());

        if self.time_range.is_real_time() {
            match &mut self.sensor_category {
                SensorCategory::Component(state) => {
                    state.append(timestamp, data);
                    state.prune_before(timestamp - self.time_range.duration_seconds());
                    state.power_graph.chart.refresh_cache();
                }
                SensorCategory::Total(state) => {
                    state.append(timestamp, data);
                    state.prune_before(timestamp - self.time_range.duration_seconds());
                    state.power_graph.chart.refresh_cache();
                }
                SensorCategory::Processes(state) => {
                    if let SensorData::Process(processes) = data {
                        state.update_from_snapshot(processes);
                    }
                }
            }
        }
    }

    pub fn push_to_history_only(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        match &mut self.sensor_category {
            SensorCategory::Component(state) => state.append(timestamp, data),
            SensorCategory::Total(state) => state.append(timestamp, data),
            SensorCategory::Processes(_) => {}
        }
    }

    pub fn load_history_batch(&mut self, data: &[(DateTime<Local>, SensorData)]) {
        if let SensorCategory::Processes(_) = self.sensor_category {
            return;
        }
        for (timestamp, sensor) in data {
            self.push_to_history_only(*timestamp, sensor);
        }
        self.refresh_chart();
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        match &mut self.sensor_category {
            SensorCategory::Component(s) => s.power_graph.chart.update_style(theme),
            SensorCategory::Total(s) => s.power_graph.chart.update_style(theme),
            SensorCategory::Processes(_) => {}
        }
    }

    pub fn refresh_chart(&mut self) {
        match &mut self.sensor_category {
            SensorCategory::Component(s) => s.power_graph.chart.refresh_cache(),
            SensorCategory::Total(s) => s.power_graph.chart.refresh_cache(),
            SensorCategory::Processes(_) => {}
        }
    }

    fn clear_data(&mut self) {
        match &mut self.sensor_category {
            SensorCategory::Component(s) => s.clear(),
            SensorCategory::Total(s) => s.clear(),
            SensorCategory::Processes(s) => s.clear(),
        }
    }

    fn time_range_selector(&self) -> PickList<'_, TimeRange, Vec<TimeRange>, TimeRange, Message, AppTheme> {
        pick_list(
            vec![TimeRange::LastMinute, TimeRange::LastHour, TimeRange::Last24Hours],
            Some(self.time_range.clone()),
            |tr| Message::ChangeChartTimeRange(self.table_name.clone(), tr),
        )
        .class(PickListStyle::TimeRange)
        .menu_class(PickListStyle::TimeRange)
    }

    fn chart_card_header<'b>(
        &'b self,
        title: Option<&'b str>,
        extra_control: Option<Element<'b, Message, AppTheme>>,
    ) -> Row<'b, Message, AppTheme> {
        let title_widget = Text::new(title.unwrap_or(&self.display_name))
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let mut controls = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(self.time_range_selector());
        if let Some(extra) = extra_control {
            controls = controls.push(extra);
        }

        Row::new()
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(title_widget)
            .push(controls)
    }

    fn sensor_chart_card<'b>(
        &'b self,
        chart: &'b PowerChartState,
        title: Option<&'b str>,
        height: f32,
        power_value: Option<f64>,
        snapshot: Option<Row<'b, Message, AppTheme>>,
        metric_selector: Option<Element<'b, Message, AppTheme>>,
    ) -> Element<'b, Message, AppTheme> {
        let header = self.chart_card_header(title, metric_selector);

        let power_text = power_value
            .map(|p| format!("{:.1} W", p))
            .unwrap_or_else(|| "N/A".to_string());
        let power_style = if power_value.is_some() {
            TextStyle::Primary
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
                    .class(power_style)
                    .font(FONT_BOLD),
            );

        let snapshot_container = match snapshot {
            Some(row) => Container::new(row)
                .height(Length::Fixed(SNAPSHOT_AREA_HEIGHT))
                .align_y(Alignment::Center),
            None => Container::new(Space::new()).height(Length::Fixed(SNAPSHOT_AREA_HEIGHT)),
        };

        let info_col = Column::new()
            .spacing(SPACING_SMALL)
            .width(Length::Fill)
            .push(power_row)
            .push(snapshot_container);

        let content = Column::new().spacing(SPACING_LARGE).push(header).push(
            Column::new()
                .push(
                    Row::new()
                        .spacing(SPACING_LARGE)
                        .align_y(Alignment::Center)
                        .push(info_col),
                )
                .push(Container::new(chart.chart.view(height)).width(Length::Fill)),
        );

        Container::new(content)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::ComponentCard)
            .into()
    }

    fn process_card<'b>(&'b self, state: &'b ProcessesState, title: Option<&'b str>) -> Element<'b, Message, AppTheme> {
        let header = self.chart_card_header(title, None);

        let header_font_size = FONT_SIZE_SMALL;
        let header_style = TextStyle::Muted;
        let header_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .push(Space::new().width(Length::Fixed(PROCESS_ICON_COLUMN_WIDTH)))
            .push(text_widget(
                "Application",
                header_font_size,
                header_style,
                Length::Fill,
                false,
            ))
            .push(text_widget(
                "Power",
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_POWER_WIDTH),
                false,
            ))
            .push(text_widget(
                "CPU",
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_CPU_WIDTH),
                false,
            ))
            .push(text_widget(
                "GPU",
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_GPU_WIDTH),
                false,
            ))
            .push(text_widget(
                "RAM",
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_RAM_WIDTH),
                false,
            ))
            .push(text_widget(
                "Disk read",
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_DISK_READ_WIDTH),
                false,
            ))
            .push(text_widget(
                "Disk write",
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_DISK_WRITE_WIDTH),
                false,
            ));

        let mut table = Column::new().spacing(SPACING_SMALL).push(header_row);
        let table_font_size = FONT_SIZE_BODY;
        for p in &state.top_processes {
            let gpu = p.process_gpu_usage.map_or("N/A".to_string(), |v| format!("{:.1}%", v));
            table = table.push(
                Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(process_icon_cell(&p.icon))
                    .push(text_widget(
                        format!("{} ({})", p.app_name, p.subprocess_count),
                        table_font_size,
                        TextStyle::Primary,
                        Length::Shrink,
                        true,
                    ))
                    .push(text_widget(
                        format!("{:.1}W", p.process_usage_watt),
                        table_font_size,
                        TextStyle::Primary,
                        Length::Fixed(PROCESS_POWER_WIDTH),
                        true,
                    ))
                    .push(text_widget(
                        format!("{:.1}%", p.process_cpu_usage),
                        table_font_size,
                        TextStyle::Secondary,
                        Length::Fixed(PROCESS_CPU_WIDTH),
                        false,
                    ))
                    .push(text_widget(
                        gpu,
                        table_font_size,
                        TextStyle::Secondary,
                        Length::Fixed(PROCESS_GPU_WIDTH),
                        false,
                    ))
                    .push(text_widget(
                        format!("{:.1}%", p.process_mem_usage),
                        table_font_size,
                        TextStyle::Secondary,
                        Length::Fixed(PROCESS_RAM_WIDTH),
                        false,
                    ))
                    .push(text_widget(
                        format!("{:.1}MB/s", bytes_to_mb(p.read_bytes_per_sec)),
                        table_font_size,
                        TextStyle::Secondary,
                        Length::Fixed(PROCESS_DISK_READ_WIDTH),
                        false,
                    ))
                    .push(text_widget(
                        format!("{:.1}MB/s", bytes_to_mb(p.written_bytes_per_sec)),
                        table_font_size,
                        TextStyle::Tertiary,
                        Length::Fixed(PROCESS_DISK_WRITE_WIDTH),
                        false,
                    )),
            );
        }

        let table_scrollable = Scrollable::new(table)
            .direction(Direction::Both {
                vertical: Scrollbar::new(),
                horizontal: Scrollbar::new(),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .class(ScrollableStyle::Standard);

        Container::new(Column::new().spacing(SPACING_LARGE).push(header).push(table_scrollable))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::ComponentCard)
            .into()
    }

    pub fn chart_card<'b>(
        &'b self,
        title: Option<&'b str>,
        height: f32,
        show_secondary: bool,
    ) -> Element<'b, Message, AppTheme> {
        let power = self.latest_reading.as_ref().and_then(|d| d.total_power_watts());

        match &self.sensor_category {
            SensorCategory::Component(state) => {
                let snapshot = state.snapshot_row(self.latest_reading.as_ref());
                let metric_selector = if show_secondary {
                    let metrics = state.available_metrics(self.latest_reading.as_ref());
                    if metrics.len() > 1 {
                        Some(
                            pick_list(metrics, Some(state.metric_type), |m| {
                                Message::ChangeChartMetricType(self.table_name.clone(), m)
                            })
                            .class(PickListStyle::TimeRange)
                            .menu_class(PickListStyle::TimeRange)
                            .into(),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.sensor_chart_card(&state.power_graph, title, height, power, snapshot, metric_selector)
            }
            SensorCategory::Total(state) => {
                self.sensor_chart_card(&state.power_graph, title, height, power, None, None)
            }
            SensorCategory::Processes(state) => self.process_card(state, title),
        }
    }
}

fn process_icon_cell(icon_bytes: &Option<Vec<u8>>) -> Element<'static, Message, AppTheme> {
    let icon: Element<'static, Message, AppTheme> = if let Some(bytes) = icon_bytes {
        image(image::Handle::from_bytes(bytes.clone()))
            .width(Length::Fixed(PROCESS_ICON_SIZE))
            .height(Length::Fixed(PROCESS_ICON_SIZE))
            .content_fit(ContentFit::Contain)
            .into()
    } else {
        Space::new().into()
    };

    Container::new(icon)
        .width(Length::Fixed(PROCESS_ICON_COLUMN_WIDTH))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}

fn prune_history(history: &HistoryRef, cutoff: DateTime<Local>) {
    if let Ok(mut borrowed) = history.try_borrow_mut() {
        while borrowed.front().is_some_and(|&(ts, _)| ts < cutoff) {
            borrowed.pop_front();
        }
    }
}
