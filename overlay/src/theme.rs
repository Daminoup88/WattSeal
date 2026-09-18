//! Self-contained palette for the overlay.
//!
//! The overlay deliberately owns its colors so it stays **decoupled** from the
//! main application's theme system. Rendering uses `iced`'s built-in [`Theme`]
//! for widget defaults plus closure-based styles for the card/values, which
//! means no dependency on the `ui` crate is required.

use iced::{Color, Theme};
use serde::{Deserialize, Serialize};

use crate::config::{BgColor, TextColor};

/// Overlay color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
}

impl ThemeChoice {
    pub const ALL: &[ThemeChoice] = &[ThemeChoice::Dark, ThemeChoice::Light];
}

/// Resolved color set for a scheme.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Base card color (alpha is applied by the configured opacity).
    pub card: Color,
    /// Primary foreground text.
    pub text: Color,
    /// Secondary / label text (already low-alpha).
    pub muted: Color,
    /// Accent (headline value color).
    pub accent: Color,
    /// Success / green value color.
    pub green: Color,
    /// Warning / amber value color.
    pub amber: Color,
    /// Card border color.
    pub border: Color,
    /// Shadow color.
    pub shadow: Color,
}

impl Palette {
    /// Card background with the user opacity applied.
    pub fn card_with_alpha(self, alpha: f32) -> Color {
        Color { a: alpha, ..self.card }
    }
}

/// Returns the palette for the given scheme.
pub fn palette(choice: ThemeChoice) -> Palette {
    match choice {
        ThemeChoice::Dark => Palette {
            card: Color::from_rgb(0.106, 0.118, 0.153), // #1b1e27
            text: Color::from_rgb(0.90, 0.94, 1.0),
            muted: Color::from_rgba(0.90, 0.94, 1.0, 0.55),
            accent: Color::from_rgb(0.0, 0.80, 0.90), // #00cce6
            green: Color::from_rgb(0.55, 1.0, 0.40),
            amber: Color::from_rgb(0.95, 0.72, 0.15),
            border: Color::from_rgba(0.0, 0.80, 0.90, 0.35),
            shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
        },
        ThemeChoice::Light => Palette {
            card: Color::from_rgb(0.937, 0.961, 0.988), // #eff5fc
            text: Color::from_rgb(0.08, 0.12, 0.20),
            muted: Color::from_rgba(0.08, 0.12, 0.20, 0.55),
            accent: Color::from_rgb(0.0, 0.62, 0.75), // #009ec0
            green: Color::from_rgb(0.20, 0.62, 0.22),
            amber: Color::from_rgb(0.78, 0.48, 0.0),
            border: Color::from_rgba(0.0, 0.62, 0.75, 0.35),
            shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
        },
    }
}

/// Returns the scheme's palette with the user's color overrides applied.
///
/// Alpha is deliberately *not* part of this: on Windows the whole layered
/// window carries a single alpha, so per-element translucency is impossible.
/// Hue and lightness, however, are fully controllable — which is what these
/// two swatches expose.
pub fn palette_with(choice: ThemeChoice, background: BgColor, foreground: TextColor) -> Palette {
    let mut result = palette(choice);

    if let Some((r, g, b)) = background.rgb() {
        result.card = Color::from_rgb(r, g, b);
        // Keep the drop shadow in the same family as the card.
        result.shadow = Color::from_rgba(r * 0.3, g * 0.3, b * 0.3, 0.35);
    }

    if let Some((r, g, b)) = foreground.rgb() {
        result.text = Color::from_rgb(r, g, b);
        result.muted = Color::from_rgba(r, g, b, 0.55);
    }

    result
}

/// The iced theme handed to widgets (labels, pick-lists, checkboxes…).
pub fn iced_theme(choice: ThemeChoice) -> Theme {
    match choice {
        ThemeChoice::Dark => Theme::Dark,
        ThemeChoice::Light => Theme::Light,
    }
}
