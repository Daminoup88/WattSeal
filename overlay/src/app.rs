//! The overlay iced application: state, update loop and view.
//!
//! Fully self-contained — it reads the shared SQLite database produced by the
//! collector (like the main UI) but owns its own state, theme and window.

use std::collections::HashMap;

use common::{ComputedSensorData, Database};
use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Padding, Shadow, Subscription, Task, Theme, Vector,
    event,
    font::{Family, Weight},
    time::{Duration, every},
    widget::{Button, Column, Container, Row, Space, Text, button, checkbox, mouse_area, pick_list, slider},
    window,
};

use crate::{
    config::{BgColor, Density, FontSize, Layout, Metric, OverlayConfig, TextColor, Transparency},
    message::Message,
    theme::{self, Palette, ThemeChoice},
    translations::{self, Labeled, Language, Localize},
};

/// Monospace bold font for values (clean, aligned digits).
const FONT_VALUE: Font = Font {
    family: Family::Monospace,
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// The settings panel is laid out in two columns so it stays compact — and
/// therefore scrollbar-free — instead of growing taller than the screen.
const SETTINGS_WIDTH: f32 = 560.0;
const SETTINGS_HEIGHT: f32 = 452.0;

/// Approximate character advance as a fraction of the font size, used by the
/// content-fitted width. The real metrics live in the renderer, so these are
/// deliberately a little generous to avoid clipping.
const LABEL_CHAR_W: f32 = 0.52;
const VALUE_CHAR_W: f32 = 0.62;

/// The width ladder. A measured width is rounded up to a multiple of this, so
/// the widget grows in visible steps instead of inching along every time a
/// digit gets wider. The `Width` setting snaps to the same rungs, and a change
/// smaller than one rung does not move the window at all.
const WIDTH_STEP: f32 = 12.0;

/// Narrowest usable width, matching the OS window minimum so a request is never
/// clamped behind our back.
const MIN_WIDTH: f32 = 24.0;

/// Range the `Width` slider offers.
const MIN_CHOICE_WIDTH: f32 = 60.0;
const MAX_CHOICE_WIDTH: f32 = 1200.0;

/// Gap between a label and its value, and between two entries of the horizontal
/// bar. Matches the row spacing both are built with.
const BAR_INNER_GAP: f32 = 6.0;

/// Padding inside each right-click menu segment, and the gap between segments.
const SEGMENT_PADDING: f32 = 6.0;
const MENU_GAP: f32 = 3.0;

const TOP_NAME_MAX: usize = 14;

const DECIMALS: &[u8] = &[0, 1, 2, 3];
const REFRESH: &[u32] = &[1, 2, 3, 5];
const TOP_APPS: &[usize] = &[1, 2, 3, 4, 5, 6, 8];

/// Advance of one character, as a fraction of the font size.
///
/// CJK glyphs are full-width — close to twice a Latin advance — and both the
/// labels and the `Top apps` process names can contain them. Measuring every
/// character at the Latin factor would under-measure a Chinese label by nearly
/// half, which the horizontal layout and the menu have no slack to absorb.
fn char_advance(character: char, latin_factor: f32) -> f32 {
    let full_width = matches!(
        character as u32,
        0x1100..=0x115F
            | 0x2E80..=0x303E
            | 0x3041..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
    );

    if full_width { latin_factor * 1.9 } else { latin_factor }
}

/// Approximate rendered width of `text` at `size`.
///
/// The renderer owns the real metrics, so this stays an estimate — deliberately
/// on the generous side, since a clipped value is worse than a little slack.
fn text_width(text: &str, size: f32, latin_factor: f32) -> f32 {
    text.chars().map(|c| char_advance(c, latin_factor)).sum::<f32>() * size
}

/// Rounds a measured width up to the next rung of the ladder: a fitted width
/// must never come out narrower than the text it has to hold.
fn width_up(width: f32) -> f32 {
    ((width / WIDTH_STEP).ceil() * WIDTH_STEP).max(MIN_WIDTH)
}

/// Rounds a chosen width to the nearest rung, so the `Width` slider lands on the
/// same ladder the measured widths use.
fn width_near(width: f32) -> f32 {
    ((width / WIDTH_STEP).round() * WIDTH_STEP).max(WIDTH_STEP)
}

/// Swaps `metric` with the neighbour `delta` places away, and reports whether it
/// moved.
///
/// Pure, so the ordering rules are tested rather than clicked: a metric that is
/// switched off has no place in the order, and either end of the list is a wall.
fn move_metric(order: &mut [Metric], metric: Metric, delta: isize) -> bool {
    let Some(index) = order.iter().position(|m| *m == metric) else {
        return false;
    };

    let target = index as isize + delta;
    if target < 0 || target as usize >= order.len() {
        return false;
    }

    order.swap(index, target as usize);
    true
}

struct BarItem {
    /// `None` when labels are switched off, or for an entry that has none.
    label: Option<String>,
    value: String,
}

pub struct OverlayApp {
    config: OverlayConfig,
    window_id: Option<window::Id>,
    window_raw: Option<u64>,
    power: HashMap<String, f64>,
    top_apps: Vec<(String, f64)>,
    /// Last `overlay_requested` value seen in the shared config. Only a true to
    /// false transition closes the overlay.
    requested: bool,
    show_settings: bool,
    /// True while the bar's content is replaced by the right-click menu.
    show_menu: bool,
    /// Size last requested from the OS, so the auto-fit does not resize — and
    /// flicker — on every tick.
    applied: iced::Size,
    /// Language the dashboard is set to; the overlay follows it.
    language: Language,
    database: Option<Database>,
}

impl OverlayApp {
    /// Boots the app: loads config, opens the database, discovers the window id.
    pub fn new() -> (Self, Task<Message>) {
        let config = OverlayConfig::load().unwrap_or_default();
        let database = Database::open_without_migrations().ok();
        let requested = config.overlay_requested;
        let language = Language::from_database(database.as_ref());

        let app = Self {
            config,
            requested,
            window_id: None,
            window_raw: None,
            power: HashMap::new(),
            top_apps: Vec::new(),
            show_settings: false,
            show_menu: false,
            applied: iced::Size::ZERO,
            language,
            database,
        };

        let task = Task::batch([window::latest().map(Message::WindowId), Task::done(Message::Tick)]);
        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // Honour flags written by the other processes (tray / main
                // window). They only edit the shared config file, and this
                // returns `true` when the overlay has been asked to close.
                if self.sync_external_state() {
                    return iced::exit();
                }
                // Keep re-opening until the collector has created the sensor
                // tables (the overlay may start before the collector is ready).
                let needs_open = self
                    .database
                    .as_ref()
                    .map(|db| db.get_tables().is_empty())
                    .unwrap_or(true);
                if needs_open {
                    self.database = Database::open_without_migrations().ok();
                }
                self.power = self.load_power();
                self.refresh_top_apps();
                // The dashboard owns the language setting, so re-reading it here
                // is what makes the overlay follow a change made over there.
                self.language = Language::from_database(self.database.as_ref());

                // Keep the window fitted to the live content, but only resize
                // when the target moved by a whole rung. Reacting to a couple of
                // pixels is what made the widget jitter while the numbers moved.
                let target = self.fitted_size();
                let width_moved = (target.width - self.applied.width).abs() >= WIDTH_STEP;
                let height_moved = (target.height - self.applied.height).abs() >= 1.0;
                if width_moved || height_moved {
                    return self.resize_task();
                }
                Task::none()
            }
            Message::WindowId(id) => {
                self.window_id = id;
                // Refresh *before* sizing, so the window opens already fitted to
                // the content instead of a placeholder computed from empty data.
                self.power = self.load_power();
                self.refresh_top_apps();
                self.applied = self.fitted_size();
                let raw = id
                    .map(|id| window::raw_id::<Message>(id).map(Message::RawWindowId))
                    .unwrap_or_else(Task::none);
                Task::batch([self.apply_window_settings(), raw])
            }
            Message::RawWindowId(raw) => {
                self.window_raw = Some(raw);
                self.apply_layered();
                self.apply_click_through();
                Task::none()
            }
            Message::StartDrag => self.window_id.map(window::drag::<Message>).unwrap_or_else(Task::none),
            Message::Moved(x, y) => {
                self.config.position = Some((x, y));
                Task::none()
            }

            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                self.show_menu = false;
                self.resize_task().chain(self.keep_on_screen())
            }
            Message::OpenMenu => {
                if self.show_settings {
                    return Task::none();
                }
                self.show_menu = true;
                self.resize_task().chain(self.keep_on_screen())
            }
            Message::CloseMenu => {
                self.show_menu = false;
                self.resize_task()
            }
            Message::TogglePin => {
                self.config.pin_mode = !self.config.pin_mode;
                self.persist();
                self.apply_click_through();
                // Close the menu so the metrics come back — and because with
                // click-through on the menu is no longer reachable anyway.
                self.show_menu = false;
                self.resize_task()
            }
            Message::TogglePinClickThrough(v) => {
                self.config.pin_click_through = v;
                self.persist();
                self.apply_click_through();
                Task::none()
            }

            // appearance
            Message::SetBgColor(v) => {
                self.config.bg_color = v;
                self.persist();
                Task::none()
            }
            Message::SetTextColor(v) => {
                self.config.text_color = v;
                self.persist();
                Task::none()
            }
            Message::ChangeOpacity(v) => {
                self.config.opacity = v.clamp(0.05, 1.0);
                self.persist();
                self.apply_layered();
                Task::none()
            }
            Message::SetTransparency(v) => {
                self.config.transparency = v;
                self.persist();
                self.apply_layered();
                Task::none()
            }
            Message::ToggleShadow(v) => {
                self.config.shadow = v;
                self.persist();
                Task::none()
            }
            Message::SetLayout(v) => {
                self.config.layout = v;
                self.persist();
                self.resize_task()
            }
            Message::SetDensity(v) => {
                self.config.density = v;
                self.persist();
                self.resize_task()
            }
            Message::SetFontSize(v) => {
                self.config.font_size = v;
                self.persist();
                self.resize_task()
            }
            Message::ToggleAbbreviated(v) => {
                self.config.abbreviated = v;
                self.persist();
                self.resize_task()
            }
            Message::SetTheme(v) => {
                self.config.theme = v;
                self.persist();
                Task::none()
            }
            Message::ToggleLabels(v) => {
                self.config.show_labels = v;
                self.persist();
                Task::none()
            }
            Message::ToggleUnits(v) => {
                self.config.show_units = v;
                self.persist();
                Task::none()
            }
            Message::SetDecimals(v) => {
                self.config.decimals = v.min(3);
                self.persist();
                Task::none()
            }
            Message::SetRefresh(v) => {
                self.config.refresh_secs = v.max(1);
                self.persist();
                Task::none()
            }

            // window
            Message::ToggleAlwaysOnTop(v) => {
                self.config.always_on_top = v;
                self.persist();
                self.set_level(if v {
                    window::Level::AlwaysOnTop
                } else {
                    window::Level::Normal
                })
            }
            Message::SetWidth(v) => {
                // Snap to the ladder, so the number shown next to the slider is
                // the width the window actually takes, and keep it above what a
                // value needs — the slider offers the same floor.
                let floor = self.width_floor();
                self.config.width = width_near(v.clamp(floor, MAX_CHOICE_WIDTH));
                self.persist();
                self.resize_task()
            }
            // content
            Message::ToggleMetric(metric, enabled) => {
                if enabled {
                    if !self.config.metrics.contains(&metric) {
                        self.config.metrics.push(metric);
                    }
                } else {
                    self.config.metrics.retain(|m| *m != metric);
                }
                self.persist();
                self.resize_task()
            }
            Message::MoveMetricUp(metric) => {
                move_metric(&mut self.config.metrics, metric, -1);
                self.persist();
                self.resize_task()
            }
            Message::MoveMetricDown(metric) => {
                move_metric(&mut self.config.metrics, metric, 1);
                self.persist();
                self.resize_task()
            }
            Message::SetTopApps(k) => {
                self.config.top_apps = k.clamp(1, 8);
                self.persist();
                self.resize_task()
            }

            Message::Quit | Message::CloseRequested => {
                // Record that the overlay is no longer wanted, so the main
                // window's footer toggle follows along instead of staying stuck
                // on "Hide overlay" after an exit from the bar's own menu.
                self.config.overlay_requested = false;
                // Closing clears the pin too: that is what makes the dashboard's
                // hide-and-show a way out of a pinned, click-through widget on a
                // system with no tray.
                self.config.pin_mode = false;
                self.config.save();
                iced::exit()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        let palette = theme::palette_with(self.config.theme, self.config.bg_color, self.config.text_color);
        let pad = self.config.density.padding();
        let spacing = self.config.density.spacing();
        let label_size = self.config.font_size.label();
        let value_size = self.config.font_size.value();

        let body: Element<'_, Message, Theme> = if self.show_settings {
            self.view_settings(palette, label_size, spacing)
        } else if self.show_menu {
            self.view_menu(palette, label_size)
        } else {
            self.view_metrics(palette, label_size, value_size, spacing)
        };

        // The header row exists only in settings mode, where it doubles as the
        // drag handle. In metrics mode the whole card is the drag / right-click
        // surface instead — reclaiming the space the old gear/close buttons took.
        let column = if self.show_settings {
            Column::new()
                .spacing(spacing)
                .push(self.view_header(palette))
                .push(body)
        } else {
            // Pin the metrics to the bottom of the card: an oversized window
            // then leaves its slack *above* the text, so the floating grip
            // (anchored to the bottom-right) always lands on the last row.
            // No `spacing` here: the filler already carries the gap, and adding
            // one would make the content need more height than `fitted_height`
            // accounts for.
            Column::new().push(Space::new().height(Length::Fill)).push(body)
        };

        let card = Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(pad))
            // `draws_shadow` is the verdict already resolved for the active
            // transparency mode, not the raw setting: a surface with no per-pixel
            // alpha would only turn the soft edge into a dark ring.
            .style(card_style(palette, self.card_alpha(), self.config.draws_shadow()));

        // The card is the drag / right-click surface only while the metrics are
        // showing: the settings panel and the menu need clickable widgets. A
        // pinned overlay is never draggable — that is the part of pin mode every
        // platform can honour.
        if self.show_settings || self.show_menu || self.config.pin_mode {
            card.into()
        } else {
            mouse_area(card).on_press(Message::StartDrag).into()
        }
    }

    /// Labels of the in-bar menu, in order.
    fn menu_labels(&self) -> [&'static str; 4] {
        let language = self.language;
        [
            translations::menu_resume(language),
            translations::menu_settings(language),
            if self.config.pin_mode {
                translations::menu_unpin(language)
            } else {
                translations::menu_pin(language)
            },
            translations::menu_exit(language),
        ]
    }

    /// Width that fits the four menu segments.
    fn menu_width(&self) -> f32 {
        let pad = self.config.density.padding();
        let size = self.config.font_size.label();
        let segments: f32 = self
            .menu_labels()
            .iter()
            .map(|label| text_width(label, size, LABEL_CHAR_W) + SEGMENT_PADDING * 2.0)
            .sum();
        let gaps = 3.0 * MENU_GAP;
        width_up(pad * 2.0 + segments + gaps).max(80.0)
    }

    /// The right-click menu.
    ///
    /// The bar's own content is replaced by four segments instead of raising an
    /// OS popup, so it behaves identically on Windows, Linux and macOS and can
    /// never be clipped by the window's own size.
    fn view_menu(&self, palette: Palette, font: f32) -> Element<'_, Message, Theme> {
        let segment = |label: &'static str, message: Message| -> Element<'_, Message, Theme> {
            button(Text::new(label).size(font).color(palette.text))
                .style(flat_button(palette))
                .padding(Padding::from([1.0, SEGMENT_PADDING]))
                .on_press(message)
                .into()
        };
        let labels = self.menu_labels();

        Row::new()
            .spacing(MENU_GAP)
            .align_y(Alignment::Center)
            .push(segment(labels[0], Message::CloseMenu))
            .push(segment(labels[1], Message::ToggleSettings))
            .push(segment(labels[2], Message::TogglePin))
            .push(segment(labels[3], Message::Quit))
            .into()
    }

    /// A slim drag handle, shown only in settings mode. Metrics mode needs no
    /// header at all because there the whole card is the drag surface.
    fn view_header(&self, palette: Palette) -> Element<'_, Message, Theme> {
        let bar = Container::new(Space::new().width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fixed(8.0))
            .style(header_style(palette));
        mouse_area(bar).on_press(Message::StartDrag).into()
    }

    fn view_metrics(
        &self,
        palette: Palette,
        label_size: f32,
        value_size: f32,
        spacing: f32,
    ) -> Element<'_, Message, Theme> {
        // Values follow the configured text color (the separate "Value color"
        // setting was removed in favour of the text swatch).
        let value_color = palette.text;

        let body: Element<'_, Message, Theme> = match self.config.layout {
            Layout::Vertical => {
                let mut column = Column::new().spacing(spacing);
                if self.config.metrics.is_empty() {
                    column = column.push(Text::new("—").size(label_size).color(palette.muted));
                }
                for metric in &self.config.metrics {
                    if metric.is_multi() {
                        for (name, watts) in &self.top_apps {
                            column = column.push(self.value_row(
                                truncate(name, TOP_NAME_MAX),
                                self.format_value(Some(*watts)),
                                palette,
                                label_size,
                                value_size,
                                value_color,
                            ));
                        }
                    } else {
                        column = column.push(self.value_row(
                            self.metric_label(*metric).to_string(),
                            self.format_value(self.power.get(metric.id()).copied()),
                            palette,
                            label_size,
                            value_size,
                            value_color,
                        ));
                    }
                }
                column.into()
            }
            Layout::Horizontal => {
                let mut row = Row::new().spacing(BAR_INNER_GAP).align_y(Alignment::Center);

                for (index, item) in self.bar_items().iter().enumerate() {
                    if index > 0 {
                        row = row.push(Text::new("·").size(label_size).color(palette.muted));
                    }
                    if let Some(label) = &item.label {
                        row = row.push(Text::new(label.clone()).size(label_size).color(palette.muted));
                    }
                    row = row.push(
                        Text::new(item.value.clone())
                            .size(value_size)
                            .font(FONT_VALUE)
                            .color(value_color),
                    );
                }

                row.into()
            }
        };

        Container::new(body).width(Length::Fill).into()
    }

    fn value_row(
        &self,
        label: String,
        value: String,
        palette: Palette,
        label_size: f32,
        value_size: f32,
        value_color: Color,
    ) -> Element<'_, Message, Theme> {
        let mut row = Row::new().spacing(6).align_y(Alignment::Center);
        if self.config.show_labels {
            row = row.push(
                Text::new(label)
                    .size(label_size)
                    .color(palette.muted)
                    .width(Length::Fill),
            );
        } else {
            row = row.push(Space::new().width(Length::Fill));
        }
        row.push(Text::new(value).size(value_size).font(FONT_VALUE).color(value_color))
            .into()
    }

    fn view_settings(&self, palette: Palette, font: f32, spacing: f32) -> Element<'_, Message, Theme> {
        let language = self.language;
        let bg_dec = (self.config.opacity - 0.05).clamp(0.05, 1.0);
        let bg_inc = (self.config.opacity + 0.05).clamp(0.05, 1.0);

        let opacity_row = Column::new()
            .spacing(2)
            .push(stepper_row(
                translations::label_opacity(language),
                self.config.opacity,
                Message::ChangeOpacity(bg_dec),
                Message::ChangeOpacity(bg_inc),
                font,
                palette,
            ))
            .push(hint(self.opacity_hint(), font, palette));

        // A shadow needs per-pixel alpha to fade into. Where the mode has none the
        // toggle is kept — it is still the user's preference, and it takes effect
        // again in a mode that can render it — but a hint says why nothing changed.
        let mut shadow_row = Column::new().spacing(2).push(toggle(
            translations::label_shadow(language),
            self.config.shadow,
            Message::ToggleShadow,
            font,
            palette,
        ));
        if self.config.shadow && !self.config.draws_shadow() {
            shadow_row = shadow_row.push(hint(translations::hint_shadow_unavailable(language), font, palette));
        }

        let appearance = Column::new()
            .spacing(spacing)
            .push(section_title(translations::section_appearance(language), font, palette))
            .push(opacity_row)
            .push(picker(
                translations::label_bg_color(language),
                labeled_pick(BgColor::ALL, self.config.bg_color, language, Message::SetBgColor),
                font,
                palette,
            ))
            .push(picker(
                translations::label_text_color(language),
                labeled_pick(TextColor::ALL, self.config.text_color, language, Message::SetTextColor),
                font,
                palette,
            ))
            .push(picker(
                translations::label_transparency(language),
                labeled_pick(
                    Transparency::ALL,
                    self.config.transparency,
                    language,
                    Message::SetTransparency,
                ),
                font,
                palette,
            ))
            .push(shadow_row)
            .push(picker(
                translations::label_layout(language),
                labeled_pick(Layout::ALL, self.config.layout, language, Message::SetLayout),
                font,
                palette,
            ))
            .push(picker(
                translations::label_density(language),
                labeled_pick(Density::ALL, self.config.density, language, Message::SetDensity),
                font,
                palette,
            ))
            .push(picker(
                translations::label_text_size(language),
                labeled_pick(FontSize::ALL, self.config.font_size, language, Message::SetFontSize),
                font,
                palette,
            ))
            .push(picker(
                translations::label_theme(language),
                labeled_pick(ThemeChoice::ALL, self.config.theme, language, Message::SetTheme),
                font,
                palette,
            ))
            .push(picker(
                translations::label_decimals(language),
                pick_list(DECIMALS, Some(self.config.decimals), Message::SetDecimals),
                font,
                palette,
            ))
            .push(picker(
                translations::label_refresh(language),
                pick_list(REFRESH, Some(self.config.refresh_secs), Message::SetRefresh),
                font,
                palette,
            ))
            .push(
                Row::new()
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .push(
                        checkbox(self.config.show_labels)
                            .label(translations::label_show_labels(language))
                            .text_size(font)
                            .on_toggle(Message::ToggleLabels),
                    )
                    .push(
                        checkbox(self.config.show_units)
                            .label(translations::label_show_units(language))
                            .text_size(font)
                            .on_toggle(Message::ToggleUnits),
                    ),
            )
            .push(
                checkbox(self.config.abbreviated)
                    .label(translations::label_short_labels(language))
                    .text_size(font)
                    .on_toggle(Message::ToggleAbbreviated),
            );

        let mut window_col = Column::new()
            .spacing(spacing)
            .push(section_title(translations::section_window(language), font, palette))
            .push(toggle(
                translations::label_always_on_top(language),
                self.config.always_on_top,
                Message::ToggleAlwaysOnTop,
                font,
                palette,
            ))
            .push(if crate::winlayer::click_through_supported() {
                toggle(
                    translations::label_pin_click_through(language),
                    self.config.pin_click_through,
                    Message::TogglePinClickThrough,
                    font,
                    palette,
                )
            } else {
                hint(translations::hint_pin_unavailable(language), font, palette)
            });
        // The setting is the widest the widget may get: it hugs its numbers and
        // stops here, which is the only way a single line can be capped without
        // wrapping it or cutting it.
        let width_floor = self.width_floor();
        window_col = window_col.push(
            Column::new()
                .spacing(2)
                .push(
                    Text::new(format!(
                        "{}  {} px",
                        translations::label_width(language),
                        self.config.width.max(width_floor).round()
                    ))
                    .size(font)
                    .color(palette.muted),
                )
                .push(slider(
                    width_floor..=MAX_CHOICE_WIDTH,
                    self.config.width.max(width_floor),
                    Message::SetWidth,
                )),
        );

        // Display order first: the metrics that are on, in the order the bar shows
        // them, each with the arrows that move it past its neighbour. The ones that
        // are off follow, and a metric switched on is appended to the end.
        let name_of = |metric: Metric| {
            if metric.is_multi() {
                translations::metric_top_apps_setting(language)
            } else {
                translations::metric_name(language, metric)
            }
        };

        let mut metrics = Column::new().spacing(spacing);
        for &metric in &self.config.metrics {
            metrics = metrics.push(
                Row::new()
                    .spacing(BAR_INNER_GAP)
                    .align_y(Alignment::Center)
                    .push(
                        checkbox(true)
                            .label(name_of(metric))
                            .text_size(font)
                            .on_toggle(move |v| Message::ToggleMetric(metric, v)),
                    )
                    .push(step_button("▲", Message::MoveMetricUp(metric), palette, font))
                    .push(step_button("▼", Message::MoveMetricDown(metric), palette, font)),
            );
        }
        for metric in Metric::ALL.iter().copied().filter(|m| !self.config.metrics.contains(m)) {
            metrics = metrics.push(
                checkbox(false)
                    .label(name_of(metric))
                    .text_size(font)
                    .on_toggle(move |v| Message::ToggleMetric(metric, v)),
            );
        }
        let mut content_col = Column::new()
            .spacing(spacing)
            .push(section_title(translations::section_content(language), font, palette))
            .push(metrics);
        if self.config.metrics.contains(&Metric::TopApps) {
            content_col = content_col.push(picker(
                translations::label_top_count(language),
                pick_list(TOP_APPS, Some(self.config.top_apps()), Message::SetTopApps),
                font,
                palette,
            ));
        }

        let done: Button<'_, Message, Theme> = button(Text::new(translations::button_done(language)).size(font))
            .style(flat_button(palette))
            .on_press(Message::ToggleSettings);

        let quit: Button<'_, Message, Theme> =
            button(Text::new(translations::button_quit_overlay(language)).size(font))
                .style(flat_button(palette))
                .on_press(Message::Quit);

        // Two balanced columns: appearance on the left, window/content on the
        // right. This halves the height, so no scrollbar is needed.
        let right = Column::new()
            .spacing(spacing)
            .width(Length::Fill)
            .push(window_col)
            .push(content_col)
            .push(Row::new().spacing(8).align_y(Alignment::Center).push(done).push(quit));

        Row::new()
            .spacing(20)
            .padding(Padding::from([0.0, 16.0]))
            .push(appearance.width(Length::Fill))
            .push(right)
            .into()
    }

    /// Tick + tray polling + window events.
    pub fn subscription(&self) -> Subscription<Message> {
        // Poll quickly until the first sample arrives, so the window settles at
        // its fitted size immediately after opening instead of a second later.
        let interval = if self.power.is_empty() {
            Duration::from_millis(200)
        } else {
            Duration::from_secs(self.config.refresh_secs.max(1) as u64)
        };
        Subscription::batch([
            every(interval).map(|_| Message::Tick),
            event::listen_with(|evt, _status, _id| match evt {
                iced::Event::Window(window::Event::CloseRequested) => Some(Message::CloseRequested),
                iced::Event::Window(window::Event::Moved(point)) => Some(Message::Moved(point.x, point.y)),
                // Handled globally rather than through the card's `mouse_area`,
                // so it works regardless of widget hit-testing.
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)) => {
                    Some(Message::OpenMenu)
                }
                _ => None,
            }),
        ])
    }

    /// Window title (task switcher only).
    pub fn title(&self) -> String {
        String::from("WattSeal Overlay")
    }

    /// Active iced theme.
    pub fn theme(&self) -> Theme {
        theme::iced_theme(self.config.theme)
    }

    // ---- helpers ----

    /// Explains how the two opacity settings interact in the active mode.
    fn opacity_hint(&self) -> &'static str {
        let language = self.language;
        let mode = self.config.transparency;
        if mode.uses_layered() {
            translations::hint_opacity_layered(language)
        } else if mode.transparent_window() {
            translations::hint_opacity_surface(language)
        } else {
            translations::hint_opacity_off(language)
        }
    }

    /// Label shown for a metric, honouring the short-label mode.
    fn metric_label(&self, metric: Metric) -> &'static str {
        if self.config.abbreviated {
            translations::metric_short_name(self.language, metric)
        } else {
            translations::metric_name(self.language, metric)
        }
    }

    fn format_value(&self, watts: Option<f64>) -> String {
        match watts {
            Some(w) => {
                let number = format!("{w:.prec$}", prec = self.config.decimals as usize);
                if self.config.show_units {
                    format!("{number}W")
                } else {
                    number
                }
            }
            None => String::from("—"),
        }
    }

    fn load_power(&mut self) -> HashMap<String, f64> {
        let Some(db) = &mut self.database else {
            return HashMap::new();
        };
        let Ok(records) = db.select_last_n_records(1) else {
            return HashMap::new();
        };
        let mut map = HashMap::new();
        for (_ts, duration_ms, data) in records {
            let metric = match &data {
                ComputedSensorData::Total(_) => Metric::Total,
                ComputedSensorData::CPU(_) => Metric::Cpu,
                ComputedSensorData::GPU(_) => Metric::Gpu,
                ComputedSensorData::Ram(_) => Metric::Ram,
                ComputedSensorData::Disk(_) => Metric::Disk,
                ComputedSensorData::Network(_) => Metric::Network,
                _ => continue,
            };
            if let Some(energy) = data.total_energy() {
                let secs = if duration_ms > 0 {
                    duration_ms as f64 / 1000.0
                } else {
                    1.0
                };
                map.insert(metric.id().to_string(), energy.as_watts_for_seconds(secs));
            }
        }
        map
    }

    /// The per-app rows for this tick, or `None` when there is nothing to show.
    ///
    /// The caller keeps the previous list on `None`. A tick that lands between
    /// two collector samples comes back with no processes at all, and assigning
    /// that would blink the rows out for a second — and, in the horizontal
    /// layout, would briefly change how many lines the bar needs.
    fn load_top_apps(&mut self) -> Option<Vec<(String, f64)>> {
        let db = self.database.as_mut()?;
        let window = self.config.refresh_secs.max(1) as i64;
        let rows = db.select_top_processes_average(window, self.config.top_apps()).ok()?;
        let (_ts, ComputedSensorData::Process(processes)) = rows.into_iter().next()? else {
            return None;
        };

        let apps: Vec<(String, f64)> = processes
            .into_iter()
            .map(|p| {
                let watts = p.process_energy.as_watts_for_seconds(window as f64);
                (p.measured.app_name, watts)
            })
            .collect();

        (!apps.is_empty()).then_some(apps)
    }

    /// Refreshes the per-app rows, keeping the previous ones when this tick has
    /// no process sample to offer.
    fn refresh_top_apps(&mut self) {
        if let Some(apps) = self.load_top_apps() {
            self.top_apps = apps;
        }
    }

    fn persist(&self) {
        self.config.save();
    }

    /// Card background alpha: the opacity in per-pixel (surface) mode; fully
    /// opaque in layered/off mode (the window itself carries the alpha there).
    fn card_alpha(&self) -> f32 {
        if self.config.transparency.transparent_window() {
            self.config.opacity
        } else {
            1.0
        }
    }

    /// Color the OS window is cleared with.
    ///
    /// Where the surface has no per-pixel alpha, whatever the card does not paint
    /// shows this color. That is the corner wedges a rounded card leaves, and it
    /// has to be the card's *own* color, overrides included: clearing with the
    /// theme's stock card painted a dark frame around a card set to another
    /// swatch.
    fn window_background(&self) -> Color {
        if self.config.transparency.transparent_window() {
            Color::TRANSPARENT
        } else {
            theme::palette_with(self.config.theme, self.config.bg_color, self.config.text_color).card
        }
    }

    /// Applies the click-through extended styles for "pin mode".
    fn apply_click_through(&self) {
        if let Some(hwnd) = self.window_raw {
            let through =
                self.config.pin_mode && self.config.pin_click_through && crate::winlayer::click_through_supported();
            crate::winlayer::set_click_through(hwnd, through);
        }
    }

    /// Mirrors the externally controlled flags from the shared config file and
    /// reports whether the main window has asked the overlay to close.
    ///
    /// There is no IPC: the tray and the main window only edit
    /// `overlay_config.json`, and this polls it once per tick.
    fn sync_external_state(&mut self) -> bool {
        let Ok(text) = std::fs::read_to_string(OverlayConfig::path()) else {
            return false;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            return false;
        };

        if let Some(pinned) = value.get("pin_mode").and_then(|flag| flag.as_bool())
            && pinned != self.config.pin_mode
        {
            self.config.pin_mode = pinned;
            self.apply_click_through();
        }

        // Only a `true -> false` transition closes the overlay, so a stale
        // `false` left behind by a previous session cannot kill a fresh run.
        let requested = value
            .get("overlay_requested")
            .and_then(|flag| flag.as_bool())
            .unwrap_or(true);
        let closing = self.requested && !requested;
        self.requested = requested;
        closing
    }

    /// Applies the Win32 layered-window alpha when that mode is selected.
    fn apply_layered(&self) {
        if let (true, Some(hwnd)) = (self.config.transparency.uses_layered(), self.window_raw) {
            crate::winlayer::apply(hwnd, self.config.opacity);
        }
    }

    fn current_height(&self) -> f32 {
        self.config.fitted_height()
    }

    /// Width the content needs, before the `Width` setting is applied.
    ///
    /// The vertical layout is as wide as its widest row and the horizontal one as
    /// wide as its single line, so the widget hugs its numbers in both cases.
    fn content_width(&self) -> f32 {
        let pad = self.config.density.padding();
        let label_size = self.config.font_size.label();
        let value_size = self.config.font_size.value();

        let widths: Vec<f32> = self
            .bar_items()
            .iter()
            .map(|item| {
                let value = text_width(&item.value, value_size, VALUE_CHAR_W);
                match &item.label {
                    Some(label) => text_width(label, label_size, LABEL_CHAR_W) + BAR_INNER_GAP + value,
                    None => value,
                }
            })
            .collect();

        let content = match self.config.layout {
            Layout::Horizontal => {
                // Between two entries: the row spacing, the `·`, and the spacing again.
                let separator = text_width("·", label_size, LABEL_CHAR_W) + BAR_INNER_GAP * 2.0;
                let gaps = widths.len().saturating_sub(1) as f32;
                widths.iter().sum::<f32>() + separator * gaps
            }
            Layout::Vertical => widths.iter().fold(0.0_f32, |widest, width| widest.max(*width)),
        };

        // The grip is an overlay, so it must NOT be measured here — otherwise the
        // bar would reserve room it does not have.
        width_up(pad * 2.0 + content)
    }

    /// The width the window should take: what the content needs, capped by the
    /// `Width` setting.
    ///
    /// The setting is a maximum rather than an exact size on purpose. Neither
    /// layout can be narrowed below its content without either wrapping it onto
    /// more lines or cutting it, and both are worse than simply being as wide as
    /// the numbers are — a window padded out to the setting would sit there
    /// mostly empty.
    fn target_width(&self) -> f32 {
        self.content_width().min(self.width_cap())
    }

    /// The `Width` setting, never below what a single value needs.
    fn width_cap(&self) -> f32 {
        self.config.width.max(self.width_floor())
    }

    /// Lowest width the `Width` slider offers, snapped to the ladder so the
    /// number shown next to it is a width the window can actually take.
    fn width_floor(&self) -> f32 {
        width_near(self.values_floor().max(MIN_CHOICE_WIDTH))
    }

    /// The entries of the bar, in the order they are shown.
    ///
    /// Both layouts are built from these; only their arrangement differs.
    fn bar_items(&self) -> Vec<BarItem> {
        let mut items = Vec::new();

        for metric in &self.config.metrics {
            if metric.is_multi() {
                for (name, watts) in &self.top_apps {
                    items.push(BarItem {
                        label: Some(truncate(name, TOP_NAME_MAX)),
                        value: self.format_value(Some(*watts)),
                    });
                }
            } else {
                items.push(BarItem {
                    label: self.config.show_labels.then(|| self.metric_label(*metric).to_string()),
                    value: self.format_value(self.power.get(metric.id()).copied()),
                });
            }
        }

        items
    }

    /// Size the window should have right now.
    ///
    /// The width is what the content needs, capped by the `Width` setting, and
    /// the height follows the content as it always did, so the widget hugs its
    /// numbers in both layouts.
    fn fitted_size(&self) -> iced::Size {
        if self.show_settings {
            return iced::Size::new(SETTINGS_WIDTH, SETTINGS_HEIGHT);
        }
        // The menu replaces the metrics, so its own width is measured instead.
        if self.show_menu {
            return iced::Size::new(self.menu_width(), self.current_height());
        }

        iced::Size::new(self.target_width(), self.config.fitted_height())
    }

    /// Narrowest a value can be and still be read.
    ///
    /// The `Width` slider stops here: a lower cap would cut the numbers
    /// themselves, and an unreadable reading is worse than a window wider than
    /// the one that was asked for.
    fn values_floor(&self) -> f32 {
        let pad = self.config.density.padding();
        let value_size = self.config.font_size.value();
        let value_w = |text: String| text_width(&text, value_size, VALUE_CHAR_W);

        let widest = self.config.metrics.iter().fold(0.0_f32, |widest, metric| {
            let value = if metric.is_multi() {
                self.top_apps.iter().fold(0.0_f32, |widest, (_, watts)| {
                    widest.max(value_w(self.format_value(Some(*watts))))
                })
            } else {
                value_w(self.format_value(self.power.get(metric.id()).copied()))
            };
            widest.max(value)
        });

        width_up(pad * 2.0 + widest)
    }

    fn resize_task(&mut self) -> Task<Message> {
        let size = self.fitted_size();
        self.applied = size;
        match self.window_id {
            Some(id) => window::resize::<Message>(id, size),
            None => Task::none(),
        }
    }

    fn set_level(&self, level: window::Level) -> Task<Message> {
        match self.window_id {
            Some(id) => window::set_level::<Message>(id, level),
            None => Task::none(),
        }
    }

    fn apply_window_settings(&self) -> Task<Message> {
        let Some(id) = self.window_id else {
            return Task::none();
        };
        let level = if self.config.always_on_top {
            window::Level::AlwaysOnTop
        } else {
            window::Level::Normal
        };
        let size = self.fitted_size();
        let mut tasks: Vec<Task<Message>> = vec![
            window::set_level::<Message>(id, level),
            window::resize::<Message>(id, size),
            // The window always hugs its content, so the OS must not offer a
            // manual resize border.
            window::set_resizable::<Message>(id, false),
        ];
        if self.config.position.is_none() {
            tasks.push(window::monitor_size(id).and_then(move |monitor| {
                let point = anchor_point(monitor, size.width, size.height);
                window::move_to::<Message>(id, point)
            }));
        }
        Task::batch(tasks)
    }

    /// Nudges the window back inside the monitor after it grew.
    ///
    /// A window grows from its top-left corner, so one docked to the right edge
    /// used to push the settings panel or the menu off the screen.
    fn keep_on_screen(&self) -> Task<Message> {
        let (Some(id), Some((x, y))) = (self.window_id, self.config.position) else {
            return Task::none();
        };

        let size = self.fitted_size();
        window::monitor_size(id).and_then(move |monitor| {
            let target = clamp_point(monitor, size, iced::Point::new(x, y));

            if target == iced::Point::new(x, y) {
                Task::none()
            } else {
                window::move_to::<Message>(id, target)
            }
        })
    }

    fn reset_position(&self) -> Task<Message> {
        let Some(id) = self.window_id else {
            return Task::none();
        };
        let size = self.fitted_size();
        window::monitor_size(id).and_then(move |monitor| {
            let point = anchor_point(monitor, size.width, size.height);
            window::move_to::<Message>(id, point)
        })
    }

    /// Re-centers and re-fits the widget (used by the settings panel).
    pub fn reset(&mut self) -> Task<Message> {
        self.config.position = None;
        self.persist();
        self.resize_task().chain(self.reset_position())
    }
}

