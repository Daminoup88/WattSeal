use iced::widget::text::{self, Catalog};

use super::colors::ExtendedPalette;
use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum TextStyle {
    #[default]
    Default,
    Primary,
    Success,
    Muted,
    Subtitle,
}

impl Catalog for AppTheme {
    type Class<'a> = TextStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        let ext = ExtendedPalette::from_theme(self);

        let color = match class {
            TextStyle::Default => ext.text,
            TextStyle::Primary => ext.primary,
            TextStyle::Success => ext.success,
            TextStyle::Muted => ext.text_muted,
            TextStyle::Subtitle => ext.text_muted,
        };

        text::Style { color: Some(color) }
    }
}
