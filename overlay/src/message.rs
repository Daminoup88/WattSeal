//! Messages handled by the overlay application.

use crate::config::{Density, FontSize, Layout, Metric};
use crate::theme::ThemeChoice;

/// All events handled by [`crate::app::OverlayApp`].
#[derive(Debug, Clone)]
pub enum Message {
    /// Periodic data refresh.
    Tick,
    /// The overlay window id (from `window::latest`).
    WindowId(Option<iced::window::Id>),
    /// The raw OS window handle (used by the layered transparency mode).
    RawWindowId(u64),
    /// Start dragging the widget.
    StartDrag,
    /// Window moved (persist position).
    Moved(f32, f32),
    /// Toggle the settings panel.
    ToggleSettings,
    /// Show the native right-click context menu.
    OpenMenu,

    // appearance
    ChangeOpacity(f32),
    SetBgColor(crate::config::BgColor),
    SetTextColor(crate::config::TextColor),
    SetTransparency(crate::config::Transparency),
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
    /// Width (logical px) used by the vertical layout.
    SetWidth(f32),

    // content
    ToggleMetric(Metric, bool),
    SetTopApps(usize),

    /// Quit the overlay.
    Quit,
    /// The OS asked to close the window.
    CloseRequested,
}