/// Snaps to the top-right corner of the monitor, keeping a small margin.
fn anchor_point(monitor: iced::Size, width: f32, _height: f32) -> iced::Point {
    const MARGIN: f32 = 16.0;
    iced::Point::new((monitor.width - width - MARGIN).max(MARGIN), MARGIN)
}

/// Pulls a window back so that one of `size` at `point` still fits the monitor,
/// keeping the same margin the anchor uses.
fn clamp_point(monitor: iced::Size, size: iced::Size, point: iced::Point) -> iced::Point {
    const MARGIN: f32 = 16.0;
    let right = (monitor.width - size.width - MARGIN).max(MARGIN);
    let bottom = (monitor.height - size.height - MARGIN).max(MARGIN);

    iced::Point::new(point.x.clamp(MARGIN, right), point.y.clamp(MARGIN, bottom))
}

fn truncate(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        return name.to_string();
    }
    let mut out: String = name.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

// ---- style helpers ----

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// The card's container style.
///
/// `shadow` is the resolved verdict from [`OverlayConfig::draws_shadow`], not the
/// raw setting: the caller knows the transparency mode, and a shadow drawn where
/// the window cannot blend one is worse than no shadow at all.
fn card_style(palette: Palette, opacity: f32, shadow: bool) -> impl Fn(&Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        background: Some(Background::Color(palette.card_with_alpha(opacity))),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        text_color: Some(palette.text),
        shadow: if shadow {
            Shadow {
                color: palette.shadow,
                offset: Vector::new(0.0, 3.0),
                blur_radius: 14.0,
            }
        } else {
            Shadow::default()
        },
        ..Default::default()
    }
}

