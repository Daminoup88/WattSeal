//! Persisted configuration for the standalone overlay.
//!
//! Stored in its own `overlay_config.json` next to the executable so the
//! overlay never touches the main application's database schema.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::theme::ThemeChoice;

const CONFIG_FILENAME: &str = "overlay_config.json";

/// Metrics that can be displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Total,
    Cpu,
    Gpu,
    Ram,
    Disk,
    Network,
    /// Top-N most power-hungry processes (count configurable).
    TopApps,
}

impl Metric {
    pub const DEFAULT_ORDER: &[Metric] = &[Metric::Total, Metric::Cpu, Metric::Gpu, Metric::Ram, Metric::TopApps];

    pub const ALL: &[Metric] = &[
        Metric::Total,
        Metric::Cpu,
        Metric::Gpu,
        Metric::Ram,
        Metric::Disk,
        Metric::Network,
        Metric::TopApps,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Metric::Total => "total",
            Metric::Cpu => "cpu",
            Metric::Gpu => "gpu",
            Metric::Ram => "ram",
            Metric::Disk => "disk",
            Metric::Network => "network",
            Metric::TopApps => "top_apps",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Metric::Total => "Total",
            Metric::Cpu => "CPU",
            Metric::Gpu => "GPU",
            Metric::Ram => "RAM",
            Metric::Disk => "Disk",
            Metric::Network => "Net",
            Metric::TopApps => "Top",
        }
    }

    /// Single-letter label used by the abbreviated mode.
    pub fn short_label(self) -> &'static str {
        match self {
            Metric::Total => "T",
            Metric::Cpu => "C",
            Metric::Gpu => "G",
            Metric::Ram => "R",
            Metric::Disk => "D",
            Metric::Network => "N",
            Metric::TopApps => "Top",
        }
    }

    /// Whether this metric expands into multiple rows (Top-N apps).
    pub fn is_multi(self) -> bool {
        self == Metric::TopApps
    }
}

/// Widget orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// One metric per line.
    #[default]
    Vertical,
    /// All metrics on a single compact line (MSI-Afterburner OSD style).
    Horizontal,
}

impl Layout {
    pub const ALL: &[Layout] = &[Layout::Vertical, Layout::Horizontal];
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Vertical => write!(f, "Vertical"),
            Layout::Horizontal => write!(f, "Horizontal"),
        }
    }
}

/// Layout density (padding / spacing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Ultra,
    #[default]
    Compact,
    Normal,
}

impl Density {
    pub const ALL: &[Density] = &[Density::Ultra, Density::Compact, Density::Normal];

    pub fn padding(self) -> f32 {
        match self {
            Density::Ultra => 4.0,
            Density::Compact => 6.0,
            Density::Normal => 8.0,
        }
    }

    pub fn spacing(self) -> f32 {
        match self {
            Density::Ultra => 3.0,
            Density::Compact => 5.0,
            Density::Normal => 7.0,
        }
    }

    pub fn row_height(self) -> f32 {
        match self {
            Density::Ultra => 15.0,
            Density::Compact => 18.0,
            Density::Normal => 21.0,
        }
    }
}

impl std::fmt::Display for Density {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Density::Ultra => "Ultra",
            Density::Compact => "Compact",
            Density::Normal => "Normal",
        };
        write!(f, "{s}")
    }
}

/// Text size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FontSize {
    #[default]
    Small,
    Medium,
    Large,
}

impl FontSize {
    pub const ALL: &[FontSize] = &[FontSize::Small, FontSize::Medium, FontSize::Large];

    pub fn label(self) -> f32 {
        match self {
            FontSize::Small => 10.5,
            FontSize::Medium => 12.0,
            FontSize::Large => 13.5,
        }
    }

    pub fn value(self) -> f32 {
        match self {
            FontSize::Small => 12.5,
            FontSize::Medium => 14.5,
            FontSize::Large => 16.5,
        }
    }
}

impl std::fmt::Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FontSize::Small => "Small",
            FontSize::Medium => "Medium",
            FontSize::Large => "Large",
        };
        write!(f, "{s}")
    }
}

/// Selectable card background color.
///
/// Alpha cannot be controlled per-element on Windows (the swapchain only offers
/// `Opaque`, so translucency is applied to the whole layered window), but the
/// *hue* is free — this is how you tune the card's look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BgColor {
    /// Follow the theme.
    #[default]
    Auto,
    Slate,
    Graphite,
    Navy,
    Plum,
    Forest,
    Sand,
    White,
}

impl BgColor {
    pub const ALL: &[BgColor] = &[
        BgColor::Auto,
        BgColor::Slate,
        BgColor::Graphite,
        BgColor::Navy,
        BgColor::Plum,
        BgColor::Forest,
        BgColor::Sand,
        BgColor::White,
    ];

