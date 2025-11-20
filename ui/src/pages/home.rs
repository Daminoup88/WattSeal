use chrono::Utc;
use iced::{Color, Element, Font, Length, alignment::Alignment, font};
use iced::widget::{Column, Container, Text};

use crate::components::SensorChart;
use crate::message::Message;

const TITLE_FONT_SIZE: u16 = 22;
const FONT_BOLD: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

pub struct HomePage {
    pub chart: SensorChart,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            chart: SensorChart::new(std::iter::empty()),
        }
    }

    pub fn update(&mut self, message: &Message) {
        match message {
            Message::Tick => {
                let now = Utc::now();
                let percent = rand::random::<f32>() * 100.0;
                let percent2 = rand::random::<f32>() * 100.0;
                self.chart.push_data(now, percent, percent2);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(
                Text::new("Iced test chart")
                    .size(TITLE_FONT_SIZE)
                    .font(FONT_BOLD),
            )
            .push(self.chart.view(300.0));

        let view: Element<'_, Message> = Container::new(content)
            .padding(5)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        view.explain(Color::BLACK)
    }
}