fn header_style(palette: Palette) -> impl Fn(&Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        background: Some(Background::Color(with_alpha(palette.text, 0.06))),
        border: Border {
            color: with_alpha(palette.text, 0.10),
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(palette.text),
        ..Default::default()
    }
}

fn flat_button(palette: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| button::Style {
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                Some(Background::Color(with_alpha(palette.text, 0.12)))
            }
            _ => None,
        },
        text_color: palette.muted,
        border: Border::default(),
        shadow: Shadow::default(),
        ..Default::default()
    }
}

fn label<'a>(text: &'a str, font: f32, palette: Palette) -> Element<'a, Message, Theme> {
    Text::new(text)
        .size(font)
        .color(palette.muted)
        .width(Length::Fill)
        .into()
}

fn section_title<'a>(text: &'a str, font: f32, palette: Palette) -> Element<'a, Message, Theme> {
    Text::new(text)
        .size(font)
        .font(Font {
            weight: Weight::Bold,
            ..Font::DEFAULT
        })
        .color(palette.text)
        .into()
}

/// A pick-list whose options and selected value are rendered in `language`.
///
/// `pick_list` displays items through `Display`, so the translation has to
/// travel with each value — the same problem the dashboard solves with its
/// `TranslatedMetricType` wrapper.
fn labeled_pick<'a, T>(
    options: &'a [T],
    selected: T,
    language: Language,
    on_select: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message, Theme>