    /// The RGB triple, or `None` to keep the theme's color.
    pub fn rgb(self) -> Option<(f32, f32, f32)> {
        Some(match self {
            BgColor::Auto => return None,
            BgColor::Slate => (0.106, 0.118, 0.153),
            BgColor::Graphite => (0.13, 0.13, 0.14),
            BgColor::Navy => (0.07, 0.11, 0.22),
            BgColor::Plum => (0.17, 0.09, 0.20),
            BgColor::Forest => (0.07, 0.16, 0.12),
            BgColor::Sand => (0.76, 0.71, 0.60),
            BgColor::White => (0.96, 0.97, 0.99),
        })
    }
}

impl std::fmt::Display for BgColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BgColor::Auto => "Auto",
            BgColor::Slate => "Slate",
            BgColor::Graphite => "Graphite",
            BgColor::Navy => "Navy",
            BgColor::Plum => "Plum",
            BgColor::Forest => "Forest",
            BgColor::Sand => "Sand",
            BgColor::White => "White",
        };
        write!(f, "{s}")
    }
}

/// Selectable text color. Separate from the background so contrast stays
/// tunable even though alpha cannot be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextColor {
    /// Follow the theme.
    #[default]
    Auto,
    White,
    Silver,
    Cyan,
    Green,
    Amber,
    Red,
    Ink,
}

impl TextColor {
    pub const ALL: &[TextColor] = &[
        TextColor::Auto,
        TextColor::White,
        TextColor::Silver,
        TextColor::Cyan,
        TextColor::Green,
        TextColor::Amber,
        TextColor::Red,
        TextColor::Ink,
    ];

    /// The RGB triple, or `None` to keep the theme's color.
    pub fn rgb(self) -> Option<(f32, f32, f32)> {
        Some(match self {
            TextColor::Auto => return None,
            TextColor::White => (0.97, 0.98, 1.0),
            TextColor::Silver => (0.72, 0.76, 0.82),
            TextColor::Cyan => (0.20, 0.88, 0.95),
            TextColor::Green => (0.45, 0.92, 0.45),
            TextColor::Amber => (0.98, 0.76, 0.25),
            TextColor::Red => (0.98, 0.45, 0.45),
            TextColor::Ink => (0.06, 0.08, 0.12),
        })
    }
}

impl std::fmt::Display for TextColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TextColor::Auto => "Auto",
            TextColor::White => "White",
            TextColor::Silver => "Silver",
            TextColor::Cyan => "Cyan",
            TextColor::Green => "Green",
            TextColor::Amber => "Amber",
            TextColor::Red => "Red",
            TextColor::Ink => "Ink",
        };
        write!(f, "{s}")
    }
}

/// How the window achieves translucency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Transparency {
    /// Per-pixel alpha through the window surface (best quality). Needs a
    /// backend that exposes alpha — Vulkan does, DX12 usually does not.
    #[default]
    Auto,
    /// Win32 layered window with uniform alpha (GPU-independent fallback: the
    /// whole window, text included, is composited at one alpha).
    Layered,
    /// Fully opaque.
    Off,
}

impl Transparency {
    pub const ALL: &[Transparency] = &[Transparency::Auto, Transparency::Layered, Transparency::Off];

    /// Whether to composite through a Win32 layered window. `Auto` picks the
    /// layered path on Windows because DX12 drops surface alpha, and the
    /// per-pixel path elsewhere.
    pub fn uses_layered(self) -> bool {
        // Kept as an explicit `match` rather than `matches!`: the `Auto` arm is
        // `cfg`-dependent, so collapsing it would silently become wrong on the
        // other platform.
        match self {
            Transparency::Auto => cfg!(target_os = "windows"),
            Transparency::Layered => true,
            Transparency::Off => false,
        }
    }

    /// Whether the OS window is created with per-pixel alpha.
    pub fn transparent_window(self) -> bool {
        match self {
            Transparency::Auto => !cfg!(target_os = "windows"),
            _ => false,
        }
    }

    pub fn enabled(self) -> bool {
        !matches!(self, Transparency::Off)
    }
}

impl std::fmt::Display for Transparency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Transparency::Auto => "Auto",
            Transparency::Layered => "Layered",
            Transparency::Off => "Off",
        };
        write!(f, "{s}")
    }
}

