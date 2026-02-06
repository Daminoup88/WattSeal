use std::time::SystemTime;

use chrono::{DateTime, Utc};
use common::{Database, DatabaseError, GPUData, SensorData};
use iced::{
    Element, Subscription, Task,
    time::{Duration, every},
    widget::{Column, Container, pick_list},
};

use crate::{
    components::header::Header,
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    themes::AppTheme,
    types::TimeRange,
};

const FPS: u64 = 1;

pub struct App<'a> {
    current_page: Page,
    dashboard_page: DashboardPage<'a>,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    header: Header,
    theme: AppTheme,
    database: Database,
}

impl<'a> App<'a> {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::Dracula;
        let current_page = Page::Dashboard;
        let database = Database::new().unwrap();
        let materials = database.get_tables();
        let (dashboard_page, task) = DashboardPage::new(theme, materials);

        (
            Self {
                current_page,
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
                self.dashboard_page.update(Message::UpdateChartData(data))
            }
            Message::LoadChartEvents(n) => {
                let data = self.load_latest_data(n);
                self.dashboard_page.update(Message::UpdateChartData(data))
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.change_page(page);
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.dashboard_page.update_theme(theme);
                Task::none()
            }
            Message::FetchChartData(table_name, time_range) => {
                let data = self.load_history(&table_name, time_range);
                self.dashboard_page.update(Message::ReplaceChartData(table_name, data))
            }
            msg @ (Message::ChangeChartMetricType(_) | Message::ChangeChartTimeRange(..)) => {
                self.dashboard_page.update(msg)
            }
            _ => Task::none(),
        }
    }

    fn load_latest_data(&mut self, n: i64) -> Vec<(DateTime<Utc>, SensorData)> {
        let data = from_db(self.database.select_last_n_records(n));
        normalize_integrated_cpu(data)
    }

    fn load_history(&mut self, table_name: &str, time_range: TimeRange) -> Vec<(DateTime<Utc>, SensorData)> {
        let result = match time_range {
            TimeRange::LastMinute => self.database.select_data_in_time_range(
                table_name,
                (Utc::now() - time_range.duration_seconds()).into(),
                Utc::now().into(),
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
            Page::Dashboard => self.dashboard_page.view(),
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

fn normalize_integrated_cpu(mut data: Vec<(DateTime<Utc>, SensorData)>) -> Vec<(DateTime<Utc>, SensorData)> {
    let mut latest_pp1: Option<f64> = None;
    let mut latest_pp1_timestamp: Option<DateTime<Utc>> = None;

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

fn from_db(data: Result<Vec<(SystemTime, SensorData)>, DatabaseError>) -> Vec<(DateTime<Utc>, SensorData)> {
    data.unwrap_or_default()
        .into_iter()
        .map(|(ts, data)| (ts.into(), data))
        .collect()
}
