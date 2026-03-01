use iced::{
    Background, Border, Color, Shadow, Vector,
    widget::button::{self, Catalog, Status},
};

use super::{
    colors::{ExtendedPalette, blend, with_alpha},
    style_constants::{BORDER_RADIUS_SMALL, BORDER_WIDTH},
};
use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonStyle {
    #[default]
    Standard,
    Toggle(bool),
    Nav,
    NavActive,
    InfoHelp,
}

impl Catalog for AppTheme {
    type Class<'a> = ButtonStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> button::Style {
        let ext = ExtendedPalette::from_theme(self);
        let base = class.base_style(&ext);

        match status {
            Status::Active => base,
            Status::Hovered => class.hovered_style(base, &ext),
            Status::Pressed => class.pressed_style(base, &ext),
            Status::Disabled => disabled_style(&ext),
        }
    }
}

impl ButtonStyle {
    fn base_style(&self, ext: &ExtendedPalette) -> button::Style {
        match self {
            Self::Standard => button::Style {
                background: Some(Background::Color(ext.card_background)),
                text_color: ext.text,
                border: Border {
                    color: ext.border,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_SMALL.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },

            Self::Toggle(active) => {
                let color = if *active { ext.success } else { ext.primary };

                button::Style {
                    background: Some(Background::Color(ext.card_background)),
                    text_color: color,
                    border: Border {
                        color,
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_SMALL.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            Self::Nav => button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: ext.text_muted,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: BORDER_RADIUS_SMALL.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },

            Self::NavActive => button::Style {
                background: Some(Background::Color(with_alpha(ext.primary, 0.15))),
                text_color: ext.primary,
                border: Border {
                    color: ext.primary,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_SMALL.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },

            Self::InfoHelp => button::Style {
                background: Some(Background::Color(with_alpha(ext.primary, 0.15))),
                text_color: ext.primary,
                border: Border {
                    color: with_alpha(ext.primary, 0.3),
                    width: BORDER_WIDTH,
                    radius: 12.0.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },
        }
    }

    fn hovered_style(&self, base: button::Style, ext: &ExtendedPalette) -> button::Style {
        let hover_bg = match &base.background {
            Some(Background::Color(c)) if *c == Color::TRANSPARENT => ext.hover_overlay,
            Some(Background::Color(c)) => blend(*c, ext.hover_overlay),
            _ => ext.hover_overlay,
        };

        button::Style {
            background: Some(Background::Color(hover_bg)),
            text_color: if matches!(self, Self::Nav) {
                ext.text
            } else {
                base.text_color
            },
            border: if let Self::Toggle(_) = self {
                base.border
            } else {
                Border {
                    color: if matches!(self, Self::Standard) {
                        ext.primary
                    } else {
                        base.border.color
                    },
                    ..base.border
                }
            },
            shadow: Shadow {
                color: with_alpha(Color::BLACK, 0.1),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..base
        }
    }

    fn pressed_style(&self, base: button::Style, ext: &ExtendedPalette) -> button::Style {
        let pressed_bg = match &base.background {
            Some(Background::Color(c)) if *c == Color::TRANSPARENT => ext.pressed_overlay,
            Some(Background::Color(c)) => blend(*c, ext.pressed_overlay),
            _ => ext.pressed_overlay,
        };

        button::Style {
            background: Some(Background::Color(pressed_bg)),
            shadow: Shadow::default(),
            ..base
        }
    }
}

fn disabled_style(ext: &ExtendedPalette) -> button::Style {
    button::Style {
        background: Some(Background::Color(ext.border_subtle)),
        text_color: ext.text_subtle,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: BORDER_RADIUS_SMALL.into(),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}
