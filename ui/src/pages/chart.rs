use chrono::Utc;
use iced::{
    Element, Font, Length, Task,
    alignment::Alignment,
    font,
    time::Duration,
    widget::{Column, Text},
};
use plotters::style::RGBColor;

use crate::{
    components::chart::{LineType, SensorChart},
    message::Message,
};

const TITLE_FONT_SIZE: u16 = 22;
const SAMPLE_EVERY: Duration = Duration::from_millis(1000);
const FONT_BOLD: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

pub struct ChartPage {
    chart: SensorChart<2>,
}

impl ChartPage {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                chart: SensorChart::new(
                    [
                        ("Series 1".to_string(), RGBColor(255, 0, 0), LineType::Area),
                        ("Series 2".to_string(), RGBColor(0, 255, 0), LineType::Dotted),
                    ],
                    RGBColor(0, 200, 0),
                ),
            },
            Task::done(Message::Tick),
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let now = Utc::now();
                let percent = rand::random::<f32>() * 100.0;
                let percent2 = rand::random::<f32>() * 100.0;
                self.chart.push_data(now, [Some(percent), Some(percent2)]);
            }
            _ => {
                todo!("Add full message match");
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Text::new("Iced test chart").size(TITLE_FONT_SIZE).font(FONT_BOLD))
            .push(self.chart.view(300.0));

        content.into()
    }
}
