use std::{collections::HashMap, os::windows::process, time::SystemTime};

use chrono::{DateTime, Local};
use common::{
    AllTimeData, Database, DatabaseEntry, DatabaseError, HardwareInfo, ProcessData, SensorData, TotalData,
    generic_name_for_table,
};
use iced::{
    Alignment, Element, Length, Padding, Subscription, Task,
    time::{Duration, every},
    widget::{
        Button, Column, Container, Row, Scrollable, Space, Text, button, center, image, mouse_area, opaque, stack,
    },
};

use crate::{
    components::{header::Header, helpers::modal, sensor_state::SensorState},
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SMALL, FONT_SIZE_SUBTITLE, PADDING_LARGE,
            PADDING_XLARGE, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{
        self, info_modal_all_time_power, info_modal_coming_soon, info_modal_current_power, info_modal_description,
        info_modal_title, info_modal_top_consumer, info_modal_top_process, modal_close, na, window_title,
    },
    types::{AppLanguage, TimeRange},
};

const FPS: u64 = 1;

pub struct App {
    current_page: Page,
    sensors: HashMap<String, SensorState>,
    hardware_info: HardwareInfo,
    dashboard_page: DashboardPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    settings_open: bool,
    info_modal_open: Option<String>,
    language: AppLanguage,
    header: Header,
    theme: AppTheme,
    database: Database,
    all_time_data: AllTimeData,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::EcoEnergy;
        let current_page = Page::Dashboard;
        let language = AppLanguage::default();
        let mut database = Database::new().expect("Failed to create database");
        let sensors = database
            .get_tables()
            .into_iter()
            .map(|table_name| {
                let display_name = generic_name_for_table(table_name.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(table_name.clone());
                (
                    table_name.clone(),
                    SensorState::new(table_name, display_name, theme, language),
                )
            })
            .collect();
        let hardware_info = database.get_hardware_info().unwrap_or_default();
        let dashboard_page = DashboardPage;
        let all_time_data = database.get_all_time_data().unwrap_or_default();
        let task = Task::done(Message::FetchAllChartsData(TimeRange::default()));

        (
            Self {
                current_page,
                sensors,
                dashboard_page,
                header: Header::new(Page::all(), current_page),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                settings_open: false,
                info_modal_open: None,
                language,
                theme,
                database,
                hardware_info,
                all_time_data,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let data = self.load_latest_data(1);
                for (timestamp, sensor_data) in data.iter() {
                    if let Some(sensor) = self.sensors.get_mut(sensor_data.table_name()) {
                        sensor.push_data(*timestamp, sensor_data);
                    }
                }
                self.refresh_all_time_data();
                Task::none()
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.change_page(page);
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                for sensor in self.sensors.values_mut() {
                    sensor.update_theme(theme);
                }
                Task::none()
            }
            Message::ChangeLanguage(language) => {
                self.language = language;
                for sensor in self.sensors.values_mut() {
                    sensor.update_language(language);
                }
                Task::none()
            }
            Message::OpenSettings => {
                self.settings_open = true;
                Task::none()
            }
            Message::CloseSettings => {
                self.settings_open = false;
                Task::none()
            }
            Message::OpenInfoModal(target) => {
                self.info_modal_open = Some(target);
                Task::none()
            }
            Message::CloseInfoModal => {
                self.info_modal_open = None;
                Task::none()
            }
            Message::ChangeChartMetricType(table_name, metric_type) => {
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    sensor.set_metric_type(metric_type);
                }
                Task::none()
            }
            Message::ChangeChartTimeRange(table_name, time_range) => {
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    return sensor.update_time_range(time_range);
                }
                Task::none()
            }
            Message::FetchChartData(table_name, time_range) => {
                let data = self.load_history(&table_name, time_range);
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    sensor.load_history_batch(&data);
                }
                Task::none()
            }
            Message::FetchAllChartsData(time_range) => {
                let table_names = self.database.get_tables();
                for table_name in table_names {
                    let data = self.load_history(&table_name, time_range.clone());
                    if let Some(sensor) = self.sensors.get_mut(&table_name) {
                        sensor.load_history_batch(&data);
                    }
                }
                Task::none()
            }
            Message::UpdateChartData(data) => {
                for (timestamp, sensor_data) in data.iter() {
                    if let Some(sensor) = self.sensors.get_mut(sensor_data.table_name()) {
                        sensor.push_data(*timestamp, sensor_data);
                    }
                }
                Task::none()
            }
            Message::ReplaceChartData(table_name, data) => {
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    sensor.load_history_batch(&data);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn get_hardware_info(&mut self) -> Result<HardwareInfo, DatabaseError> {
        self.database.get_hardware_info()
    }

    fn load_latest_data(&mut self, n: i64) -> Vec<(DateTime<Local>, SensorData)> {
        from_db(self.database.select_last_n_records(n))
    }

    fn load_history(&mut self, table_name: &str, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        if table_name == ProcessData::table_name_static() {
            return self.load_process_data(time_range);
        }
        let result = match time_range {
            TimeRange::LastMinute => self.database.select_data_in_time_range(
                table_name,
                time_range.start_time().into(),
                time_range.end_time().into(),
            ),
            _ => {
                let window = time_range.granularity_seconds();
                self.database
                    .select_last_n_seconds_average(time_range as i64, table_name, window)
            }
        };
        from_db(result)
    }

    fn load_process_data(&mut self, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        from_db(self.database.select_top_processes_average(time_range as i64, 10))
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let page_content = match self.current_page {
            Page::Dashboard => self
                .dashboard_page
                .view(&self.sensors, &self.all_time_data, self.language),
            Page::Info => self.info_page.view(&self.hardware_info, self.theme, self.language),
            Page::Optimization => self.optimization_page.view(self.language),
        };

        let content: Element<'_, Message, AppTheme> = Column::new()
            .push(self.header.view(self.language))
            .push(page_content)
            .into();

        if self.settings_open {
            modal(
                content,
                self.settings_page.view(self.theme, self.language),
                Message::CloseSettings,
            )
        } else if let Some(ref target) = self.info_modal_open {
            modal(content, self.info_modal_view(target), Message::CloseInfoModal)
        } else {
            stack![content].into()
        }
    }

    fn info_modal_view(&self, target: &str) -> Element<'_, Message, AppTheme> {
        let language = self.language;

        let title = Text::new(info_modal_title(language, target))
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let close_button: Button<'_, Message, AppTheme> = button(Text::new(modal_close(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::CloseInfoModal);

        let top_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(title)
            .push(close_button);

        let description = Text::new(info_modal_description(language, target)).size(FONT_SIZE_BODY);

        let mut content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(top_row)
            .push(description);

        if let Some(sensor) = self.sensors.get(target) {
            let power = sensor.get_latest_reading().and_then(|d| d.total_power_watts());

            let power_text = power
                .map(|p| format!("{:.1} W", p))
                .unwrap_or_else(|| na(language).to_string());

            let power_row = Row::new()
                .spacing(SPACING_MEDIUM)
                .align_y(Alignment::Center)
                .push(
                    Text::new(info_modal_current_power(language))
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                )
                .push(
                    Text::new(power_text)
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Primary),
                );

            content = content.push(power_row);
        }

        let all_time_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(
                Text::new(info_modal_all_time_power(language))
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted),
            )
            .push(
                Text::new(info_modal_coming_soon(language))
                    .size(FONT_SIZE_BODY)
                    .class(TextStyle::Muted)
                    .font(FONT_BOLD),
            );
        content = content.push(all_time_row);

        if target == TotalData::table_name_static() {
            if let Some((name, power)) = self.find_top_consumer() {
                let consumer_row = Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(info_modal_top_consumer(language))
                            .size(FONT_SIZE_BODY)
                            .class(TextStyle::Muted),
                    )
                    .push(
                        Text::new(format!("{} ({:.1} W)", name, power))
                            .size(FONT_SIZE_SUBTITLE)
                            .font(FONT_BOLD)
                            .class(TextStyle::Secondary),
                    );
                content = content.push(consumer_row);
            }
        }

        if target == ProcessData::table_name_static() {
            if let Some(proc_sensor) = self.sensors.get(ProcessData::table_name_static()) {
                if let Some(top_proc) = proc_sensor.get_top_process() {
                    let mut proc_row = Row::new().spacing(SPACING_MEDIUM).align_y(Alignment::Center).push(
                        Text::new(info_modal_top_process(language))
                            .size(FONT_SIZE_BODY)
                            .class(TextStyle::Muted),
                    );

                    if let Some(icon_handle) = proc_sensor.get_process_icon(top_proc) {
                        proc_row = proc_row.push(
                            image(icon_handle)
                                .width(Length::Fixed(16.0))
                                .height(Length::Fixed(16.0)),
                        );
                    }

                    proc_row = proc_row.push(
                        Text::new(format!("{} ({:.1} W)", top_proc.app_name, top_proc.process_power_watts))
                            .size(FONT_SIZE_SUBTITLE)
                            .font(FONT_BOLD)
                            .class(TextStyle::Primary),
                    );

                    content = content.push(proc_row);
                }
            }
        }

        Container::new(Scrollable::new(content).width(Length::Fill).height(Length::Shrink))
            .width(Length::Fixed(560.0))
            .max_height(500.0)
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }

    fn find_top_consumer(&self) -> Option<(String, f64)> {
        self.sensors
            .iter()
            .filter(|(name, _)| *name != TotalData::table_name_static() && *name != ProcessData::table_name_static())
            .filter_map(|(_, sensor)| {
                let power = sensor.get_latest_reading().and_then(|d| d.total_power_watts())?;
                Some((sensor.name().to_string(), power))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    pub fn title(&self) -> String {
        window_title(self.language).to_string()
    }

    pub fn theme(&self) -> AppTheme {
        self.theme
    }

    fn refresh_all_time_data(&mut self) {
        if let Ok(all_time_data) = self.database.get_all_time_data() {
            self.all_time_data = all_time_data;
        }
    }
}

fn from_db(data: Result<Vec<(SystemTime, SensorData)>, DatabaseError>) -> Vec<(DateTime<Local>, SensorData)> {
    data.unwrap_or_default()
        .into_iter()
        .map(|(ts, data)| (ts.into(), data))
        .collect()
}
