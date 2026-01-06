use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::{CPUData, Database, DatabaseEntry, Event, GPUData};
use iced::{
    Element, Subscription, Task, Theme,
    time::{Duration, every},
    widget::{Column, pick_list},
};

use crate::{
    components::header::Header,
    message::Message,
    pages::{Page, chart::ChartPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    themes::AppTheme,
};

const FPS: u64 = 1;

pub struct App {
    current_page: Page,
    chart_page: ChartPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    header: Header,
    theme: AppTheme,
    database: Database,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::Dracula;
        let (chart_page, task) = ChartPage::new(theme);
        let current_page = Page::Chart;
        let database = Database::new().unwrap();

        (
            Self {
                current_page,
                chart_page,
                header: Header::new(&current_page.to_string(), Page::all()),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                theme,
                database,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                if let Some(event) = self.load_latest_event() {
                    self.chart_page.update(Message::UpdateChartData(event));
                }
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.set_title(&page.to_string());
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.chart_page.update_theme(theme);
            }
            _ => {}
        }
    }

    fn load_latest_event(&mut self) -> Option<Event> {
        self.database
            .select_last_n_events(1)
            .ok()
            .and_then(|events| events.into_iter().next())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let page_content = match self.current_page {
            Page::Chart => self.chart_page.view(),
            Page::Info => self.info_page.view(),
            Page::Optimization => self.optimization_page.view(),
            Page::Settings => self.settings_page.view(),
        };

        Column::new()
            .push(self.header.view())
            .push(page_content)
            .push(pick_list(AppTheme::all(), Some(self.theme), Message::ChangeTheme))
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    pub fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }
}
