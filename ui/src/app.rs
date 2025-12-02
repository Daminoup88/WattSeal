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
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::Dracula;
        let (chart_page, task) = ChartPage::new(theme);
        let current_page = Page::Chart;

        (
            Self {
                current_page,
                chart_page,
                header: Header::new(&current_page.to_string(), Page::all()),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                theme,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => self.chart_page.update(Message::Tick),
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.set_title(&page.to_string());
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.chart_page.update_theme(theme);
            }
        }
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
