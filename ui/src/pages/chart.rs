use chrono::Utc;
use iced::{
    Element, Font, Length, Task,
    alignment::Alignment,
    font,
    time::Duration,
    widget::{Column, Text},
};

use crate::{
    components::chart::{LineType, SensorChart},
    message::Message,
    themes::AppTheme,
};

const TITLE_FONT_SIZE: u16 = 22;
const SAMPLE_EVERY: Duration = Duration::from_millis(1000);
const TITLE_FONT: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

pub struct ChartPage {
    chart: SensorChart<2>,
}

impl ChartPage {
    pub fn new(theme: AppTheme) -> (Self, Task<Message>) {
        let chart = SensorChart::new(
            [
                ("Series 1".into(), LineType::Area),
                ("Series 2".into(), LineType::Dotted),
            ],
            None,
            None,
            theme,
        );
        (Self { chart }, Task::done(Message::Tick))
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        self.chart.update_style(theme);
    }

    pub fn update(&mut self, message: Message) {
        if let Message::Tick = message {
            let now = Utc::now();
            self.chart.push_data(
                now,
                [
                    Some(rand::random::<f32>() * 100.0),
                    Some(rand::random::<f32>() * 1000.0),
                ],
            );
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Text::new("Iced test chart").size(TITLE_FONT_SIZE).font(TITLE_FONT))
            .push(self.chart.view(300.0))
            .into()
    }
}
