use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use chrono::{DateTime, Duration, Local, TimeZone, Timelike};
use common::SensorData;
use iced::{
    Element, Length, Point, Rectangle, Size,
    advanced::graphics::text::cosmic_text::skrifa::color,
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
    data,
    prelude::ChartBuilder,
    style::{Color, RGBAColor, RGBColor},
};
use plotters_backend::DrawingBackend;
use plotters_iced2::{Chart, ChartWidget, DrawingArea, Renderer, plotters_backend};

use crate::{
    message::Message,
    themes::AppTheme,
    translations::{self, tooltip_time, tooltip_value},
    types::AppLanguage,
};

const PLOT_SECONDS: usize = 60;
const SNAP_DISTANCE_PX: f32 = 30.0;
const Y_LABELS_COUNT: usize = 5;
const VALUE_MIN: f32 = 0.0;
const VALUE_MAX: f32 = 100.0;
const X_LABEL_AREA_SIZE: f32 = 15.0;
const Y_LABEL_AREA_SIZE: f32 = 50.0;
// const RIGHT_Y_LABEL_AREA_SIZE: f32 = 90.0;
const CHART_MARGIN: f32 = 20.0;
const CHART_MARGIN_LEFT: f32 = 0.0;
const CHART_MARGIN_RIGHT: f32 = 10.0;

const TOOLTIP_WIDTH: f32 = 160.0;
const TOOLTIP_MIN_HEIGHT: f32 = 60.0;
const TOOLTIP_PADDING: f32 = 8.0;
const TOOLTIP_OFFSET: f32 = 20.0;
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
    pub unit: String,
    pub time: DateTime<Local>,
    pub description: Option<String>,
    pub series_index: usize,
    pub color_index: Option<usize>,
    pub x_range_secs: i64,
}

