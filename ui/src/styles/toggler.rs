use iced::{
    Background, Color,
    widget::toggler::{Catalog, Style},
};

use super::{
    colors::ExtendedPalette,
    style_constants::{BORDER_RADIUS_LARGE, BORDER_WIDTH},
};
use crate::themes::AppTheme;

#[derive(Default)]
pub enum TogglerStyle {
    #[default]
    Standard,
}

impl Catalog for AppTheme {
    type Class<'a> = TogglerStyle;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        TogglerStyle::default()
    }

    fn style(&self, _class: &<Self as Catalog>::Class<'_>, status: iced::widget::toggler::Status) -> Style {
        let ext = ExtendedPalette::from_theme(self);

        match status {
            iced::widget::toggler::Status::Active { is_toggled } => Style {
                background: if is_toggled {
                    Background::Color(ext.primary)
                } else {
                    Background::Color(ext.background)
                },
                background_border_width: if is_toggled { 0.0 } else { BORDER_WIDTH },
                background_border_color: if is_toggled { Color::TRANSPARENT } else { ext.border },
                foreground: if is_toggled {
                    Background::Color(Color::WHITE)
                } else {
                    Background::Color(ext.card_background)
                },
                foreground_border_width: 0.0,
                foreground_border_color: Color::TRANSPARENT,
                text_color: Some(ext.text),
                border_radius: Some(BORDER_RADIUS_LARGE.into()),
                padding_ratio: 0.24,
            },
            iced::widget::toggler::Status::Hovered { is_toggled } => Style {
                background: if is_toggled {
                    Background::Color(Color {
                        r: (ext.primary.r * 0.92).min(1.0),
                        g: (ext.primary.g * 0.92).min(1.0),
                        b: (ext.primary.b * 0.92).min(1.0),
                        a: ext.primary.a,
                    })
                } else {
                    Background::Color(ext.background)
                },
                background_border_width: if is_toggled { 0.0 } else { BORDER_WIDTH },
                background_border_color: if is_toggled { Color::TRANSPARENT } else { ext.primary },
                foreground: if is_toggled {
                    Background::Color(Color::WHITE)
                } else {
                    Background::Color(ext.card_background)
                },
                foreground_border_width: 0.0,
                foreground_border_color: Color::TRANSPARENT,
                text_color: Some(ext.text),
                border_radius: Some(BORDER_RADIUS_LARGE.into()),
                padding_ratio: 0.24,
            },
            iced::widget::toggler::Status::Disabled { .. } => Style {
                background: Background::Color(ext.border_subtle),
                background_border_width: 0.0,
                background_border_color: Color::TRANSPARENT,
                foreground: Background::Color(ext.text_subtle),
                foreground_border_width: 0.0,
                foreground_border_color: Color::TRANSPARENT,
                text_color: Some(ext.text_subtle),
                border_radius: Some(BORDER_RADIUS_LARGE.into()),
                padding_ratio: 0.24,
            },
        }
    }
}
