use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use chrono::{DateTime, Duration, TimeZone, Utc};
use common::SensorData;
use iced::{
    Element, Length, Point, Rectangle, Size,
    alignment::Alignment,
    event::Status,
    mouse::{self, Cursor},
    widget::{
        Column, Text,
        canvas::{self, Cache, Event, Frame, Geometry},
    },
};
use plotters::{
    coord::Shift,
    prelude::ChartBuilder,
    style::{Color, RGBAColor, RGBColor},
};
use plotters_backend::DrawingBackend;
use plotters_iced2::{Chart, ChartWidget, DrawingArea, Renderer, plotters_backend};

use crate::{message::Message, themes::AppTheme};

const PLOT_SECONDS: usize = 60;
const SNAP_DISTANCE_PX: f32 = 30.0;
const VALUE_MIN: f32 = 0.0;
const VALUE_MAX: f32 = 100.0;
const X_LABEL_AREA_SIZE: f32 = 50.0;
const Y_LABEL_AREA_SIZE: f32 = 80.0;
const RIGHT_Y_LABEL_AREA_SIZE: f32 = 90.0;
const CHART_MARGIN: f32 = 20.0;
const CHART_MARGIN_LEFT: f32 = 40.0;
const CHART_MARGIN_RIGHT: f32 = 40.0;

const TOOLTIP_WIDTH: f32 = 150.0;
const TOOLTIP_MIN_HEIGHT: f32 = 60.0;
const TOOLTIP_PADDING: f32 = 8.0;
const TOOLTIP_OFFSET: f32 = 12.0;
const TOOLTIP_CORNER_RADIUS: f32 = 4.0;
const TOOLTIP_LINE_HEIGHT: f32 = 16.0;

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub grid_bold: RGBAColor,
    pub grid_light: RGBAColor,
    pub axis: RGBAColor,
    pub text: RGBAColor,
    pub legend_background: RGBAColor,
    pub legend_border: RGBColor,
    pub tooltip_background: RGBAColor,
    pub tooltip_border: RGBAColor,
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
            tooltip_background: background.mix(0.95),
            tooltip_border: text.mix(0.3),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TooltipSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipContent {
    pub title: String,
    pub value: f32,
    pub time: DateTime<Utc>,
    pub description: Option<String>,
    pub series_index: usize,
    pub axis_type: AxisType,
}