where
    T: Localize + Clone + PartialEq + 'a,
{
    let choices: Vec<Labeled<T>> = options
        .iter()
        .cloned()
        .map(|value| Labeled::new(value, language))
        .collect();

    pick_list(
        choices,
        Some(Labeled::new(selected, language)),
        move |option: Labeled<T>| on_select(option.value),
    )
    .into()
}

fn picker<'a>(
    title: &'a str,
    list: impl Into<Element<'a, Message, Theme>>,
    font: f32,
    palette: Palette,
) -> Element<'a, Message, Theme> {
    Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(label(title, font, palette))
        .push(
            Container::new(list.into())
                .width(Length::Fixed(108.0))
                .align_x(Alignment::End),
        )
        .into()
}

fn toggle(
    title: &'static str,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
    font: f32,
    _palette: Palette,
) -> Element<'static, Message, Theme> {
    checkbox(checked)
        .label(title)
        .text_size(font)
        .on_toggle(on_toggle)
        .into()
}

fn step_button<'a>(text: &'a str, message: Message, palette: Palette, font: f32) -> Element<'a, Message, Theme> {
    button(Text::new(text).size(font).font(FONT_VALUE))
        .style(flat_button(palette))
        .on_press(message)
        .padding(Padding::from([1, 8]))
        .into()
}

