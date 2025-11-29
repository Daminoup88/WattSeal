use std::{cell::RefCell, collections::VecDeque};

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

impl ChartStyle {
    pub fn from_theme(theme: AppTheme) -> Self {
        let palette = theme.palette();
        let text = to_plotters_color(palette.text);
        let background = to_plotters_color(palette.background);
        let primary = to_plotters_color(palette.primary);
        let success = to_plotters_color(palette.success);
        let danger = to_plotters_color(palette.danger);

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

    pub fn series_color(&self, index: usize) -> RGBColor {
        self.series_colors[index % self.series_colors.len()]
    }
}

struct ChartRange {
    min: f32,
    max: f32,
}

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
    range: ChartRange,
    dynamic_range: bool,
    style: ChartStyle,
}

pub enum LineType {
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

impl TimeSeries {
    fn push_front(&mut self, value: (DateTime<Utc>, f32)) {
        self.data.push_front(value);
    }

    fn pop_back(&mut self) -> Option<(DateTime<Utc>, f32)> {
        self.data.pop_back()
    }

    fn iter(&self) -> impl Iterator<Item = (DateTime<Utc>, f32)> {
        self.data.iter().map(|(time, value)| (*time, *value))
    }

    fn vec(&self) -> Vec<(DateTime<Utc>, f32)> {
        self.data.iter().map(|(time, value)| (*time, *value)).collect()
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
        let data: Vec<TimeSeries> = series
            .into_iter()
            .map(|(label, line_type)| TimeSeries {
                label,
                data: VecDeque::new(),
                line_type,
            })
            .collect();

        Self {
            cache: RefCell::new(Cache::new()),
            data_series: data,
            limit: Duration::from_secs(PLOT_SECONDS as u64),
            hovered: RefCell::new(None),
            range: ChartRange {
                min: min_y.unwrap_or(VALUE_MIN),
                max: max_y.unwrap_or(VALUE_MAX),
            },
            dynamic_range: min_y.is_none() || max_y.is_none(),
            style: ChartStyle::from_theme(theme),
        }
    }

    pub fn update_style(&mut self, theme: AppTheme) {
        self.style = ChartStyle::from_theme(theme);
        self.cache.borrow_mut().clear();
    }

    pub fn push_data(&mut self, time: DateTime<Utc>, series: [Option<f32>; N]) {
        let cur_ms = time.timestamp_millis();
        for (ts, value) in self.data_series.iter_mut().zip(series) {
            let value = match value {
                Some(v) => v,
                None => continue,
            };
            ts.push_front((time, value));
            if self.dynamic_range {
                if value < self.range.min {
                    self.range.min = value;
                }
                if value > self.range.max {
                    self.range.max = value;
                }
            }
            while let Some(&(old_time, _)) = ts.data.back() {
                if cur_ms - old_time.timestamp_millis() > self.limit.as_millis() as i64 {
                    let p = ts.pop_back();
                    if self.dynamic_range {
                        if let Some((_, old_value)) = p {
                            if (old_value - self.range.min).abs() < f32::EPSILON
                                || (old_value - self.range.max).abs() < f32::EPSILON
                            {
                                let mut new_min = f32::MAX;
                                let mut new_max = f32::MIN;
                                for &(_, v) in ts.data.iter() {
                                    if v < new_min {
                                        new_min = v;
                                    }
                                    if v > new_max {
                                        new_max = v;
                                    }
                                }
                                self.range.min = new_min;
                                self.range.max = new_max;
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        }
        self.cache.borrow_mut().clear();
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

    fn build_chart_2d<DB: DrawingBackend>(&self, mut chart: ChartBuilder<DB>) {
        use plotters::prelude::*;

        let style = &self.style;

        let newest_time = self
            .data_series
            .iter()
            .flat_map(|series| series.data.front().map(|(time, _)| *time))
            .max()
            .unwrap_or_else(|| Utc::now());
        let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);

        let mut chart = chart
            .x_label_area_size(X_LABEL_AREA_SIZE)
            .y_label_area_size(Y_LABEL_AREA_SIZE)
            .margin(CHART_MARGIN)
            .build_cartesian_2d(oldest_time..newest_time, self.range.min..self.range.max)
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
            let series_anno = match series.line_type {
                LineType::Line => chart.draw_series(LineSeries::new(series.iter(), color)),
                LineType::Area => chart.draw_series(
                    AreaSeries::new(series.vec(), 0.0, color.mix(0.2))
                        .border_style(ShapeStyle::from(color).stroke_width(2)),
                ),
                LineType::Dotted => chart.draw_series(DottedLineSeries::new(series.vec(), 5, 10, {
                    move |(x, y)| Circle::new((x, y), 3, color.filled())
                })),
                LineType::Points => chart.draw_series(PointSeries::of_element(
                    series.iter(),
                    5,
                    &color,
                    &|coord, size, style| EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled()),
                )),
                LineType::Dashed => chart.draw_series(DashedLineSeries::new(
                    series.vec(),
                    5,
                    10,
                    ShapeStyle {
                        color: color.to_rgba(),
                        filled: false,
                        stroke_width: 1,
                    },
                )),
            };

            series_anno
                .expect("failed to draw chart data")
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
            let hover_color = style.text;
            chart
                .draw_series(PointSeries::of_element(
                    vec![(info.time, info.value)],
                    6,
                    ShapeStyle::from(hover_color.mix(0.8)).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), size + 3, ShapeStyle::from(hover_color).stroke_width(2))
                            + Circle::new((0, 0), size, style.clone())
                    },
                ))
                .expect("failed to draw hover marker");

            let label = format!("{:.1}% @ {}", info.value, info.time.format("%H:%M:%S"));

            chart
                .draw_series(std::iter::once(Text::new(
                    label,
                    (info.time, info.value),
                    TextStyle::from(("sans-serif", 14).into_font()).color(&hover_color.mix(0.8)),
                )))
                .expect("failed to draw hover tooltip");
        }
    }

