use std::{collections::HashMap, os::windows::process, time::SystemTime};

use chrono::{DateTime, Local};
use common::{
    AllTimeData, Database, DatabaseEntry, DatabaseError, GPUData, HardwareInfo, ProcessData, SensorData,
    generic_name_for_table,
};
use iced::{
    Element, Subscription, Task,
    time::{Duration, every},
    widget::{Column, Container, canvas::path, pick_list},
};

use crate::{
    components::{header::Header, sensor_state::SensorState},
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    themes::AppTheme,
    types::TimeRange,
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
    header: Header,
    theme: AppTheme,
    database: Database,
    all_time_data: AllTimeData,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::EcoEnergy;
        let current_page = Page::Dashboard;
        let mut database = Database::new().expect("Failed to create database");
        let sensors = database
            .get_tables()
            .into_iter()
            .map(|table_name| {
                let display_name = generic_name_for_table(table_name.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(table_name.clone());
                (table_name.clone(), SensorState::new(table_name, display_name, theme))
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
        let data = from_db(self.database.select_last_n_records(n));
        normalize_integrated_cpu(data)
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
        normalize_integrated_cpu(from_db(result))
    }

    fn load_process_data(&mut self, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        from_db(self.database.select_top_processes_average(time_range as i64, 10))
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let page_content = match self.current_page {
            Page::Dashboard => self.dashboard_page.view(&self.sensors, &self.all_time_data),
            Page::Info => self.info_page.view(&self.hardware_info, self.theme),
            Page::Optimization => self.optimization_page.view(),
            Page::Settings => self.settings_page.view(),
        };

        let theme_picker =
            Container::new(pick_list(AppTheme::all(), Some(self.theme), Message::ChangeTheme)).padding(10);

        Column::new()
            .push(self.header.view())
            .push(page_content)
            .push(theme_picker)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
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

fn normalize_integrated_cpu(mut data: Vec<(DateTime<Local>, SensorData)>) -> Vec<(DateTime<Local>, SensorData)> {
    let mut latest_pp1: Option<f64> = None;
    let mut latest_pp1_timestamp: Option<DateTime<Local>> = None;

    for (time, sensor) in data.iter_mut() {
        if let SensorData::CPU(cpu) = sensor {
            if let Some(pp1) = cpu.pp1_power_watts {
                latest_pp1 = Some(pp1);
                latest_pp1_timestamp = Some(time.clone());
                if let Some(total) = cpu.total_power_watts {
                    cpu.total_power_watts = Some(total - pp1);
                }
            }
        }
    }

    if let Some(pp1) = latest_pp1
        && let Some(pp1_time) = latest_pp1_timestamp
    {
        data.push((
            pp1_time,
            SensorData::GPU(GPUData {
                total_power_watts: Some(pp1),
                usage_percent: None,
                vram_usage_percent: None,
            }),
        ));
    }

    data
}

fn from_db(data: Result<Vec<(SystemTime, SensorData)>, DatabaseError>) -> Vec<(DateTime<Local>, SensorData)> {
    data.unwrap_or_default()
        .into_iter()
        .map(|(ts, data)| (ts.into(), data))
        .collect()
}
