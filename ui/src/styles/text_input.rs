use iced::{
    Background, Border, Color,
    widget::text_input::{Catalog, Status, Style},
};

use super::{
    colors::{ExtendedPalette, with_alpha},
    style_constants::{BORDER_RADIUS_SMALL, BORDER_WIDTH},
};
use crate::themes::AppTheme;

#[derive(Default, Clone, Copy)]
pub enum TextInputStyle {
    #[default]
    Standard,
}

impl Catalog for AppTheme {
    type Class<'a> = TextInputStyle;

    fn default<'a>() -> Self::Class<'a> {
        TextInputStyle::default()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        let ext = ExtendedPalette::from_theme(self);

        let (border_color, background) = match status {
            Status::Active => (ext.border, ext.card_background),
            Status::Hovered => (with_alpha(ext.primary, 0.6), ext.card_background),
            Status::Focused { .. } => (ext.primary, ext.card_background),
            Status::Disabled => (ext.border, with_alpha(ext.card_background, 0.5)),
        };

        Style {
            background: Background::Color(background),
            border: Border {
                color: border_color,
                width: BORDER_WIDTH,
                radius: BORDER_RADIUS_SMALL.into(),
            },
            icon: ext.text_muted,
            placeholder: ext.text_muted,
            value: ext.text,
            selection: with_alpha(ext.primary, 0.35),
        }
    }
}
