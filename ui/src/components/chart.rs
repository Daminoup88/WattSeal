use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use iced::{
    Element, Length, Size,
    alignment::Alignment,
    time::Duration,
    widget::{
        Column, Text,
        canvas::{Cache, Frame, Geometry},
    },
};
use plotters::{coord::Shift, prelude::ChartBuilder};
use plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartWidget, DrawingArea, Renderer, plotters_backend};

use crate::message::Message;

const PLOT_SECONDS: usize = 60;

pub struct SensorChart {
    cache: Cache,
    data: VecDeque<(DateTime<Utc>, f32)>,
    data2: VecDeque<(DateTime<Utc>, f32)>,
    limit: Duration,
}

impl SensorChart {
    pub fn new(data: impl Iterator<Item = (DateTime<Utc>, f32)>) -> Self {
        let data: VecDeque<_> = data.collect();
        Self {
            cache: Cache::new(),
            data,
            data2: VecDeque::new(),
            limit: Duration::from_secs(PLOT_SECONDS as u64),
        }
    }

    pub fn push_data(&mut self, time: DateTime<Utc>, value: f32, value2: f32) {
        let cur_ms = time.timestamp_millis();
        self.data.push_front((time, value));
        self.data2.push_front((time, value2));
        loop {
            if let Some((time, _)) = self.data.back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > self.limit {
                    self.data.pop_back();
                    continue;
                }
            }

            if let Some((time, _)) = self.data2.back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > self.limit {
                    self.data2.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    pub fn view(&self, chart_height: f32) -> Element<'_, Message> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(5)
            .align_x(Alignment::Center)
            .push(Text::new("Processor x"))
            .push(ChartWidget::new(self).height(Length::Fixed(chart_height)))
            .into()
    }
}

impl Chart<Message> for SensorChart {
    type State = ();
    // fn update(
    //     &self,
    //     event: Event,
    //     bounds: Rectangle,
    //     cursor: Cursor,
    // ) -> (event::Status, Option<Message>) {
    //     self.cache.clear();
    //     (event::Status::Ignored, None)
    // }

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(&self, renderer: &R, bounds: Size, draw_fn: F) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, chart: ChartBuilder<DB>) {
        build_chart_2d(chart, &self.data, &self.data2);

        // build_chart_3d(chart);
    }

    // fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    // fn draw_chart<DB: DrawingBackend>(&self, _state: &Self::State, root: DrawingArea<DB, Shift>) {
    //     let children = root.split_evenly((1, 2));
    //     for (i, area) in children.iter().enumerate() {
    //         let builder = ChartBuilder::on(area);
    //         draw_chart(builder, i + 1);
    //     }
    // }
}

// fn draw_chart<DB: DrawingBackend>(mut chart: ChartBuilder<DB>, power: usize) {
//     let mut chart = chart
//         .margin(30)
//         .caption(format!("y=x^{}", power), ("sans-serif", 22))
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_cartesian_2d(-1f32..1f32, -1.2f32..1.2f32)
//         .unwrap();

//     chart
//         .configure_mesh()
//         .x_labels(3)
//         .y_labels(3)
//         // .y_label_style(
//         //     ("sans-serif", 15)
//         //         .into_font()
//         //         .color(&plotters::style::colors::BLACK.mix(0.8))
//         //         .transform(FontTransform::RotateAngle(30.0)),
//         // )
//         .draw()
//         .unwrap();

//     chart
//         .draw_series(LineSeries::new(
//             (-50..=50)
//                 .map(|x| x as f32 / 50.0)
//                 .map(|x| (x, x.powf(power as f32))),
//             &RED,
//         ))
//         .unwrap();
// }

fn build_chart_2d<DB: DrawingBackend>(
    mut chart: ChartBuilder<DB>,
    data: &VecDeque<(DateTime<Utc>, f32)>,
    data2: &VecDeque<(DateTime<Utc>, f32)>,
) {
    use plotters::prelude::*;
    const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

    // Acquire time range
    let newest_time = data
        .front()
        .unwrap_or(&(DateTime::from_timestamp(0, 0).unwrap(), 0.0))
        .0;
    let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);
    let mut chart = chart
        .x_label_area_size(0)
        .y_label_area_size(28)
        .margin(20)
        .build_cartesian_2d(oldest_time..newest_time, 0.0f32..100.0f32)
        .expect("failed to build chart");

    chart
        .configure_mesh()
        .bold_line_style(plotters::style::colors::BLUE.mix(0.1))
        .light_line_style(plotters::style::colors::BLUE.mix(0.05))
        .axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
        .y_labels(10)
        .y_label_style(
            ("sans-serif", 15)
                .into_font()
                .color(&plotters::style::colors::BLUE.mix(0.65))
                .transform(FontTransform::Rotate90),
        )
        .y_label_formatter(&|y: &f32| format!("{}%", y))
        .draw()
        .expect("failed to draw chart mesh");

    chart
        .draw_series(LineSeries::new(data.iter().map(|x| (x.0, x.1)), PLOT_LINE_COLOR))
        .expect("failed to draw chart data");

    chart
        .draw_series(
            AreaSeries::new(
                data2.iter().map(|x| (x.0, x.1)),
                0.0,
                RGBColor(255, 100, 100).mix(0.175),
            )
            .border_style(ShapeStyle::from(RGBColor(255, 100, 100)).stroke_width(2)),
        )
        .expect("failed to draw chart data");
}

fn build_chart_3d<DB: DrawingBackend>(mut chart: ChartBuilder<DB>) {
    use plotters::prelude::*;
    let mut chart = chart
        .margin(10)
        .build_cartesian_3d(-3.0..3.0f64, -3.0..3.0f64, -3.0..3.0f64)
        .unwrap();
    chart.configure_axes().draw().unwrap();
    chart
        .draw_series(
            [("x", (3., -3., -3.)), ("y", (-3., 3., -3.)), ("z", (-3., -3., 3.))]
                .map(|(label, position)| Text::new(label, position, TextStyle::from(("sans-serif", 20).into_font()))),
        )
        .unwrap();
    chart
        .draw_series(
            SurfaceSeries::xoz(
                (-30..30).map(|v| v as f64 / 10.0),
                (-30..30).map(|v| v as f64 / 10.0),
                |x: f64, z: f64| (0.7 * (x * x + z * z)).cos(),
            )
            .style(&BLUE.mix(0.5)),
        )
        .unwrap();
}