impl TooltipContent {
    pub fn new(
        title: String,
        value: f32,
        unit: String,
        time: DateTime<Local>,
        series_index: usize,
        color_index: Option<usize>,
        x_range_secs: i64,
    ) -> Self {
        Self {
            title,
            value,
            unit,
            time,
            description: None,
            series_index,
            color_index,
            x_range_secs,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    fn value_text(&self) -> String {
        let decimals = if self.value < 1.0 { 2 } else { 1 };
        format!("{:.*}{}", decimals, self.value, self.unit)
    }

    fn timestamp_text(&self) -> String {
        if self.x_range_secs > 86400 {
            self.time.format("%Y-%m-%d %H:%M").to_string()
        } else {
            self.time.format("%H:%M:%S").to_string()
        }
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
            TooltipSide::Right => point_x + TOOLTIP_OFFSET,
            TooltipSide::Left => point_x - TOOLTIP_OFFSET - TOOLTIP_WIDTH,
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
    hovered: RefCell<Option<TooltipData>>,
    x_range: Duration,
    y_range: Range,
    y_label_area_size: f32,
    x_unit: &'static str,
    y_unit: &'static str,
    dynamic_range: bool,
    style: ChartStyle,
    language: AppLanguage,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum LineType {
    #[default]
    Line,
    Dashed,
    Area,
    Dotted,
    Points,
    Step,
}

#[derive(Default, Debug, Clone)]
struct TimeSeries {
    points: Rc<RefCell<VecDeque<(DateTime<Local>, f32)>>>,
    line_type: LineType,
    color_index: Option<usize>,
    display_label: String,
}

impl TimeSeries {
    fn iter(&self) -> Vec<(DateTime<Local>, f32)> {
        self.points
            .try_borrow()
            .map(|points| points.iter().copied().collect())
            .unwrap_or_default()
    }

    fn steps_iter(&self) -> Vec<(DateTime<Local>, f32)> {
        self.points
            .try_borrow()
            .map(|points| {
                let pts: Vec<_> = points.iter().copied().collect();
                if pts.len() < 2 {
                    return pts;
                }
                let mut result = Vec::with_capacity(pts.len() * 2);
                for i in 0..pts.len() {
                    let (time, value) = pts[i];
                    result.push((time, value));
                    if i + 1 < pts.len() {
                        let (next_time, _) = pts[i + 1];
                        result.push((next_time, value));
                    }
                }
                result
            })
            .unwrap_or_default()
    }

    fn newest_time(&self) -> Option<DateTime<Local>> {
        self.points
            .try_borrow()
            .ok()
            .and_then(|points| points.back().map(|(t, _)| *t))
    }

    fn oldest_time(&self) -> Option<DateTime<Local>> {
        self.points
            .try_borrow()
            .ok()
            .and_then(|points| points.front().map(|(t, _)| *t))
    }
}

type ChartData = HashMap<String, TimeSeries>;

fn to_plotters_color(color: iced::Color) -> RGBColor {
    let rgba = color.into_rgba8();
    RGBColor(rgba[0], rgba[1], rgba[2])
}

impl SensorChart {
    fn clear_cache(&self) {
        if let Ok(cache) = self.cache.try_borrow_mut() {
            cache.clear();
        }
    }

    pub fn new(theme: AppTheme, language: AppLanguage) -> Self {
        Self {
            cache: RefCell::default(),
            data: ChartData::default(),
            hovered: RefCell::default(),
            x_range: Duration::seconds(PLOT_SECONDS as i64),
            y_range: (VALUE_MIN, VALUE_MAX),
            y_label_area_size: Y_LABEL_AREA_SIZE,
            dynamic_range: true,
            style: theme.into(),
            x_unit: "",
            y_unit: "",
            language,
        }
    }

    pub fn add_series(&mut self, key: &str, display_label: &str, line_type: LineType, color_idx: Option<usize>) {
        self.data.insert(
            key.to_string(),
            TimeSeries {
                points: Rc::new(RefCell::new(VecDeque::new())),
                line_type,
                color_index: color_idx,
                display_label: display_label.to_string(),
            },
        );
    }

    pub fn set_all_line_types(&mut self, line_type: LineType) {
        for series in self.data.values_mut() {
            series.line_type = line_type;
        }
    }

    pub fn remove_series(&mut self, label: &str) {
        self.data.remove(label);
    }

    pub fn set_data(&mut self, label: &str, points: Rc<RefCell<VecDeque<(DateTime<Local>, f32)>>>) {
        if let Some(series) = self.data.get_mut(label) {
            series.points = points;
            self.refresh_cache();
        }
    }

    pub fn refresh_cache(&mut self) {
        if self.dynamic_range {
            self.recalculate_range();
        }
        self.clear_cache();
    }

    pub fn set_x_axis_unit(&mut self, unit: &'static str) {
        self.x_unit = unit;
    }

    pub fn set_y_axis_unit(&mut self, unit: &'static str) {
        self.y_unit = unit;
        self.recalculate_y_label_area_size();
        self.clear_cache();
    }

    pub fn newest_time(&self) -> Option<DateTime<Local>> {
        self.data.values().filter_map(|series| series.newest_time()).max()
    }

    pub fn oldest_time(&self) -> Option<DateTime<Local>> {
        self.data.values().filter_map(|series| series.oldest_time()).min()
    }

    pub fn clear_all(&mut self) {
        self.data.clear();
        self.clear_cache();
    }

    pub fn update_style(&mut self, theme: AppTheme) {
        self.style = theme.into();
        self.clear_cache();
    }

    pub fn update_language(&mut self, language: AppLanguage) {
        self.language = language;
        self.clear_cache();
    }

    pub fn set_x_range(&mut self, duration: Duration) {
        self.x_range = duration;
        self.clear_cache();
    }

    fn recalculate_y_label_area_size(&mut self) {
        let mut longest_label_length = 0;

        let (min, max) = self.y_range;
        for i in 0..Y_LABELS_COUNT {
            let t = i as f32 / (Y_LABELS_COUNT - 1) as f32;
            let value = min + t * (max - min);
            let s = self.format_y_label(value);
            longest_label_length = longest_label_length.max(s.chars().count());
        }

        if longest_label_length <= 4 {
            self.y_label_area_size = Y_LABEL_AREA_SIZE;
            return;
        }
        let extra_per_char = 8;
        self.y_label_area_size = Y_LABEL_AREA_SIZE + ((longest_label_length - 4) * extra_per_char) as f32;
    }

    fn recalculate_range(&mut self) {
        let mut max = f32::MIN;
        for series in self.data.values() {
            if let Ok(points) = series.points.try_borrow() {
                for &(_, value) in points.iter() {
                    if value > max {
                        max = value;
                    }
                }
            }
        }

        if max >= 0.0 {
            self.y_range = (0.0, max);
        }

        if self.y_range.1 == self.y_range.0 {
            self.y_range = (VALUE_MIN, VALUE_MAX)
        }
        self.recalculate_y_label_area_size();
    }

    pub fn view(&self, height: f32) -> Element<'_, Message, AppTheme> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(5)
            .align_x(Alignment::Center)
            .push(ChartWidget::new(self).height(Length::Fixed(height)))
            .into()
    }

    fn time_bounds(&self) -> (DateTime<Local>, DateTime<Local>) {
        let newest = self.newest_time().unwrap_or(Local::now());
        let oldest = newest - self.x_range;
        (oldest, newest)
    }

    fn format_x_label(&self, x: &DateTime<Local>, total_secs: i64, newest: &DateTime<Local>) -> String {
        match total_secs {
            0..=120 => {
                let seconds_ago = (newest.timestamp() - x.timestamp()).max(0);
                if seconds_ago % 10 == 0 {
                    format!("{}s", seconds_ago)
                } else {
                    "".to_string()
                }
            }
            121..=86400 => x.format("%H:%M").to_string(),
            _ => {
                if x.hour() == 0 && x.minute() == 0 {
                    x.format("%Y-%m-%d").to_string()
                } else {
                    "".to_string()
                }
            }
        }
    }

    fn format_y_label(&self, y: f32) -> String {
        let decimals = if y < 1.0 && y.fract() > 0.0 { 2 } else { 0 };
        format!("{:.*}{}", decimals, y, self.y_unit)
    }

    fn build_chart_2d<DB: DrawingBackend>(&self, mut builder: ChartBuilder<DB>) {
        use plotters::prelude::*;

        let style = &self.style;
        let (oldest_time, newest_time) = self.time_bounds();
        let label_style = ("sans-serif", 15).into_font().color(&style.text);

        let x_seconds = self.x_range.num_seconds();

        let mut chart = match builder
            .x_label_area_size(X_LABEL_AREA_SIZE)
            .y_label_area_size(self.y_label_area_size)
            .margin(CHART_MARGIN)
            .margin_left(CHART_MARGIN_LEFT)
            .margin_right(CHART_MARGIN_RIGHT)
            .build_cartesian_2d(oldest_time..newest_time, self.y_range.0..self.y_range.1)
        {
            Ok(chart) => chart,
            Err(e) => {
                eprintln!("failed to build chart: {}", e);
                return;
            }
        };

        chart
            .configure_mesh()
            .max_light_lines(1)
            .bold_line_style(style.grid_bold)
            .light_line_style(style.grid_light)
            .axis_style(ShapeStyle::from(style.axis).stroke_width(1))
            .y_labels(Y_LABELS_COUNT)
            .y_label_style(label_style.clone())
            .y_label_formatter(&|y| self.format_y_label(*y))
            .x_label_style(label_style.clone())
            .x_labels(if x_seconds <= 120 { x_seconds as usize } else { 7 })
            .x_label_formatter(&|x: &DateTime<Local>| self.format_x_label(x, x_seconds, &newest_time))
            .draw()
            .ok();

        for (i, (_key, series)) in self.data.iter().enumerate() {
            let color = series
                .color_index
                .map(|idx| style.series_color(idx))
                .unwrap_or_else(|| style.series_color(i));
            let data = match series.line_type {
                LineType::Step => series.steps_iter(),
                _ => series.iter(),
            };

            let annotation = match series.line_type {
                LineType::Line => chart.draw_series(LineSeries::new(data, color.stroke_width(2))),
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
                LineType::Step => chart.draw_series(LineSeries::new(data, color.stroke_width(2))),
            };

            if let Some(anno) = annotation.ok() {
                anno.label(format!("{}   ", series.display_label))
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
            }
        }

        chart
            .configure_series_labels()
            .border_style(&style.legend_border)
            .background_style(&style.legend_background)
            .label_font(("sans-serif", 12).into_font().color(&style.text))
            .draw()
            .ok();

        if let Ok(hovered) = self.hovered.try_borrow()
            && let Some(tooltip) = hovered.as_ref()
        {
            let series_color = tooltip
                .content
                .color_index
                .map(|idx| style.series_color(idx))
                .unwrap_or_else(|| style.series_color(tooltip.content.series_index));
            let point = (tooltip.content.time, tooltip.content.value);
            chart
                .draw_series(PointSeries::of_element(
                    vec![point],
                    6,
                    ShapeStyle::from(series_color).filled(),
                    &|coord, size, st| {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), size + 3, ShapeStyle::from(style.text).stroke_width(2))
                            + Circle::new((0, 0), size, st.clone())
                    },
                ))
                .ok();

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
        let series_color = content
            .color_index
            .map(|idx| style.series_color(idx))
            .unwrap_or_else(|| style.series_color(content.series_index));
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
            tooltip_value(self.language, &content.value_text()),
            (text_x, text_y),
            text_style.clone(),
        ))
        .ok();
        text_y += TOOLTIP_LINE_HEIGHT as i32;

        area.draw(&Text::new(
            tooltip_time(self.language, &content.timestamp_text()),
            (text_x, text_y),
            text_style.clone(),
        ))
        .ok();
        text_y += TOOLTIP_LINE_HEIGHT as i32;

        if let Some(desc) = &content.description {
            area.draw(&Text::new(desc.clone(), (text_x, text_y), text_style)).ok();
        }

        // draw tooltip point to test position
        // area.draw(&Circle::new(
        //     (tooltip.point_x as i32, tooltip.point_y as i32),
        //     5,
        //     ShapeStyle::from(series_color).filled(),
        // ))
        // .ok();
    }

