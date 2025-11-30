use std::{cell::RefCell, collections::VecDeque, fmt::format};

use chrono::{DateTime, Utc};
use iced::{
    Element, Length, Point, Rectangle, Size,
    alignment::Alignment,
    mouse::{self, Cursor},
    time::Duration,
    widget::{
        Column, Text,
        canvas::{self, Cache, Event, Frame, Geometry, event},
        text_input::cursor,
    },
};
use plotters::{
    coord::Shift,
    data,
    prelude::ChartBuilder,
    style::{Color, RGBAColor, RGBColor},
};
use plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartWidget, DrawingArea, Renderer, plotters_backend};

use crate::{message::Message, themes::AppTheme};

const PLOT_SECONDS: usize = 60;
const SNAP_DISTANCE_PX: f32 = 30.0;
const VALUE_MIN: f32 = 0.0;
const VALUE_MAX: f32 = 100.0;
const X_LABEL_AREA_SIZE: f32 = 40.0;
const Y_LABEL_AREA_SIZE: f32 = 28.0;
const CHART_MARGIN: f32 = 20.0;

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub grid_bold: RGBAColor,
    pub grid_light: RGBAColor,
    pub axis: RGBAColor,
    pub text: RGBAColor,
    pub legend_background: RGBAColor,
    pub legend_border: RGBColor,
    pub series_colors: [RGBColor; 4],
}

impl From<AppTheme> for ChartStyle {
    fn from(theme: AppTheme) -> Self {
        let p = theme.palette();
        let [text, background, primary, success, danger] =
            [p.text, p.background, p.primary, p.success, p.danger].map(to_plotters_color);

        Self {
            grid_bold: text.mix(0.1),
            grid_light: text.mix(0.05),
            axis: text.mix(0.45),
            text: text.mix(0.65),
            legend_background: background.mix(0.8),
            legend_border: text,
            series_colors: [primary, success, danger, text],
        }
    }
}

impl ChartStyle {
    pub fn series_color(&self, index: usize) -> RGBColor {
        self.series_colors[index % self.series_colors.len()]
    }
}

type Range = (f32, f32);

#[derive(PartialEq)]
struct HoverInfo {
    label: String,
    time: DateTime<Utc>,
    value: f32,
}

pub struct SensorChart<const N: usize> {
    cache: RefCell<Cache>,
    data_series: Vec<TimeSeries>,
    limit: Duration,
    hovered: RefCell<Option<HoverInfo>>,
    range: Range,
    dynamic_range: bool,
    style: ChartStyle,
}

#[derive(Default, Clone, Copy)]
pub enum LineType {
    #[default]
    Line,
    Dashed,
    Area,
    Dotted,
    Points,
}

struct TimeSeries {
    label: String,
    data: VecDeque<(DateTime<Utc>, f32)>,
    line_type: LineType,
}

impl From<(String, LineType)> for TimeSeries {
    fn from((label, line_type): (String, LineType)) -> Self {
        Self {
            label,
            data: VecDeque::new(),
            line_type,
        }
    }
}

impl TimeSeries {
    fn iter(&self) -> impl Iterator<Item = (DateTime<Utc>, f32)> + '_ {
        self.data.iter().copied()
    }

    fn newest_time(&self) -> Option<DateTime<Utc>> {
        self.data.front().map(|(time, _)| *time)
    }

    fn oldest_time(&self) -> Option<DateTime<Utc>> {
        self.data.back().map(|(time, _)| *time)
    }
}

fn to_plotters_color(color: iced::Color) -> RGBColor {
    let rgba = color.into_rgba8();
    RGBColor(rgba[0], rgba[1], rgba[2])
}

impl<const N: usize> SensorChart<N> {
    pub fn new(series: [(String, LineType); N], min_y: Option<f32>, max_y: Option<f32>, theme: AppTheme) -> Self {
        Self {
            cache: RefCell::default(),
            data_series: series.into_iter().map(Into::into).collect(),
            limit: Duration::from_secs(PLOT_SECONDS as u64),
            hovered: RefCell::default(),
            range: (min_y.unwrap_or(VALUE_MIN), max_y.unwrap_or(VALUE_MAX)),
            dynamic_range: min_y.is_none() || max_y.is_none(),
            style: theme.into(),
        }
    }