/// A `label  −  80%  +` row, used for the two opacity settings.
fn stepper_row<'a>(
    title: &'a str,
    ratio: f32,
    decrease: Message,
    increase: Message,
    font: f32,
    palette: Palette,
) -> Element<'a, Message, Theme> {
    Row::new()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(label(title, font, palette))
        .push(step_button("−", decrease, palette, font))
        .push(
            Text::new(format!("{:.0}%", ratio * 100.0))
                .size(font)
                .font(FONT_VALUE)
                .color(palette.text)
                .width(Length::Fixed(34.0)),
        )
        .push(step_button("+", increase, palette, font))
        .into()
}

/// A small muted explanatory line shown under a setting.
fn hint<'a>(text: &'a str, font: f32, palette: Palette) -> Element<'a, Message, Theme> {
    Text::new(text).size(font * 0.85).color(palette.muted).into()
}

/// Boots the standalone overlay window.
pub fn run() -> iced::Result {
    let config = OverlayConfig::load().unwrap_or_default();
    let level = if config.always_on_top {
        window::Level::AlwaysOnTop
    } else {
        window::Level::Normal
    };
    let pos = match config.position {
        Some((x, y)) => window::Position::Specific(iced::Point::new(x, y)),
        None => window::Position::Centered,
    };

    iced::application(OverlayApp::new, OverlayApp::update, OverlayApp::view)
        .title(OverlayApp::title)
        .settings(iced::Settings {
            id: Some(String::from("wattseal-overlay")),
            fonts: Vec::new(),
            default_font: Font {
                family: Family::SansSerif,
                weight: Weight::Medium,
                ..Font::DEFAULT
            },
            default_text_size: 12.0.into(),
            antialiasing: true,
            vsync: true,
        })
        .window(window::Settings {
            icon: window::icon::from_file_data(common::WINDOW_ICON_BYTES, Some(common::WINDOW_ICON_TYPE)).ok(),
            size: iced::Size::new(config.width, config.fitted_height()),
            position: pos,
            // The overlay is always sized to its content; no manual resizing.
            resizable: false,
            decorations: false,
            transparent: config.transparency.transparent_window(),
            blur: config.blur,
            level,
            platform_specific: platform_specific(),
            exit_on_close_request: false,
            ..Default::default()
        })
        .style(|state: &OverlayApp, _theme: &Theme| iced::theme::Style {
            background_color: state.window_background(),
            text_color: Color::WHITE,
        })
        .subscription(OverlayApp::subscription)
        .theme(OverlayApp::theme)
        .exit_on_close_request(false)
        .run()
}