impl TooltipContent {
    pub fn new(title: String, value: f32, time: DateTime<Utc>, series_index: usize, axis_type: AxisType) -> Self {
        Self {
            title,
            value,
            time,
            description: None,
            series_index,
            axis_type,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    fn unit(&self) -> &str {
        &self.axis_type.unit()
    }

    fn value_text(&self) -> String {
        format!("{:.1}{}", self.value, self.unit())
    }

    fn timestamp_text(&self) -> String {
        self.time.format("%H:%M:%S").to_string()
    }

    pub fn calculate_height(&self) -> f32 {
        let lines = 3 + usize::from(self.description.is_some());
        (TOOLTIP_PADDING * 2.0 + lines as f32 * TOOLTIP_LINE_HEIGHT).max(TOOLTIP_MIN_HEIGHT)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipData {
    pub content: TooltipContent,
    pub point_x: f32,
    pub point_y: f32,
    pub side: TooltipSide,
    pub bounds: TooltipBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TooltipBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn corners(&self) -> [(i32, i32); 4] {
        let x1 = self.x as i32;
        let y1 = self.y as i32;
        let x2 = (self.x + self.width) as i32;
        let y2 = (self.y + self.height) as i32;
        [(x1, y1), (x2, y1), (x2, y2), (x1, y2)]
    }
}

impl TooltipData {
    pub fn new(content: TooltipContent, point_x: f32, point_y: f32, chart_width: f32, chart_height: f32) -> Self {
        let tooltip_height = content.calculate_height();

        let space_right = chart_width - point_x - TOOLTIP_OFFSET;
        let space_left = point_x - TOOLTIP_OFFSET;

        let side = if space_right >= TOOLTIP_WIDTH {
            TooltipSide::Right
        } else if space_left >= TOOLTIP_WIDTH {
            TooltipSide::Left
        } else {
            if space_right >= space_left {
                TooltipSide::Right
            } else {
                TooltipSide::Left
            }
        };

        let tooltip_x = match side {
            TooltipSide::Right => point_x + TOOLTIP_OFFSET + CHART_MARGIN_LEFT - 10.0,
            TooltipSide::Left => point_x - TOOLTIP_OFFSET - TOOLTIP_WIDTH + CHART_MARGIN_LEFT - 10.0,
        };

        let tooltip_y = (point_y - tooltip_height / 2.0)
            .max(0.0)
            .min(chart_height - tooltip_height);

        let bounds = TooltipBounds::new(
            tooltip_x.max(0.0).min(chart_width - TOOLTIP_WIDTH),
            tooltip_y,
            TOOLTIP_WIDTH,
            tooltip_height,
        );

        Self {
            content,
            point_x,
            point_y,
            side,
            bounds,
        }
    }
}

pub struct SensorChart {
    cache: RefCell<Cache>,
    data: ChartData,
    limit: Duration,
    hovered: RefCell<Option<TooltipData>>,
    y_ranges: (Range, Range),
    x_axis: AxisType,
    y_axes: (AxisType, AxisType),
    dynamic_range: bool,
    style: ChartStyle,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum LineType {
    #[default]
    Line,
    Dashed,
    Area,
    Dotted,
    Points,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AxisType {
    Primary(String, String),
    Secondary(String, String),
}

impl AxisType {
    pub fn label(&self) -> &str {
        match self {
            AxisType::Primary(label, _) | AxisType::Secondary(label, _) => label,
        }
    }

    pub fn unit(&self) -> &str {
        match self {
            AxisType::Primary(_, unit) | AxisType::Secondary(_, unit) => unit,
        }
    }
}

impl Default for AxisType {
    fn default() -> Self {
        AxisType::Primary("Value".to_string(), "Units".to_string())
    }
}

#[derive(Default, Debug, Clone)]
struct TimeSeries {
    points: VecDeque<(DateTime<Utc>, f32)>,
    line_type: LineType,
    axis_type: AxisType,
}

impl TimeSeries {
    fn iter(&self) -> impl Iterator<Item = (DateTime<Utc>, f32)> + '_ {
        self.points.iter().copied()
    }

    fn newest_time(&self) -> Option<DateTime<Utc>> {
        self.points.back().map(|(time, _)| *time)
    }

    fn oldest_time(&self) -> Option<DateTime<Utc>> {
        self.points.front().map(|(time, _)| *time)
    }
}

#[derive(Debug, Clone, Default)]
struct ChartData {
    series: HashMap<String, TimeSeries>,
}

pub type SeriesSettings = Vec<(String, LineType, AxisType)>;

impl ChartData {
    fn from(series: SeriesSettings) -> Self {
        let mut ts = Self::default();
        for (label, line_type, axis_type) in series {
            ts.series.insert(
                label,
                TimeSeries {
                    points: VecDeque::new(),
                    line_type,
                    axis_type,
                },
            );
        }
        ts
    }

    fn newest_time(&self) -> Option<DateTime<Utc>> {
        self.series.values().filter_map(|series| series.newest_time()).max()
    }

    fn oldest_time(&self) -> Option<DateTime<Utc>> {
        self.series.values().filter_map(|series| series.oldest_time()).min()
    }

    fn push_data(&mut self, time: DateTime<Utc>, label: String, value: f32) {
        if let Some(series) = self.series.get_mut(&label) {
            series.points.push_back((time, value));
        } else {
            let mut series = TimeSeries::default();
            series.points.push_back((time, value));
            self.series.insert(label, series);
        }
    }
}

fn to_plotters_color(color: iced::Color) -> RGBColor {
    let rgba = color.into_rgba8();
    RGBColor(rgba[0], rgba[1], rgba[2])
}

impl SensorChart {
    pub fn new(
        series: SeriesSettings,
        min_y: Option<f32>,
        max_y: Option<f32>,
        theme: AppTheme,
        x_axis: AxisType,
        y_axes: (AxisType, AxisType),
    ) -> Self {
        Self {
            cache: RefCell::default(),
            data: ChartData::from(series),
            limit: Duration::seconds(PLOT_SECONDS as i64),
            hovered: RefCell::default(),
            y_ranges: (
                (min_y.unwrap_or(VALUE_MIN), max_y.unwrap_or(VALUE_MAX)),
                (VALUE_MIN, VALUE_MAX),
            ),
            dynamic_range: min_y.is_none() || max_y.is_none(),
            style: theme.into(),
            x_axis,
            y_axes,
        }
    }

    pub fn update_style(&mut self, theme: AppTheme) {
        self.style = theme.into();
        self.cache.borrow_mut().clear();
    }

    pub fn push_data(&mut self, data: Vec<(DateTime<Utc>, SensorData)>) {
        if data.is_empty() {
            return;
        }

        let mut max = Utc.timestamp_opt(0, 0).unwrap();
        for (time, value) in data {
            if time > max {
                max = time;
            }

            match value {
                SensorData::CPU(cpu_data) => {
                    if let Some(power) = cpu_data.total_power_watts {
                        self.data.push_data(time, "CPU Power".to_string(), power as f32);
                    }
                    self.data
                        .push_data(time, "CPU Usage".to_string(), cpu_data.usage_percent as f32);
                }
                SensorData::GPU(gpu_data) => {
                    if let Some(power) = gpu_data.total_power_watts {
                        self.data.push_data(time, "GPU Power".to_string(), power as f32);
                    }
                    if let Some(usage) = gpu_data.usage_percent {
                        self.data.push_data(time, "GPU Usage".to_string(), usage as f32);
                    }
                }
                _ => {}
            }
        }

        let cutoff = max - self.limit;
        for series in self.data.series.values_mut() {
            series.points.retain(|(time, _)| *time >= cutoff);
        }

        if self.dynamic_range {
            self.recalculate_range();
        }

        self.cache.borrow_mut().clear();
    }

    fn recalculate_range(&mut self) {
        let (min, max) = self
            .data
            .series
            .values()
            .filter(|series| matches!(series.axis_type, AxisType::Primary(_, _)))
            .flat_map(|series| series.points.iter().map(|(_, v)| *v))
            .fold((f32::MAX, f32::MIN), |(min, max), v| (min.min(v), max.max(v)));

        if min <= max {
            self.y_ranges.0 = (min, max);
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
        let newest = self.data.newest_time().unwrap_or_else(Utc::now);
        let oldest = newest - self.limit;
        (oldest, newest)
    }

    fn build_chart_2d<DB: DrawingBackend>(&self, mut builder: ChartBuilder<DB>) {
        use plotters::prelude::*;

        let style = &self.style;
        let (oldest_time, newest_time) = self.time_bounds();
        let label_style = ("sans-serif", 15).into_font().color(&style.text);

        let mut chart = builder
            .x_label_area_size(X_LABEL_AREA_SIZE)
            .y_label_area_size(Y_LABEL_AREA_SIZE)
            .right_y_label_area_size(RIGHT_Y_LABEL_AREA_SIZE)
            .margin(CHART_MARGIN)
            .margin_left(CHART_MARGIN_LEFT)
            .margin_right(CHART_MARGIN_RIGHT)
            .build_cartesian_2d(oldest_time..newest_time, self.y_ranges.0.0..self.y_ranges.0.1)
            .expect("failed to build chart")
            .set_secondary_coord(oldest_time..newest_time, self.y_ranges.1.0..self.y_ranges.1.1);

        chart
            .configure_mesh()
            .bold_line_style(style.grid_bold)
            .light_line_style(style.grid_light)
            .axis_style(ShapeStyle::from(style.axis).stroke_width(1))
            .y_labels(5)
            .y_label_style(label_style.clone())
            .y_label_formatter(&|y: &f32| format!("{}{}", y, self.y_axes.0.unit()))
            .y_desc(format!("{} ({})", self.y_axes.0.label(), self.y_axes.0.unit()))
            .axis_desc_style(label_style.clone().transform(FontTransform::Rotate90))
            .x_label_style(label_style.clone())
            .x_labels(60)
            .x_label_formatter(&|x: &DateTime<Utc>| {
                let t = (newest_time.timestamp_millis() - x.timestamp_millis()) / 1000;
                if t % 5 == 0 { format!("{}", t) } else { "".to_string() }
            })
            .x_desc(format!("{} ({})", self.x_axis.label(), self.x_axis.unit()))
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .configure_secondary_axes()
            .axis_style(ShapeStyle::from(style.axis).stroke_width(1))
            .y_labels(10)
            .y_label_formatter(&|y: &f32| format!("{}{}", y, self.y_axes.1.unit()))
            .y_desc(format!("{} ({})", self.y_axes.1.label(), self.y_axes.1.unit()))
            .axis_desc_style(label_style.clone().transform(FontTransform::Rotate90))
            .label_style(label_style.clone())
            .draw()
            .expect("failed to draw secondary axes");

        for (i, (label, series)) in self.data.series.iter().enumerate() {
            let color = style.series_color(i);
            let data: Vec<_> = series.iter().collect();
            let is_secondary = matches!(series.axis_type, AxisType::Secondary(_, _));

            macro_rules! draw {
                ($series_expr:expr) => {
                    if is_secondary {
                        chart.draw_secondary_series($series_expr)
                    } else {
                        chart.draw_series($series_expr)
                    }
                };
            }
            let annotation = match series.line_type {
                LineType::Line => draw!(LineSeries::new(data, color.stroke_width(2))),
                LineType::Area => draw!(
                    AreaSeries::new(data, 0.0, color.mix(0.2)).border_style(ShapeStyle::from(color).stroke_width(2))
                ),
                LineType::Dotted => draw!(DottedLineSeries::new(data, 5, 10, move |(x, y)| {
                    Circle::new((x, y), 3, color.filled())
                })),
                LineType::Points => draw!(PointSeries::of_element(data, 5, &color, &|coord, size, style| {
                    EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
                })),
                LineType::Dashed => draw!(DashedLineSeries::new(
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
                .label(label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
        }

        chart
            .configure_series_labels()
            .border_style(&style.legend_border)
            .background_style(&style.legend_background)
            .label_font(("sans-serif", 12).into_font().color(&style.text))
            .draw()
            .expect("failed to draw legend");

        if let Some(tooltip) = self.hovered.borrow().as_ref() {
            let series_color = style.series_color(tooltip.content.series_index);
            let point = (tooltip.content.time, tooltip.content.value);
            let is_secondary = matches!(tooltip.content.axis_type, AxisType::Secondary(_, _));

            macro_rules! draw {
                ($series_expr:expr) => {
                    if is_secondary {
                        chart.draw_secondary_series($series_expr)
                    } else {
                        chart.draw_series($series_expr)
                    }
                };
            }
            draw!(PointSeries::of_element(
                vec![point],
                6,
                ShapeStyle::from(series_color).filled(),
                &|coord, size, st| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size + 3, ShapeStyle::from(style.text).stroke_width(2))
                        + Circle::new((0, 0), size, st.clone())
                },
            ))
            .expect("failed to draw hover marker");

            let backend_area = chart.plotting_area().strip_coord_spec();
            self.draw_tooltip_on_backend(&backend_area, tooltip, style);
        }
    }

    fn draw_tooltip_on_backend<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, Shift>,
        tooltip: &TooltipData,
        style: &ChartStyle,
    ) {
        use plotters::prelude::*;

        let bounds = &tooltip.bounds;
        let content = &tooltip.content;
        let series_color = style.series_color(content.series_index);
        let series_color_rgba = series_color.to_rgba();

        let rect_style = ShapeStyle {
            color: style.tooltip_background,
            filled: true,
            stroke_width: 0,
        };

        let border_style = ShapeStyle {
            color: style.tooltip_border,
            filled: false,
            stroke_width: 1,
        };

        let x1 = bounds.x as i32;
        let y1 = bounds.y as i32;
        let x2 = (bounds.x + bounds.width) as i32;
        let y2 = (bounds.y + bounds.height) as i32;

        area.draw(&Rectangle::new([(x1, y1), (x2, y2)], rect_style)).ok();

        area.draw(&Rectangle::new([(x1, y1), (x2, y2)], border_style)).ok();

        let indicator_width = 4;
        area.draw(&Rectangle::new(
            [(x1, y1), (x1 + indicator_width, y2)],
            ShapeStyle::from(series_color).filled(),
        ))
        .ok();

        let text_x = x1 + indicator_width + TOOLTIP_PADDING as i32;
        let mut text_y = y1 + TOOLTIP_PADDING as i32;

        let text_style = TextStyle::from(("sans-serif", 12).into_font()).color(&style.text);
        let title_style = TextStyle::from(("sans-serif", 12).into_font()).color(&series_color_rgba);

        area.draw(&Text::new(content.title.clone(), (text_x, text_y), title_style))
            .ok();
        text_y += TOOLTIP_LINE_HEIGHT as i32;

        area.draw(&Text::new(
            format!("Value: {}", content.value_text()),
            (text_x, text_y),
            text_style.clone(),
        ))
        .ok();
        text_y += TOOLTIP_LINE_HEIGHT as i32;

        area.draw(&Text::new(
            format!("Time: {}", content.timestamp_text()),
            (text_x, text_y),
            text_style.clone(),
        ))
        .ok();
        text_y += TOOLTIP_LINE_HEIGHT as i32;

        if let Some(desc) = &content.description {
            area.draw(&Text::new(desc.clone(), (text_x, text_y), text_style)).ok();
        }
    }

    fn hovered_point_at(&self, cursor: Point, bounds: Size, snap_distance: f32) -> Option<TooltipData> {
        let chart_bounds = Size::new(
            bounds.width
                - Y_LABEL_AREA_SIZE
                - 2.0 * CHART_MARGIN
                - CHART_MARGIN_LEFT
                - CHART_MARGIN_RIGHT
                - RIGHT_Y_LABEL_AREA_SIZE,
            bounds.height - X_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN,
        );

        if chart_bounds.width <= 0.0 || chart_bounds.height <= 0.0 {
            return None;
        }

        let chart_cursor = Point::new(
            cursor.x - Y_LABEL_AREA_SIZE - CHART_MARGIN - CHART_MARGIN_LEFT,
            cursor.y - CHART_MARGIN,
        );

        let (oldest, _) = self.time_bounds();
        let total_ms = self.limit.num_milliseconds() as f32;
        let snap_sq = snap_distance * snap_distance;

        self.data
            .series
            .iter()
            .enumerate()
            .filter_map(|(idx, (label, s))| s.newest_time().map(|_| (idx, label.clone(), s)))
            .flat_map(|(idx, label, s)| {
                s.points
                    .iter()
                    .map(move |d| (idx, label.clone(), d, s.axis_type.clone()))
            })
            .filter_map(|(series_idx, label, (time, value), axis_type)| {
                let px = self.point_x_for_time(*time, oldest, total_ms, chart_bounds.width);
                let py = self.point_y_for_value(*value, chart_bounds.height, &axis_type);
                let dist_sq = (px - chart_cursor.x).powi(2) + (py - chart_cursor.y).powi(2);

                if dist_sq <= snap_sq {
                    let content = TooltipContent::new(label, *value, *time, series_idx, axis_type);
                    let tooltip = TooltipData::new(content, px, py, chart_bounds.width, chart_bounds.height);
                    Some((tooltip, dist_sq))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(tooltip, _)| tooltip)
    }

    fn point_y_for_value(&self, value: f32, height: f32, axis_type: &AxisType) -> f32 {
        let (min, max) = match axis_type {
            AxisType::Primary(_, _) => self.y_ranges.0,
            AxisType::Secondary(_, _) => self.y_ranges.1,
        };

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

    fn update_hover(&self, new: Option<TooltipData>) -> bool {
        let mut current = self.hovered.borrow_mut();
        if *current != new {
            *current = new;
            self.cache.borrow_mut().clear();
            true
        } else {
            false
        }
    }

    fn process_event(&self, event: &Event, bounds: Rectangle, cursor: Cursor) -> (Status, Option<Message>) {
        let captured = match *event {
            canvas::Event::Mouse(mouse::Event::CursorLeft) => self.clear_hover(),
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => cursor
                .position_in(bounds)
                .filter(|_| bounds.width > 0.0)
                .map(|pos| self.hovered_point_at(pos, bounds.size(), SNAP_DISTANCE_PX))
                .map(|h| self.update_hover(h))
                .unwrap_or_else(|| self.clear_hover()),
            _ => false,
        };

        let status = if captured { Status::Captured } else { Status::Ignored };
        let message = captured.then_some(Message::Redraw);
        (status, message)
    }
}

impl Chart<Message> for SensorChart {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<Message>) {
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
