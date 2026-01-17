use iced::{Color, theme::Palette};

use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy)]
pub struct ExtendedPalette {
    // Base colors from iced::Palette
    pub background: Color,
    pub text: Color,
    pub primary: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,

    // Extended colors
    pub card_background: Color,
    pub elevated_background: Color,
    pub border: Color,
    pub border_subtle: Color,
    pub text_muted: Color,
    pub text_subtle: Color,

    // States
    pub hover_overlay: Color,
    pub pressed_overlay: Color,

    // Mode indicator
    pub is_dark: bool,
}

impl ExtendedPalette {
    pub fn from_theme(theme: &AppTheme) -> Self {
        let palette = theme.palette();
        let is_dark = is_dark(&palette);

        Self {
            background: palette.background,
            text: palette.text,
            primary: palette.primary,
            success: palette.success,
            danger: palette.danger,
            warning: palette.warning,

            card_background: if is_dark {
                lighten(palette.background, 0.08)
            } else {
                darken(palette.background, 0.03)
            },
            elevated_background: if is_dark {
                lighten(palette.background, 0.12)
            } else {
                darken(palette.background, 0.05)
            },
            border: with_alpha(palette.text, 0.2),
            border_subtle: with_alpha(palette.text, 0.1),
            text_muted: with_alpha(palette.text, 0.6),
            text_subtle: with_alpha(palette.text, 0.4),
            hover_overlay: with_alpha(palette.text, 0.08),
            pressed_overlay: with_alpha(palette.text, 0.12),
            is_dark,
        }
    }
}

/// Check if a palette represents a dark theme
pub fn is_dark(palette: &Palette) -> bool {
    luminance(palette.background) < 0.5
}

/// Calculate perceived luminance of a color
pub fn luminance(color: Color) -> f32 {
    0.299 * color.r + 0.587 * color.g + 0.114 * color.b
}

/// Apply alpha to a color
pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// Lighten a color by a given amount
pub fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

/// Darken a color by a given amount
pub fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}

/// Get a contrasting text color (black or white) for a given background
pub fn contrast_text(background: Color) -> Color {
    if luminance(background) > 0.5 {
        Color::BLACK
    } else {
        Color::WHITE
    }
}
