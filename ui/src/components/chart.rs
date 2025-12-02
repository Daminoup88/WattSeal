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
const X_LABEL_AREA_SIZE: f32 = 50.0;
const Y_LABEL_AREA_SIZE: f32 = 80.0;
const CHART_MARGIN: f32 = 20.0;
const CHART_MARGIN_LEFT: f32 = 40.0;

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
    pub value: String,
    pub timestamp: String,
    pub description: Option<String>,
    pub series_index: usize,
}

impl TooltipContent {
    pub fn new(title: String, value: f32, time: DateTime<Utc>, series_index: usize) -> Self {
        Self {
            title: title,
            value: format!("{:.1}%", value),
            timestamp: time.format("%H:%M:%S").to_string(),
            description: None,
            series_index,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn calculate_height(&self) -> f32 {
        let mut height = TOOLTIP_PADDING * 2.0;
        height += TOOLTIP_LINE_HEIGHT;
        height += TOOLTIP_LINE_HEIGHT;
        height += TOOLTIP_LINE_HEIGHT;
        if self.description.is_some() {
            height += TOOLTIP_LINE_HEIGHT;
        }
        height.max(TOOLTIP_MIN_HEIGHT)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipData {
    pub content: TooltipContent,
    pub time: DateTime<Utc>,
    pub value: f32,
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
    pub fn new(
        content: TooltipContent,
        time: DateTime<Utc>,
        value: f32,
        point_x: f32,
        point_y: f32,
        chart_width: f32,
        chart_height: f32,
    ) -> Self {
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
            time,
            value,
            point_x,
            point_y,
            side,
            bounds,
        }
    }
}

pub struct SensorChart<const N: usize> {
    cache: RefCell<Cache>,
    data_series: Vec<TimeSeries>,
    limit: Duration,
    hovered: RefCell<Option<TooltipData>>,
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
            .margin_left(CHART_MARGIN_LEFT)
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
            .y_desc("Value (%)")
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

        if let Some(tooltip) = self.hovered.borrow().as_ref() {
            let series_color = style.series_color(tooltip.content.series_index);
            let point = (tooltip.time, tooltip.value);

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
                .expect("hover marker");

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
            format!("Value: {}", content.value),
            (text_x, text_y),
            text_style.clone(),
        ))
        .ok();
        text_y += TOOLTIP_LINE_HEIGHT as i32;

        area.draw(&Text::new(
            format!("Time: {}", content.timestamp),
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
            bounds.width - Y_LABEL_AREA_SIZE - 2.0 * CHART_MARGIN - CHART_MARGIN_LEFT,
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
        let total_ms = self.limit.as_millis().max(1) as f32;
        let snap_sq = snap_distance * snap_distance;

        self.data_series
            .iter()
            .enumerate()
            .filter_map(|(idx, s)| s.newest_time().map(|_| (idx, s)))
            .flat_map(|(idx, s)| s.data.iter().map(move |d| (idx, s.label.clone(), d)))
            .filter_map(|(series_idx, label, (time, value))| {
                let px = self.point_x_for_time(*time, oldest, total_ms, chart_bounds.width);
                let py = self.point_y_for_value(*value, chart_bounds.height);
                let dist_sq = (px - chart_cursor.x).powi(2) + (py - chart_cursor.y).powi(2);

                if dist_sq <= snap_sq {
                    let content = TooltipContent::new(label, *value, *time, series_idx);
                    let tooltip =
                        TooltipData::new(content, *time, *value, px, py, chart_bounds.width, chart_bounds.height);
                    Some((tooltip, dist_sq))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(tooltip, _)| tooltip)
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