    fn hovered_point_at(&self, cursor: Point, bounds: Size, snap_distance: f32) -> Option<TooltipData> {
        let chart_bounds = Size::new(
            bounds.width - self.y_label_area_size - CHART_MARGIN_LEFT - CHART_MARGIN_RIGHT,
            bounds.height - X_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN,
        );

        if chart_bounds.width <= 0.0 || chart_bounds.height <= 0.0 {
            return None;
        }

        let chart_cursor = Point::new(
            cursor.x - self.y_label_area_size - CHART_MARGIN_LEFT,
            cursor.y - CHART_MARGIN,
        );

        let (oldest, newest) = self.time_bounds();
        let total_ms = (newest - oldest).num_milliseconds() as f32;
        let snap_sq = snap_distance * snap_distance;

        let mut best: Option<(TooltipData, f32)> = None;

        let mut update_best_tooltip = |tooltip: TooltipData, dist_sq: f32| {
            if best.as_ref().map(|b| dist_sq < b.1).unwrap_or(true) {
                best = Some((tooltip, dist_sq));
            }
        };

        let create_tooltip = |label: &str,
                              value: f32,
                              time: DateTime<Local>,
                              idx: usize,
                              px: f32,
                              py: f32,
                              color_index: Option<usize>| {
            let content = TooltipContent::new(
                label.to_string(),
                value,
                self.y_unit.to_string(),
                time,
                idx,
                color_index,
                self.x_range.num_seconds(),
            );
            TooltipData::new(content, px, py, chart_bounds.width, chart_bounds.height)
        };

        for (idx, (_key, s)) in self.data.iter().enumerate() {
            if s.newest_time().is_none() {
                continue;
            }

            let points = match s.points.try_borrow() {
                Ok(p) => p.iter().copied().collect::<Vec<_>>(),
                Err(_) => continue,
            };

            if points.is_empty() {
                continue;
            }

            match s.line_type {
                LineType::Step => {
                    for i in 0..points.len() {
                        let (time, value) = points[i];
                        let py = self.point_y_for_value(value, chart_bounds.height);

                        let y_dist = (py - chart_cursor.y).abs();
                        if y_dist > snap_distance {
                            continue;
                        }

                        let start_x = self.point_x_for_time(time, oldest, total_ms, chart_bounds.width);
                        let end_x = if i + 1 < points.len() {
                            self.point_x_for_time(points[i + 1].0, oldest, total_ms, chart_bounds.width)
                        } else {
                            chart_bounds.width
                        };

                        if chart_cursor.x >= start_x && chart_cursor.x <= end_x {
                            let cursor_time_ms = (chart_cursor.x / chart_bounds.width) * total_ms;
                            let cursor_time = oldest + Duration::milliseconds(cursor_time_ms as i64);

                            let tooltip = create_tooltip(
                                &s.display_label,
                                value,
                                cursor_time,
                                idx,
                                chart_cursor.x,
                                py,
                                s.color_index,
                            );
                            update_best_tooltip(tooltip, y_dist * y_dist);
                        }
                    }
                }
                _ => {
                    for (time, value) in points {
                        let px = self.point_x_for_time(time, oldest, total_ms, chart_bounds.width);
                        let py = self.point_y_for_value(value, chart_bounds.height);
                        let dist_sq = (px - chart_cursor.x).powi(2) + (py - chart_cursor.y).powi(2);

                        if dist_sq <= snap_sq {
                            let tooltip = create_tooltip(&s.display_label, value, time, idx, px, py, s.color_index);
                            update_best_tooltip(tooltip, dist_sq);
                        }
                    }
                }
            }
        }

        best.map(|(tooltip, _)| tooltip)
    }

