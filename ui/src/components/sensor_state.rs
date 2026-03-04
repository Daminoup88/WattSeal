use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use chrono::{DateTime, Duration, Local, Timelike};
use common::{
    DatabaseEntry, DiskData, MetricType, NetworkData, ProcessData, RamData, SecondaryValues, SensorData, TotalData,
    utils::bytes_to_mb,
};
use iced::{
    Alignment, ContentFit, Element, Length, Padding, Task,
    widget::{
        Button, Column, Container, Row, Scrollable, Space, Text, button, image, pick_list,
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
        button::ButtonStyle,
        container::ContainerStyle,
        picklist::PickListStyle,
        scrollable::ScrollableStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_SUBTITLE, PADDING_LARGE, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL,
            SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{
        TranslatedMetricType, TranslatedTimeRange, application, cpu, disk_read, disk_write, gpu, metric_type_name, na,
        power_or_energy, power_or_energy_label, ram, sensor_name, translate_label,
    },
    types::{AppLanguage, TimeRange},
};

const SNAPSHOT_AREA_HEIGHT: f32 = 34.0;
const PROCESS_APP_WIDTH: f32 = 400.0;
const PROCESS_ICON_COLUMN_WIDTH: f32 = 24.0;
const PROCESS_ICON_SIZE: f32 = 16.0;
const PROCESS_POWER_WIDTH: f32 = 100.0;
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
    fn new(theme: AppTheme, language: AppLanguage) -> Self {
        Self {
            power_history: Rc::new(RefCell::new(VecDeque::new())),
            chart: SensorChart::new(theme, language),
            line_type: LineType::default(),
        }
    }

    fn init_power_series(&mut self, display_name: &str, language: AppLanguage) {
        let metric_type = MetricType::default();
        self.chart.set_y_axis_unit(metric_type.unit());
        let key = metric_type.legend(display_name);
        let display = metric_type_name(language, metric_type);
        self.chart
            .add_series(&key, display, self.line_type, Some(metric_type as usize));
        self.chart.set_data(&key, self.power_history.clone());
    }

    fn apply_time_settings(&mut self, line_type: LineType, x_unit: &'static str, duration: Duration) {
        self.line_type = line_type;
        self.chart.set_all_line_types(line_type);
        self.chart.set_x_axis_unit(x_unit);
        self.chart.set_x_range(duration);
    }

    fn append_power(&self, timestamp: DateTime<Local>, power: f32) {
        if let Ok(mut h) = self.power_history.try_borrow_mut() {
            h.push_back((timestamp, power));
        }
    }

    fn prune_before(&self, cutoff: DateTime<Local>) {
        prune_history(&self.power_history, cutoff);
    }

    fn latest_timestamp(&self) -> Option<DateTime<Local>> {
        self.power_history
            .try_borrow()
            .ok()
            .and_then(|h| h.back().map(|(ts, _)| *ts))
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
    _show_in_total: bool,
    metric_type: MetricType,
    pending_initial_metric: Option<MetricType>,
}

impl ComponentState {
    fn new(theme: AppTheme, display_name: &str, language: AppLanguage, table_name: &str) -> Self {
        let mut power_graph = PowerChartState::new(theme, language);
        power_graph.init_power_series(display_name, language);

        Self {
            power_graph,
            secondary_histories: Vec::new(),
            _show_in_total: true,
            metric_type: MetricType::default(),
            pending_initial_metric: initial_metric_for_table(table_name),
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

    fn is_newer_than_latest(&self, timestamp: DateTime<Local>) -> bool {
        self.power_graph
            .latest_timestamp()
            .map_or(true, |latest| timestamp > latest)
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
        language: AppLanguage,
        secondary_values: Option<SecondaryValues>,
        energy_mode: bool,
    ) {
        self.metric_type = metric_type;
        self.power_graph.chart.clear_all();
        self.power_graph
            .chart
            .set_y_axis_unit(metric_type.effective_unit(energy_mode));
        match metric_type {
            MetricType::Power => {
                let key = metric_type.legend(display_name);
                let display = metric_type_name(language, metric_type);
                self.power_graph.chart.add_series(
                    &key,
                    display,
                    self.power_graph.line_type,
                    Some(metric_type as usize),
                );
                self.power_graph
                    .chart
                    .set_data(&key, self.power_graph.power_history.clone());
            }
            _ => {
                if let Some(secondary_values) = secondary_values {
                    self.extend_secondary_histories(secondary_values.values.len());
                    for (i, labeled_value) in secondary_values.values.into_iter().enumerate() {
                        let key = format!("{} {}", display_name, labeled_value.label);
                        let display = translate_label(language, labeled_value.label);
                        self.power_graph.chart.add_series(
                            &key,
                            display,
                            self.power_graph.line_type,
                            Some(i.saturating_add(1)),
                        );
                        self.power_graph
                            .chart
                            .set_data(&key, self.secondary_histories[i].clone());
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

    fn snapshot_row(
        &self,
        latest: Option<&SensorData>,
        language: AppLanguage,
    ) -> Option<Row<'static, Message, AppTheme>> {
        let secondary_values = latest?.secondary_values()?;
        let mut col = Column::new().spacing(SPACING_SMALL);
        for (i, labeled_value) in secondary_values.values.into_iter().enumerate() {
            if let Some(value) = labeled_value.value {
                let style = match i % 3 {
                    0 => TextStyle::Secondary,
                    1 => TextStyle::Tertiary,
                    _ => TextStyle::Subtitle,
                };
                let translated = translate_label(language, labeled_value.label);
                col = col.push(
                    Row::new()
                        .spacing(SPACING_MEDIUM)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(format!("{}:", translated))
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
    _tooltip_history: TooltipHistoryRef,
}

impl TotalState {
    fn new(theme: AppTheme, display_name: &str, language: AppLanguage) -> Self {
        let mut power_graph = PowerChartState::new(theme, language);
        power_graph.init_power_series(display_name, language);
        Self {
            power_graph,
            _tooltip_history: Rc::new(RefCell::new(VecDeque::new())),
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

    fn is_newer_than_latest(&self, timestamp: DateTime<Local>) -> bool {
        self.power_graph
            .latest_timestamp()
            .map_or(true, |latest| timestamp > latest)
    }

    fn clear(&self) {
        self.power_graph.clear();
    }
}

struct ProcessesState {
    top_processes: Vec<ProcessData>,
    icon_handles: HashMap<String, image::Handle>,
}

impl ProcessesState {
    fn new() -> Self {
        Self {
            top_processes: Vec::new(),
            icon_handles: HashMap::new(),
        }
    }

    fn update_from_snapshot(&mut self, processes: &[ProcessData]) {
        let next_top = processes.to_vec();

        for process in &next_top {
            if let Some(icon_data) = &process.icon {
                self.icon_handles.insert(
                    process_identity(process),
                    image::Handle::from_rgba(icon_data.width, icon_data.height, icon_data.pixels.clone()),
                );
            }
        }

        self.icon_handles
            .retain(|key, _| next_top.iter().any(|process| process_identity(process) == *key));
        self.top_processes = next_top;
    }

    fn clear(&mut self) {
        self.top_processes.clear();
        self.icon_handles.clear();
    }

    fn icon_handle_for(&self, process: &ProcessData) -> Option<image::Handle> {
        self.icon_handles.get(&process_identity(process)).cloned()
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
    language: AppLanguage,
}

impl SensorState {
    pub fn new(table_name: String, display_name: String, theme: AppTheme, language: AppLanguage) -> Self {
        let sensor_category = if table_name == TotalData::table_name_static() {
            SensorCategory::Total(TotalState::new(theme, &display_name, language))
        } else if table_name == ProcessData::table_name_static() {
            SensorCategory::Processes(ProcessesState::new())
        } else {
            SensorCategory::Component(ComponentState::new(theme, &display_name, language, &table_name))
        };

        let mut state = Self {
            table_name,
            display_name,
            sensor_category,
            latest_reading: None,
            time_range: TimeRange::default(),
            language,
        };
        let _ = state.apply_time_range(TimeRange::default());
        state
    }

    pub fn name(&self) -> &str {
        &self.display_name
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn get_latest_reading(&self) -> Option<&SensorData> {
        self.latest_reading.as_ref()
    }

    pub fn get_top_process(&self) -> Option<&ProcessData> {
        if let SensorCategory::Processes(state) = &self.sensor_category {
            state.top_processes.first()
        } else {
            None
        }
    }

    pub fn get_process_icon(&self, process: &ProcessData) -> Option<image::Handle> {
        if let SensorCategory::Processes(state) = &self.sensor_category {
            state.icon_handle_for(process)
        } else {
            None
        }
    }

    pub fn current_time_range(&self) -> &TimeRange {
        &self.time_range
    }

    fn apply_time_range(&mut self, time_range: TimeRange) -> Task<Message> {
        self.time_range = time_range;
        let line_type = match &self.time_range {
            TimeRange::LastMinute => LineType::Line,
            _ => LineType::Step,
        };
        let unit = self.time_range.unit();
        let duration = self.time_range.duration_seconds();
        let energy_mode = self.time_range.is_energy_mode();
        match &mut self.sensor_category {
            SensorCategory::Component(s) => {
                s.power_graph.apply_time_settings(line_type, unit, duration);
                if s.metric_type == MetricType::Power {
                    s.power_graph
                        .chart
                        .set_y_axis_unit(MetricType::Power.effective_unit(energy_mode));
                    s.power_graph
                        .chart
                        .set_all_display_labels(power_or_energy(self.language, energy_mode));
                }
            }
            SensorCategory::Total(s) => {
                s.power_graph.apply_time_settings(line_type, unit, duration);
                s.power_graph
                    .chart
                    .set_y_axis_unit(MetricType::Power.effective_unit(energy_mode));
                s.power_graph.chart.set_all_line_types(line_type);
                s.power_graph
                    .chart
                    .set_all_display_labels(power_or_energy(self.language, energy_mode));
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
            state.update_metric_type(
                metric_type,
                &self.display_name,
                self.language,
                secondary_values,
                self.time_range.is_energy_mode(),
            );
        }
    }

    pub fn push_data(&mut self, timestamp: DateTime<Local>, data: &SensorData) {
        if matches!(self.sensor_category, SensorCategory::Processes(_)) {
            return;
        }

        let timestamp = timestamp.with_nanosecond(0).unwrap_or(timestamp);
        self.latest_reading = Some(data.clone());

        if self.time_range.is_real_time() {
            match &mut self.sensor_category {
                SensorCategory::Component(state) => {
                    if state.is_newer_than_latest(timestamp) {
                        state.append(timestamp, data);
                    }
                    state.prune_before(timestamp - self.time_range.duration_seconds());
                    state.power_graph.chart.refresh_cache();
                }
                SensorCategory::Total(state) => {
                    if state.is_newer_than_latest(timestamp) {
                        state.append(timestamp, data);
                    }
                    state.prune_before(timestamp - self.time_range.duration_seconds());
                    state.power_graph.chart.refresh_cache();
                }
                SensorCategory::Processes(_) => {}
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
        if let SensorCategory::Processes(state) = &mut self.sensor_category {
            if let Some((_, SensorData::Process(processes))) = data.last() {
                state.update_from_snapshot(processes);
            }
            return;
        }

        if let SensorCategory::Component(state) = &mut self.sensor_category {
            if let Some(initial_metric) = state.pending_initial_metric.take() {
                if let Some((_, sensor_data)) = data.last() {
                    let secondary = sensor_data.secondary_values();
                    state.update_metric_type(
                        initial_metric,
                        &self.display_name,
                        self.language,
                        secondary,
                        self.time_range.is_energy_mode(),
                    );
                }
            }
        }

        self.clear_data();
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

    pub fn update_language(&mut self, language: AppLanguage) {
        self.language = language;
        match &mut self.sensor_category {
            SensorCategory::Component(state) => {
                state.power_graph.chart.update_language(language);
                let secondary_values = self.latest_reading.as_ref().and_then(|d| d.secondary_values());
                state.update_metric_type(
                    state.metric_type,
                    &self.display_name,
                    language,
                    secondary_values,
                    self.time_range.is_energy_mode(),
                );
            }
            SensorCategory::Total(state) => {
                state.power_graph.chart.update_language(language);
            }
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

    fn time_range_selector(&self) -> Element<'_, Message, AppTheme> {
        let options = if self.table_name == TotalData::table_name_static()
            || self.table_name == ProcessData::table_name_static()
        {
            TranslatedTimeRange::options_total(self.language)
        } else {
            TranslatedTimeRange::options_component(self.language)
        };
        pick_list(
            options,
            Some(TranslatedTimeRange::new(self.time_range.clone(), self.language)),
            |tr: TranslatedTimeRange| Message::ChangeChartTimeRange(self.table_name.clone(), tr.range),
        )
        .class(PickListStyle::TimeRange)
        .menu_class(PickListStyle::TimeRange)
        .into()
    }

    fn chart_card_header<'b>(
        &'b self,
        title: Option<&'b str>,
        extra_control: Option<Element<'b, Message, AppTheme>>,
    ) -> Row<'b, Message, AppTheme> {
        let default_name = sensor_name(self.language, &self.display_name);
        let title_widget = Text::new(title.unwrap_or(default_name))
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let info_button: Button<'b, Message, AppTheme> = button(Text::new("?").size(FONT_SIZE_BODY).font(FONT_BOLD))
            .class(ButtonStyle::InfoHelp)
            .on_press(Message::OpenInfoModal(self.table_name.clone()))
            .padding(Padding::from([2, 8]));

        let mut controls = Row::new().spacing(SPACING_MEDIUM).align_y(Alignment::Center);

        if !self.time_range.is_real_time() {
            let refresh_button: Button<'b, Message, AppTheme> =
                button(Text::new("↻").size(FONT_SIZE_BODY).font(FONT_BOLD))
                    .class(ButtonStyle::Standard)
                    .on_press(Message::FetchChartData(
                        self.table_name.clone(),
                        self.time_range.clone(),
                    ))
                    .padding(Padding::from([2, 8]));
            controls = controls.push(refresh_button);
        }

        controls = controls.push(self.time_range_selector());

        if let Some(extra) = extra_control {
            controls = controls.push(extra);
        }
        controls = controls.push(info_button);

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

        // Snapshot always shows CURRENT power in watts
        let power_text = power_value
            .map(|p| format!("{:.1} W", p))
            .unwrap_or_else(|| na(self.language).to_string());
        let power_style = if power_value.is_some() {
            TextStyle::Primary
        } else {
            TextStyle::Muted
        };

        let power_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(
                Text::new(power_or_energy_label(self.language, false))
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted),
            )
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
        let energy_mode = self.time_range.is_energy_mode();
        let unit_str = self.time_range.power_unit();

        let header_font_size = FONT_SIZE_BODY;
        let header_style = TextStyle::Muted;
        let header_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .push(Space::new().width(Length::Fixed(PROCESS_ICON_COLUMN_WIDTH)))
            .push(text_widget(
                application(self.language),
                header_font_size,
                header_style,
                Length::Fill,
                false,
            ))
            .push(text_widget(
                power_or_energy(self.language, energy_mode),
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_POWER_WIDTH),
                false,
            ))
            .push(text_widget(
                cpu(self.language),
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_CPU_WIDTH),
                false,
            ))
            .push(text_widget(
                gpu(self.language),
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_GPU_WIDTH),
                false,
            ))
            .push(text_widget(
                ram(self.language),
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_RAM_WIDTH),
                false,
            ))
            .push(text_widget(
                disk_read(self.language),
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_DISK_READ_WIDTH),
                false,
            ))
            .push(text_widget(
                disk_write(self.language),
                header_font_size,
                header_style,
                Length::Fixed(PROCESS_DISK_WRITE_WIDTH),
                false,
            ));

        let mut table = Column::new().spacing(SPACING_SMALL).push(header_row);
        let table_font_size = FONT_SIZE_BODY;
        for p in &state.top_processes {
            let gpu = p
                .process_gpu_usage
                .map_or(na(self.language).to_string(), |v| format!("{:.1}%", v));
            table = table.push(
                Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(process_icon_cell(state.icon_handle_for(p)))
                    .push(text_widget(
                        format!("{} ({})", p.app_name, p.subprocess_count),
                        table_font_size,
                        TextStyle::Primary,
                        Length::Fixed(PROCESS_APP_WIDTH),
                        true,
                    ))
                    .push(text_widget(
                        format!("{:.1}{}", p.process_power_watts, unit_str),
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

    pub fn sensor_visual_card<'b>(
        &'b self,
        title: Option<&'b str>,
        height: f32,
        show_secondary: bool,
    ) -> Element<'b, Message, AppTheme> {
        let power = self.latest_reading.as_ref().and_then(|d| d.total_power_watts());

        match &self.sensor_category {
            SensorCategory::Component(state) => {
                let snapshot = state.snapshot_row(self.latest_reading.as_ref(), self.language);
                let metric_selector = if show_secondary {
                    let metrics = state.available_metrics(self.latest_reading.as_ref());
                    if metrics.len() > 1 {
                        let translated: Vec<TranslatedMetricType> = metrics
                            .into_iter()
                            .map(|m| TranslatedMetricType::new(m, self.language))
                            .collect();
                        let selected = TranslatedMetricType::new(state.metric_type, self.language);
                        Some(
                            pick_list(translated, Some(selected), |tm: TranslatedMetricType| {
                                Message::ChangeChartMetricType(self.table_name.clone(), tm.metric)
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

fn process_icon_cell(cached_handle: Option<image::Handle>) -> Element<'static, Message, AppTheme> {
    let icon: Element<'static, Message, AppTheme> = if let Some(handle) = cached_handle {
        image(handle)
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

fn process_identity(process: &ProcessData) -> String {
    process
        .process_exe_path
        .as_ref()
        .cloned()
        .unwrap_or_else(|| process.app_name.clone())
}

fn prune_history(history: &HistoryRef, cutoff: DateTime<Local>) {
    if let Ok(mut borrowed) = history.try_borrow_mut() {
        while borrowed.front().is_some_and(|&(ts, _)| ts < cutoff) {
            borrowed.pop_front();
        }
    }
}

fn initial_metric_for_table(table_name: &str) -> Option<MetricType> {
    if table_name == RamData::table_name_static() {
        Some(MetricType::Usage)
    } else if table_name == DiskData::table_name_static() || table_name == NetworkData::table_name_static() {
        Some(MetricType::Speed)
    } else {
        None
    }
}
