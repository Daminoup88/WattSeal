//! Messages handled by the overlay application.

use crate::{
    config::{Density, FontSize, Layout, Metric},
    theme::ThemeChoice,
};

/// All events handled by [`crate::app::OverlayApp`].
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    /// The overlay window id (from `window::latest`).
    WindowId(Option<iced::window::Id>),
    /// The raw OS window handle (used by the layered transparency mode).
    RawWindowId(u64),
    StartDrag,
    /// Window moved (persist position).
    Moved(f32, f32),
    ToggleSettings,
    OpenMenu,
    CloseMenu,
    TogglePin,
    TogglePinClickThrough(bool),

    // appearance
    ChangeOpacity(f32),
    SetBgColor(crate::config::BgColor),
    SetTextColor(crate::config::TextColor),
    SetTransparency(crate::config::Transparency),
    ToggleShadow(bool),
    SetLayout(Layout),
    SetDensity(Density),
    SetFontSize(FontSize),
    SetTheme(ThemeChoice),
    ToggleLabels(bool),
    ToggleUnits(bool),
    ToggleAbbreviated(bool),
    SetDecimals(u8),
    SetRefresh(u32),

    // window
    ToggleAlwaysOnTop(bool),
    /// Width (logical px) the widget may grow to.
    SetWidth(f32),

    // content
    ToggleMetric(Metric, bool),
    /// Moves a metric one place earlier or later in the bar.
    MoveMetricUp(Metric),
    MoveMetricDown(Metric),
    SetTopApps(usize),

    /// Quit the overlay.
    Quit,
    /// The OS asked to close the window.
    CloseRequested,
}