    fn point_y_for_value(&self, value: f32, height: f32) -> f32 {
        let (min, max) = self.y_range;

        let range = max - min;
        if height <= 0.0 || range <= f32::EPSILON {
            return height / 2.0;
        }
        height * (1.0 - (value.clamp(min, max) - min) / range)
    }

    fn point_x_for_time(&self, time: DateTime<Local>, oldest: DateTime<Local>, total_ms: f32, width: f32) -> f32 {
        let ratio = ((time - oldest).num_milliseconds() as f32 / total_ms).clamp(0.0, 1.0);
        ratio * width
    }

    fn clear_hover(&self) -> bool {
        let mut current = match self.hovered.try_borrow_mut() {
            Ok(current) => current,
            Err(_) => return false,
        };
        if current.is_some() {
            *current = None;
            self.clear_cache();
            true
        } else {
            false
        }
    }

    fn update_hover(&self, new: Option<TooltipData>) -> bool {
        let mut current = match self.hovered.try_borrow_mut() {
            Ok(current) => current,
            Err(_) => return false,
        };
        if *current != new {
            *current = new;
            self.clear_cache();
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
        if let Ok(cache) = self.cache.try_borrow() {
            renderer.draw_cache(&cache, bounds, draw_fn)
        } else {
            renderer.draw_cache(&Cache::new(), bounds, draw_fn)
        }
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, chart: ChartBuilder<DB>) {
        self.build_chart_2d(chart);
    }
}
