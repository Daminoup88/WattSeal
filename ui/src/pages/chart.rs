use chrono::{DateTime, Utc};
use common::SensorData;
use iced::{
    Element, Font, Length, Task,
    alignment::Alignment,
    font,
    time::Duration,
    widget::{Column, Text},
};

use crate::{
    components::chart::{AxisType, LineType, SensorChart},
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
    chart: SensorChart,
}

impl ChartPage {
    pub fn new(theme: AppTheme) -> (Self, Task<Message>) {
        let series = vec![
            (
                "CPU Power".to_string(),
                LineType::Line,
                AxisType::Primary("CPU Power".to_string(), "W".to_string()),
            ),
            (
                "GPU Power".to_string(),
                LineType::Line,
                AxisType::Primary("GPU Power".to_string(), "W".to_string()),
            ),
            (
                "CPU Usage".to_string(),
                LineType::Dashed,
                AxisType::Secondary("CPU Usage".to_string(), "%".to_string()),
            ),
            (
                "GPU Usage".to_string(),
                LineType::Dashed,
                AxisType::Secondary("GPU Usage".to_string(), "%".to_string()),
            ),
        ];
        let x_axis = AxisType::Primary("Time".to_string(), "s".to_string());
        let y_axes = (
            AxisType::Primary("Power".to_string(), "W".to_string()),
            AxisType::Secondary("Usage".to_string(), "%".to_string()),
        );
        let chart = SensorChart::new(series, None, None, theme, x_axis, y_axes);
        (Self { chart }, Task::done(Message::LoadChartEvents(60)))
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        self.chart.update_style(theme);
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateChartData(data) => {
                self.chart.push_data(data);
            }
            _ => {}
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
