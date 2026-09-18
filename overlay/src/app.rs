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

/// Padding inside each right-click menu segment, and the gap between segments.
const SEGMENT_PADDING: f32 = 6.0;
const MENU_GAP: f32 = 3.0;

const TOP_NAME_MAX: usize = 14;

const DECIMALS: &[u8] = &[0, 1, 2, 3];
const REFRESH: &[u32] = &[1, 2, 3, 5];
const TOP_APPS: &[usize] = &[1, 2, 3, 4, 5, 6, 8];

/// The overlay application state.
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

    /// Handles a message.
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
                self.top_apps = self.load_top_apps();
                // The dashboard owns the language setting, so re-reading it here
                // is what makes the overlay follow a change made over there.
                self.language = Language::from_database(self.database.as_ref());

                // Keep the window fitted to the live content, but only issue a
                // resize when the target actually changed.
                // The window is always exactly its content's size, so this only
                // has to react when the live values change the measured width.
                let target = self.fitted_size();
                if (target.width - self.applied.width).abs() >= 1.0
                    || (target.height - self.applied.height).abs() >= 1.0
                {
                    return self.resize_task();
                }
                Task::none()
            }
            Message::WindowId(id) => {
                self.window_id = id;
                // Refresh *before* sizing, so the window opens already fitted to
                // the content instead of a placeholder computed from empty data.
                self.power = self.load_power();
                self.top_apps = self.load_top_apps();
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
                self.resize_task()
            }
            Message::OpenMenu => {
                if self.show_settings {
                    return Task::none();
                }
                self.show_menu = true;
                self.resize_task()
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
                self.config.width = v.clamp(60.0, 600.0);
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
                self.config.save();
                iced::exit()
            }
        }
    }

    /// Renders the widget.
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
            .style(card_style(palette, self.card_alpha()));

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
            .map(|label| label.chars().count() as f32 * size * LABEL_CHAR_W + SEGMENT_PADDING * 2.0)
            .sum();
        let gaps = 3.0 * MENU_GAP;
        (((pad * 2.0 + segments + gaps) / 4.0).ceil() * 4.0).max(80.0)
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
                let mut row = Row::new().spacing(6).align_y(Alignment::Center);
                let mut first = true;
                let sep = || Text::new("·").size(label_size).color(palette.muted);
                for metric in &self.config.metrics {
                    if metric.is_multi() {
                        for (name, watts) in &self.top_apps {
                            if !first {
                                row = row.push(sep());
                            }
                            first = false;
                            row = row.push(
                                Text::new(truncate(name, TOP_NAME_MAX))
                                    .size(label_size)
                                    .color(palette.muted),
                            );
                            row = row.push(
                                Text::new(self.format_value(Some(*watts)))
                                    .size(value_size)
                                    .font(FONT_VALUE)
                                    .color(value_color),
                            );
                        }
                    } else {
                        if !first {
                            row = row.push(sep());
                        }
                        first = false;
                        if self.config.show_labels {
                            row = row.push(
                                Text::new(self.metric_label(*metric))
                                    .size(label_size)
                                    .color(palette.muted),
                            );
                        }
                        row = row.push(
                            Text::new(self.format_value(self.power.get(metric.id()).copied()))
                                .size(value_size)
                                .font(FONT_VALUE)
                                .color(value_color),
                        );
                    }
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
        // Only the vertical layout has a user-chosen width: the horizontal one
        // is measured from its own content and cannot be set.
        if self.config.layout == Layout::Vertical {
            window_col = window_col.push(
                Column::new()
                    .spacing(2)
                    .push(
                        Text::new(format!(
                            "{}  {} px",
                            translations::label_width(language),
                            self.config.width.round()
                        ))
                        .size(font)
                        .color(palette.muted),
                    )
                    .push(slider(60.0..=600.0, self.config.width, Message::SetWidth)),
            );
        }

        let mut metrics = Column::new().spacing(spacing);
        for &metric in Metric::ALL {
            let enabled = self.config.metrics.contains(&metric);
            metrics = metrics.push(
                checkbox(enabled)
                    .label(if metric.is_multi() {
                        translations::metric_top_apps_setting(language)
                    } else {
                        translations::metric_name(language, metric)
                    })
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

    fn load_top_apps(&mut self) -> Vec<(String, f64)> {
        let Some(db) = &mut self.database else {
            return Vec::new();
        };
        let window = self.config.refresh_secs.max(1) as i64;
        let Ok(rows) = db.select_top_processes_average(window, self.config.top_apps()) else {
            return Vec::new();
        };
        let Some((_ts, ComputedSensorData::Process(processes))) = rows.into_iter().next() else {
            return Vec::new();
        };
        processes
            .into_iter()
            .map(|p| {
                let watts = p.process_energy.as_watts_for_seconds(window as f64);
                (p.measured.app_name, watts)
            })
            .collect()
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

    /// Color the OS window is cleared with. In layered mode we clear with the
    /// card color so the whole (layered) window composites uniformly.
    fn window_background(&self) -> Color {
        if self.config.transparency.transparent_window() {
            Color::TRANSPARENT
        } else {
            theme::palette(self.config.theme).card
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

    /// Window width that fits the current fonts, density, metrics and values.
    ///
    /// Text advances are approximated (the renderer owns the real metrics) and
    /// the result is quantised to 4px so it stays stable as digits change.
    fn fitted_width(&self) -> f32 {
        let pad = self.config.density.padding();
        let spacing = self.config.density.spacing();
        let label_size = self.config.font_size.label();
        let value_size = self.config.font_size.value();

        let label_w = |text: &str| text.chars().count() as f32 * label_size * LABEL_CHAR_W;
        let value_w = |text: &str| text.chars().count() as f32 * value_size * VALUE_CHAR_W;

        let mut rows: Vec<f32> = Vec::new();
        for metric in &self.config.metrics {
            if metric.is_multi() {
                for (name, watts) in &self.top_apps {
                    let name = truncate(name, TOP_NAME_MAX);
                    rows.push(label_w(&name) + spacing + value_w(&self.format_value(Some(*watts))));
                }
            } else {
                let mut width = value_w(&self.format_value(self.power.get(metric.id()).copied()));
                if self.config.show_labels {
                    width += label_w(self.metric_label(*metric)) + spacing;
                }
                rows.push(width);
            }
        }

        let content = match self.config.layout {
            Layout::Horizontal => {
                let separator = label_w(" · ");
                let gaps = rows.len().saturating_sub(1) as f32;
                rows.iter().sum::<f32>() + separator * gaps
            }
            Layout::Vertical => rows.iter().fold(0.0_f32, |widest, row| widest.max(*row)),
        };

        // The grip is an overlay, so it must NOT be measured here — otherwise
        // the fitted width would be larger than the text and the bar could never
        // be dragged down to the minimum.
        let raw = pad * 2.0 + content;
        // Never below the window's own minimum, otherwise the OS clamps the
        // resize and the value we asked for and got would disagree.
        ((raw / 4.0).ceil() * 4.0).max(24.0)
    }

    /// Size the window should have right now.
    ///
    /// The height is always the measured content height, so the overlay never
    /// carries blank space. The width is measured exactly for the horizontal
    /// layout (where any slack is glaring) and user-chosen — but never narrower
    /// than the text — for the vertical one.
    fn fitted_size(&self) -> iced::Size {
        if self.show_settings {
            return iced::Size::new(SETTINGS_WIDTH, SETTINGS_HEIGHT);
        }
        // The menu replaces the metrics, so its own width is measured instead.
        if self.show_menu {
            return iced::Size::new(self.menu_width(), self.current_height());
        }
        let width = match self.config.layout {
            Layout::Horizontal => self.fitted_width(),
            Layout::Vertical => self.config.width.max(self.fitted_width()),
        };
        iced::Size::new(width, self.current_height())
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

fn card_style(palette: Palette, opacity: f32) -> impl Fn(&Theme) -> iced::widget::container::Style {
    move |_theme| iced::widget::container::Style {
        background: Some(Background::Color(palette.card_with_alpha(opacity))),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        text_color: Some(palette.text),
        shadow: Shadow {
            color: palette.shadow,
            offset: Vector::new(0.0, 3.0),
            blur_radius: 14.0,
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
