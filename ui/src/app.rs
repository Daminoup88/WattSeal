use std::{collections::HashMap, os::windows::process, time::SystemTime};

use chrono::{DateTime, Local};
use common::{Database, DatabaseError, GPUData, ProcessData, SensorData, generic_name_for_table};
use iced::{
    Element, Subscription, Task,
    time::{Duration, every},
    widget::{Column, Container, canvas::path, pick_list},
};

use crate::{
    components::{component_state::ComponentState, header::Header},
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    themes::AppTheme,
    types::TimeRange,
};

const FPS: u64 = 1;

pub struct App<'a> {
    current_page: Page,
    components: HashMap<String, ComponentState<'a>>,
    dashboard_page: DashboardPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    header: Header,
    theme: AppTheme,
    database: Database,
}

impl<'a> App<'a> {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::EcoEnergy;
        let current_page = Page::Dashboard;
        let database = Database::new().expect("Failed to create database");
        let components = database
            .get_tables()
            .into_iter()
            .map(|table_name| {
                let sensor_type = generic_name_for_table(table_name.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(table_name.clone());
                (table_name.clone(), ComponentState::new(table_name, sensor_type, theme))
            })
            .collect();
        let dashboard_page = DashboardPage;
        let task = Task::done(Message::FetchAllChartsData(TimeRange::default()));

        (
            Self {
                current_page,
                components,
                dashboard_page,
                header: Header::new(Page::all(), current_page),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                theme,
                database,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let data = self.load_latest_data(1);
                for (timestamp, sensor) in data.iter() {
                    if let Some(component) = self.components.get_mut(sensor.table_name()) {
                        component.push_data(*timestamp, sensor);
                    }
                }
                Task::none()
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.change_page(page);
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                for component in self.components.values_mut() {
                    component.update_theme(theme);
                }
                Task::none()
            }
            Message::ChangeChartMetricType(table_name, metric_type) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    component.set_metric_type(metric_type);
                }
                Task::none()
            }
            Message::ChangeChartTimeRange(table_name, time_range) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    return component.update_time_range(time_range);
                }
                Task::none()
            }
            Message::FetchChartData(table_name, time_range) => {
                let data = self.load_history(&table_name, time_range);
                if let Some(component) = self.components.get_mut(&table_name) {
                    component.load_history_batch(&data);
                }
                Task::none()
            }
            Message::FetchAllChartsData(time_range) => {
                let data = self.load_all_charts_history(time_range);
                for (timestamp, sensor) in data.iter() {
                    if let Some(component) = self.components.get_mut(sensor.table_name()) {
                        component.push_data(*timestamp, sensor);
                    }
                }
                Task::none()
            }
            Message::UpdateChartData(data) => {
                for (timestamp, sensor) in data.iter() {
                    if let Some(component) = self.components.get_mut(sensor.table_name()) {
                        component.push_data(*timestamp, sensor);
                    }
                }
                Task::none()
            }
            Message::ReplaceChartData(table_name, data) => {
                if let Some(component) = self.components.get_mut(&table_name) {
                    component.load_history_batch(&data);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn load_all_charts_history(&mut self, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        normalize_integrated_cpu(from_db(
            self.database
                .select_all_data_in_time_range(time_range.start_time().into(), time_range.end_time().into()),
        ))
    }

    fn load_latest_data(&mut self, n: i64) -> Vec<(DateTime<Local>, SensorData)> {
        let data = from_db(self.database.select_last_n_records(n));
        normalize_integrated_cpu(data)
    }

    fn load_history(&mut self, table_name: &str, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
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

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let page_content = match self.current_page {
            Page::Dashboard => self.dashboard_page.view(&self.components),
            Page::Info => self.info_page.view(),
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