    fn hovered_point_at(&self, cursor: Point, bounds: Size, snap_distance: f32) -> Option<HoverInfo> {
        let chart_area_bounds = Size::new(
            bounds.width - Y_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN,
            bounds.height - X_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN,
        );
        let chart_area_cursor = Point::new(cursor.x - Y_LABEL_AREA_SIZE - CHART_MARGIN, cursor.y - CHART_MARGIN);
        if chart_area_bounds.width <= 0.0 || chart_area_bounds.height <= 0.0 {
            return None;
        }
        let mut candidate: Option<(HoverInfo, f32)> = None;
        let chrono_limit =
            chrono::Duration::from_std(self.limit).unwrap_or_else(|_| chrono::Duration::seconds(PLOT_SECONDS as i64));
        let total_ms = chrono_limit.num_milliseconds().max(1) as f32;

        for series in &self.data_series {
            let newest_time_opt = series.newest_time();
            let newest_time = match newest_time_opt {
                Some(t) => t,
                None => continue,
            };

            let oldest_time_limit = newest_time - chrono_limit;
            let snap_distance_sq = snap_distance * snap_distance;
            self.update_hover_candidate(
                &mut candidate,
                &series.data,
                chart_area_cursor,
                chart_area_bounds,
                oldest_time_limit,
                total_ms,
                snap_distance_sq,
            );
        }
        candidate.map(|(info, _)| info)
    }

    fn update_hover_candidate(
        &self,
        candidate: &mut Option<(HoverInfo, f32)>,
        series: &VecDeque<(DateTime<Utc>, f32)>,
        cursor: Point,
        bounds: Size,
        oldest_time: DateTime<Utc>,
        total_ms: f32,
        snap_distance_sq: f32,
    ) {
        for (time, value) in series.iter() {
            let px = self.point_x_for_time(*time, oldest_time, total_ms, bounds.width);
            let py = self.point_y_for_value(*value, bounds.height);
            let dx = px - cursor.x;
            let dy = py - cursor.y;
            let distance_sq = dx * dx + dy * dy;
            if distance_sq <= snap_distance_sq {
                let info = HoverInfo {
                    label: format!("{}: {:.2}%", time.format("%H:%M:%S"), value),
                    time: *time,
                    value: *value,
                };

                match candidate {
                    None => *candidate = Some((info, distance_sq)),
                    Some((_, best_distance)) if distance_sq < *best_distance => *candidate = Some((info, distance_sq)),
                    _ => {}
                }
            }
        }
    }

    fn point_y_for_value(&self, value: f32, height: f32) -> f32 {
        if height <= 0.0 {
            return 0.0;
        }

        let range = self.range.max - self.range.min;
        if range <= f32::EPSILON {
            return height / 2.0;
        }

        let clamped = value.clamp(self.range.min, self.range.max);
        let ratio = (clamped - self.range.min) / range;
        height - (ratio * height)
    }

    fn point_x_for_time(&self, time: DateTime<Utc>, oldest_time: DateTime<Utc>, total_ms: f32, width: f32) -> f32 {
        let elapsed = (time - oldest_time).num_milliseconds() as f32;
        let clamped_ratio = (elapsed / total_ms).clamp(0.0, 1.0);
        clamped_ratio * width
    }

    fn process_event(&self, event: Event, bounds: Rectangle, cursor: Cursor) -> (event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                let mut current = self.hovered.borrow_mut();
                if current.is_some() {
                    *current = None;
                    self.cache.borrow_mut().clear();
                    return (event::Status::Captured, None);
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(position) = cursor.position_in(bounds) {
                    if bounds.width > 0.0 {
                        let chart_size = Size::new(bounds.width, bounds.height);
                        let hovered = self.hovered_point_at(position, chart_size, SNAP_DISTANCE_PX);
                        let mut current = self.hovered.borrow_mut();
                        if hovered != *current {
                            *current = hovered;
                            self.cache.borrow_mut().clear();
                            return (event::Status::Captured, None);
                        }
                    }
                } else {
                    let mut current = self.hovered.borrow_mut();
                    if current.is_some() {
                        *current = None;
                        self.cache.borrow_mut().clear();
                        return (event::Status::Captured, None);
                    }
                }
            }
            _ => {}
        }
        (event::Status::Ignored, None)
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
