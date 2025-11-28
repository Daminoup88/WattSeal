use chrono::Utc;
use iced::{
    Color, Element, Font, Length, Subscription, Task, Theme,
    alignment::Alignment,
    font,
    time::{Duration, every},
    widget::{Column, shader::wgpu::naga::back},
};

use crate::{
    components::header::Header,
    message::Message,
    pages::{Page, chart::ChartPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
};

pub struct App {
    current_page: Page,
    chart_page: ChartPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    header: Header,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (chart_page, task) = ChartPage::new();
        let current_page = Page::Chart;
        (
            Self {
                current_page,
                chart_page,
                header: Header::new(
                    &current_page.to_string(),
                    vec![Page::Chart, Page::Info, Page::Optimization, Page::Settings],
                ),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.chart_page.update(Message::Tick);
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.set_title(&page.to_string());
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let view: Element<'_, Message, Theme> = Column::new()
            .push(self.header.view())
            .push(match self.current_page {
                Page::Chart => self.chart_page.view(),
                Page::Info => self.info_page.view(),
                Page::Optimization => self.optimization_page.view(),
                Page::Settings => self.settings_page.view(),
            })
            .into();

        view.explain(Color::BLACK)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        const FPS: u64 = 1;
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }
}