/// Windows-only tweaks: hide from the taskbar and round the corners.
fn platform_specific() -> window::settings::PlatformSpecific {
    #[cfg(target_os = "windows")]
    {
        use window::settings::platform::CornerPreference;
        window::settings::PlatformSpecific {
            skip_taskbar: true,
            drag_and_drop: false,
            undecorated_shadow: false,
            corner_preference: CornerPreference::Round,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        window::settings::PlatformSpecific::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_width_glyphs_measure_wider_than_latin_ones() {
        // Measuring CJK at the Latin advance is what clipped the Chinese
        // labels, so the ratio itself is what is worth pinning down.
        let ratio = char_advance('总', 1.0) / char_advance('A', 1.0);
        assert!(ratio >= 1.5, "a full-width glyph measured at only {ratio}x a Latin one");
    }

    #[test]
    fn metrics_move_one_place_and_stop_at_the_ends() {
        let mut order = vec![Metric::Total, Metric::Cpu, Metric::Gpu];

        assert!(move_metric(&mut order, Metric::Cpu, -1));
        assert_eq!(order, vec![Metric::Cpu, Metric::Total, Metric::Gpu]);

        // Neither end of the list gives way, and a metric that is switched off has
        // no place to move from.
        assert!(!move_metric(&mut order, Metric::Cpu, -1));
        assert!(!move_metric(&mut order, Metric::Gpu, 1));
        assert!(!move_metric(&mut order, Metric::Ram, 1));
        assert_eq!(order, vec![Metric::Cpu, Metric::Total, Metric::Gpu]);
    }

    #[test]
    fn a_row_fits_the_font_it_is_drawn_with() {
        // Ultra is the tightest density, so it is the one that clipped: the value
        // font's line box is taller than the row the density used to reserve.
        for density in [Density::Ultra, Density::Compact, Density::Normal] {
            for font in [FontSize::Small, FontSize::Medium, FontSize::Large] {
                let row = density.row_height(font);
                assert!(
                    row >= font.value() * 1.2,
                    "{density:?} at {font:?} reserves {row}px for a {}px font",
                    font.value()
                );
            }
        }

        // The density still shows through where the font leaves room for it.
        assert!(Density::Normal.row_height(FontSize::Small) > Density::Ultra.row_height(FontSize::Small));
    }

    #[test]
    fn a_label_mixing_scripts_adds_up() {
        let latin = text_width("AB", 10.0, 0.5);
        let chinese = text_width("总", 10.0, 0.5);

        assert_eq!(text_width("AB总", 10.0, 0.5), latin + chinese);
        assert_eq!(latin, 10.0);
    }

    #[test]
    fn long_process_names_are_truncated_to_the_limit() {
        assert_eq!(truncate("firefox", TOP_NAME_MAX), "firefox");
        assert_eq!(truncate("a-very-long-process-name", 8), "a-very-…");
        assert_eq!(truncate("a-very-long-process-name", 8).chars().count(), 8);
    }

    #[test]
    fn the_widget_is_anchored_inside_the_monitor() {
        assert_eq!(
            anchor_point(iced::Size::new(1920.0, 1080.0), 200.0, 100.0),
            iced::Point::new(1704.0, 16.0)
        );

        // A widget wider than the monitor still lands inside it.
        let point = anchor_point(iced::Size::new(100.0, 100.0), 200.0, 50.0);
        assert_eq!(point.x, 16.0);
    }

    #[test]
    fn a_window_that_grew_is_nudged_back_inside_the_monitor() {
        let monitor = iced::Size::new(1920.0, 1080.0);
        let docked = iced::Point::new(1704.0, 16.0);

        // The metrics bar keeps its place while it fits where it is.
        assert_eq!(clamp_point(monitor, iced::Size::new(200.0, 40.0), docked), docked);

        // Opening the settings panel there would stick out, so it comes back.
        let nudged = clamp_point(monitor, iced::Size::new(560.0, 452.0), docked);
        assert_eq!(nudged.x, 1920.0 - 560.0 - 16.0);
        assert_eq!(nudged.y, 16.0);
    }

    #[test]
    fn a_measured_width_rounds_up_to_a_rung_of_the_ladder() {
        assert_eq!(width_up(1.0), MIN_WIDTH);
        assert_eq!(width_up(100.0), 108.0);
        // Already on a rung: no extra step is added on top.
        assert_eq!(width_up(108.0), 108.0);
    }

    #[test]
    fn a_chosen_width_snaps_to_the_nearest_rung() {
        assert_eq!(width_near(100.0), 96.0);
        assert_eq!(width_near(110.0), 108.0);
        assert_eq!(width_near(140.0), 144.0);
    }
}