/// All persisted overlay preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    // appearance
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    /// Card background swatch (`Auto` follows the theme).
    #[serde(default)]
    pub bg_color: BgColor,
    /// Text swatch (`Auto` follows the theme).
    #[serde(default)]
    pub text_color: TextColor,
    #[serde(default)]
    pub transparency: Transparency,
    #[serde(default)]
    pub layout: Layout,
    #[serde(default)]
    pub density: Density,
    #[serde(default)]
    pub font_size: FontSize,
    #[serde(default)]
    pub theme: ThemeChoice,
    #[serde(default = "default_true")]
    pub show_labels: bool,
    #[serde(default = "default_true")]
    pub show_units: bool,
    /// Collapse metric labels to a single letter (`Total` -> `T`, `CPU` -> `C`).
    #[serde(default)]
    pub abbreviated: bool,
    #[serde(default = "default_decimals")]
    pub decimals: u8,
    #[serde(default = "default_refresh")]
    pub refresh_secs: u32,

    // window
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    /// Width used by the **vertical** layout, in logical pixels. The horizontal
    /// layout ignores it and measures its own content instead.
    #[serde(default = "default_width")]
    pub width: f32,
    /// Whether the overlay should be running. The main window flips this to
    /// `false` to close the overlay; the overlay only ever reads it, so a
    /// standalone `--overlay` run is unaffected by a stale value.
    #[serde(default = "default_true")]
    pub overlay_requested: bool,
    /// "Pin mode": the overlay stops responding to the mouse so it cannot be
    /// moved by accident. See `pin_click_through` for how far that goes.
    #[serde(default)]
    pub pin_mode: bool,
    /// When set, pin mode also makes the window transparent to the mouse, so
    /// every click lands on whatever is underneath. That is the strict overlay
    /// behaviour, but it means the right-click menu can no longer be reached and
    /// the overlay has to be released from the tray. Pinning always locks the
    /// position, whether or not this is set.
    #[serde(default = "default_true")]
    pub pin_click_through: bool,
    #[serde(default)]
    pub blur: bool,
    #[serde(default)]
    pub position: Option<(f32, f32)>,

    // content
    #[serde(default = "default_metrics")]
    pub metrics: Vec<Metric>,
    #[serde(default = "default_top_apps")]
    pub top_apps: usize,
}

fn default_opacity() -> f32 {
    0.80
}
fn default_width() -> f32 {
    140.0
}
fn default_true() -> bool {
    true
}
fn default_decimals() -> u8 {
    1
}
fn default_refresh() -> u32 {
    1
}
fn default_top_apps() -> usize {
    3
}
fn default_metrics() -> Vec<Metric> {
    Metric::DEFAULT_ORDER.to_vec()
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            opacity: default_opacity(),
            bg_color: BgColor::default(),
            text_color: TextColor::default(),
            transparency: Transparency::default(),
            layout: Layout::default(),
            density: Density::default(),
            font_size: FontSize::default(),
            theme: ThemeChoice::default(),
            show_labels: true,
            show_units: true,
            abbreviated: false,
            decimals: default_decimals(),
            refresh_secs: default_refresh(),
            always_on_top: true,
            width: default_width(),
            overlay_requested: true,
            pin_mode: false,
            pin_click_through: true,
            blur: false,
            position: None,
            metrics: default_metrics(),
            top_apps: default_top_apps(),
        }
    }
}

impl OverlayConfig {
    /// Config file path (next to the executable).
    pub fn path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join(CONFIG_FILENAME)))
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILENAME))
    }

    pub fn load() -> Option<Self> {
        let contents = std::fs::read_to_string(Self::path()).ok()?;
        serde_json::from_str(&contents).ok()
    }

    pub fn save(&self) -> bool {
        match serde_json::to_string_pretty(self) {
            Ok(json) => std::fs::write(Self::path(), json).is_ok(),
            Err(_) => false,
        }
    }

    /// Clamps `top_apps` into the supported 1..=8 range.
    pub fn top_apps(&self) -> usize {
        self.top_apps.clamp(1, 8)
    }

    /// Number of rendered lines in vertical layout.
    fn content_rows(&self) -> usize {
        let mut rows = 0usize;
        for m in &self.metrics {
            rows += if m.is_multi() { self.top_apps() } else { 1 };
        }
        rows.max(1)
    }

    /// Window height that fits the content (used when `fit_height` is on).
    pub fn fitted_height(&self) -> f32 {
        let pad = self.density.padding();
        let spacing = self.density.spacing();
        let row = self.density.row_height();

        // No header is drawn in metrics mode, so it must not be counted here —
        // doing so used to leave a visible empty strip under the text.
        let content = match self.layout {
            Layout::Horizontal => row,
            Layout::Vertical => {
                let rows = self.content_rows() as f32;
                rows * row + (rows - 1.0).max(0.0) * spacing
            }
        };
        // A single horizontal line needs far less room than a stacked list.
        let floor = match self.layout {
            Layout::Horizontal => 16.0,
            Layout::Vertical => 32.0,
        };
        (pad * 2.0 + content).ceil().max(floor)
    }
}
