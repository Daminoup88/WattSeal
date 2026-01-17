use iced::{
    Background, Border, Color, Shadow, Vector,
    widget::container::{self, Catalog},
};

use super::{
    colors::{ExtendedPalette, with_alpha},
    style_constants::{BORDER_RADIUS_LARGE, BORDER_RADIUS_MEDIUM, BORDER_WIDTH},
};
use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum ContainerStyle {
    #[default]
    Transparent,
    Card,
    Header,
    PowerCard,
    ComponentCard,
}

impl Catalog for AppTheme {
    type Class<'a> = ContainerStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let ext = ExtendedPalette::from_theme(self);

        match class {
            ContainerStyle::Transparent => container::Style::default(),

            ContainerStyle::Card => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border {
                    color: ext.border_subtle,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow::default(),
                ..Default::default()
            },

            ContainerStyle::Header => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border::default(),
                text_color: Some(ext.text),
                shadow: Shadow {
                    color: with_alpha(Color::BLACK, 0.15),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                ..Default::default()
            },

            ContainerStyle::PowerCard => container::Style {
                background: Some(Background::Color(ext.elevated_background)),
                border: Border {
                    color: with_alpha(ext.primary, 0.4),
                    width: BORDER_WIDTH * 1.5,
                    radius: BORDER_RADIUS_LARGE.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow {
                    color: with_alpha(ext.primary, 0.2),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            },

            ContainerStyle::ComponentCard => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border {
                    color: ext.border_subtle,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow::default(),
                ..Default::default()
            },
        }
    }
}