    pub fn update_style(&mut self, theme: AppTheme) {
        self.style = theme.into();
        self.cache.borrow_mut().clear();
    }

    pub fn push_data(&mut self, time: DateTime<Utc>, values: [Option<f32>; N]) {
        let cutoff = time - chrono::Duration::from_std(self.limit).unwrap_or_default();

        for (ts, value) in self.data_series.iter_mut().zip(values) {
            let Some(value) = value else { continue };

            ts.data.push_front((time, value));

            if self.dynamic_range {
                self.range = (self.range.0.min(value), self.range.1.max(value));
            }

            while ts.data.back().is_some_and(|(t, _)| *t < cutoff) {
                ts.data.pop_back();
            }
        }

        if self.dynamic_range {
            self.recalculate_range();
        }

        self.cache.borrow_mut().clear();
    }

    fn recalculate_range(&mut self) {
        let (min, max) = self
            .data_series
            .iter()
            .flat_map(|s| s.data.iter().map(|(_, v)| *v))
            .fold((f32::MAX, f32::MIN), |(min, max), v| (min.min(v), max.max(v)));

        if min <= max {
            self.range = (min, max);
        }
    }

    pub fn view(&self, chart_height: f32) -> Element<'_, Message> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(5)
            .align_x(Alignment::Center)
            .push(Text::new("Sensor Chart"))
            .push(ChartWidget::new(self).height(Length::Fixed(chart_height)))
            .into()
    }

    fn time_bounds(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let newest = self
            .data_series
            .iter()
            .filter_map(|series| series.newest_time())
            .max()
            .unwrap_or_else(Utc::now);
        (newest - chrono::Duration::seconds(PLOT_SECONDS as i64), newest)
    }

    fn build_chart_2d<DB: DrawingBackend>(&self, mut builder: ChartBuilder<DB>) {
        use plotters::prelude::*;

        let style = &self.style;
        let (oldest_time, newest_time) = self.time_bounds();

        let mut chart = builder
            .x_label_area_size(X_LABEL_AREA_SIZE)
            .y_label_area_size(Y_LABEL_AREA_SIZE)
            .margin(CHART_MARGIN)
            .build_cartesian_2d(oldest_time..newest_time, self.range.0..self.range.1)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(style.grid_bold)
            .light_line_style(style.grid_light)
            .axis_style(ShapeStyle::from(style.axis).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&style.text)
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y: &f32| format!("{}%", y))
            .y_desc("Usage (%)")
            .x_label_style(("sans-serif", 15).into_font().color(&style.text))
            .x_labels(60)
            .x_label_formatter(&|x: &DateTime<Utc>| {
                let t = (newest_time.timestamp_millis() - x.timestamp_millis()) / 1000;
                if t % 5 == 0 { format!("{}", t) } else { "".to_string() }
            })
            .x_desc("Time (s)")
            .draw()
            .expect("failed to draw chart mesh");

        for (i, series) in self.data_series.iter().enumerate() {
            let color = style.series_color(i);
            let data: Vec<_> = series.iter().collect();

            let annotation = match series.line_type {
                LineType::Line => chart.draw_series(LineSeries::new(data, color)),
                LineType::Area => chart.draw_series(
                    AreaSeries::new(data, 0.0, color.mix(0.2)).border_style(ShapeStyle::from(color).stroke_width(2)),
                ),
                LineType::Dotted => chart.draw_series(DottedLineSeries::new(data, 5, 10, move |(x, y)| {
                    Circle::new((x, y), 3, color.filled())
                })),
                LineType::Points => {
                    chart.draw_series(PointSeries::of_element(data, 5, &color, &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
                    }))
                }
                LineType::Dashed => chart.draw_series(DashedLineSeries::new(
                    data,
                    5,
                    10,
                    ShapeStyle {
                        color: color.to_rgba(),
                        filled: false,
                        stroke_width: 1,
                    },
                )),
            };

            annotation
                .expect("failed to draw series")
                .label(&series.label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
        }

        chart
            .configure_series_labels()
            .border_style(&style.legend_border)
            .background_style(&style.legend_background)
            .label_font(("sans-serif", 12).into_font().color(&style.text))
            .draw()
            .expect("failed to draw legend");

        if let Some(info) = self.hovered.borrow().as_ref() {
            let hover_color = style.text.mix(0.8);
            let point = (info.time, info.value);

            chart
                .draw_series(PointSeries::of_element(
                    vec![point],
                    6,
                    ShapeStyle::from(hover_color).filled(),
                    &|coord, size, st| {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), size + 3, ShapeStyle::from(style.text).stroke_width(2))
                            + Circle::new((0, 0), size, st.clone())
                    },
                ))
                .expect("hover marker");

            chart
                .draw_series(std::iter::once(Text::new(
                    format!("{:.1}% @ {}", info.value, info.time.format("%H:%M:%S")),
                    point,
                    TextStyle::from(("sans-serif", 14).into_font()).color(&hover_color),
                )))
                .expect("failed to draw hover tooltip");
        }
    }

    fn hovered_point_at(&self, cursor: Point, bounds: Size, snap_distance: f32) -> Option<HoverInfo> {
        let chart_bounds = Size::new(
            bounds.width - Y_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN,
            bounds.height - X_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN,
        );

        if chart_bounds.width <= 0.0 || chart_bounds.height <= 0.0 {
            return None;
        }

        let chart_cursor = Point::new(cursor.x - Y_LABEL_AREA_SIZE - CHART_MARGIN, cursor.y - CHART_MARGIN);

        let (oldest, _) = self.time_bounds();
        let total_ms = self.limit.as_millis().max(1) as f32;
        let snap_sq = snap_distance * snap_distance;

        self.data_series
            .iter()
            .filter_map(|s| s.newest_time().map(|_| s))
            .flat_map(|s| s.data.iter())
            .filter_map(|(time, value)| {
                let px = self.point_x_for_time(*time, oldest, total_ms, chart_bounds.width);
                let py = self.point_y_for_value(*value, chart_bounds.height);
                let dist_sq = (px - chart_cursor.x).powi(2) + (py - chart_cursor.y).powi(2);
                (dist_sq <= snap_sq).then_some((
                    HoverInfo {
                        label: format!("{}: {:.2}%", time.format("%H:%M:%S"), value),
                        time: *time,
                        value: *value,
                    },
                    dist_sq,
                ))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(info, _)| info)
    }

    fn point_y_for_value(&self, value: f32, height: f32) -> f32 {
        let (min, max) = self.range;
        let range = max - min;
        if height <= 0.0 || range <= f32::EPSILON {
            return height / 2.0;
        }
        height * (1.0 - (value.clamp(min, max) - min) / range)
    }

    fn point_x_for_time(&self, time: DateTime<Utc>, oldest: DateTime<Utc>, total_ms: f32, width: f32) -> f32 {
        let ratio = ((time - oldest).num_milliseconds() as f32 / total_ms).clamp(0.0, 1.0);
        ratio * width
    }

    fn clear_hover(&self) -> bool {
        let mut current = self.hovered.borrow_mut();
        if current.is_some() {
            *current = None;
            self.cache.borrow_mut().clear();
            true
        } else {
            false
        }
    }

    fn update_hover(&self, new: Option<HoverInfo>) -> bool {
        let mut current = self.hovered.borrow_mut();
        if *current != new {
            *current = new;
            self.cache.borrow_mut().clear();
            true
        } else {
            false
        }
    }

    fn process_event(&self, event: Event, bounds: Rectangle, cursor: Cursor) -> (event::Status, Option<Message>) {
        let captured = match event {
            canvas::Event::Mouse(mouse::Event::CursorLeft) => self.clear_hover(),
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => cursor
                .position_in(bounds)
                .filter(|_| bounds.width > 0.0)
                .map(|pos| self.hovered_point_at(pos, bounds.size(), SNAP_DISTANCE_PX))
                .map(|h| self.update_hover(h))
                .unwrap_or_else(|| self.clear_hover()),
            _ => false,
        };

        (
            if captured {
                event::Status::Captured
            } else {
                event::Status::Ignored
            },
            None,
        )
    }
}

impl<const N: usize> Chart<Message> for SensorChart<N> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        self.process_event(event, bounds, cursor)
    }

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(&self, renderer: &R, bounds: Size, draw_fn: F) -> Geometry {
        renderer.draw_cache(&self.cache.borrow(), bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, chart: ChartBuilder<DB>) {
        self.build_chart_2d(chart);
    }
}
