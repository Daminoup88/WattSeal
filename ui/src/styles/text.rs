use iced::{
    Color,
    widget::text::{self, Catalog},
};

use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum TextStyle {
    #[default]
    Default,
    Primary,
    Success,
    Danger,
    Warning,
    Muted,
    Title,
    Subtitle,
    Label,
}

impl TextStyle {
    fn get_style(&self, theme: &AppTheme) -> text::Style {
        text::Style {
            color: Some(self.color(theme)),
        }
    }

    pub fn color(&self, theme: &AppTheme) -> Color {
        let palette = theme.palette();

        match self {
            TextStyle::Default | TextStyle::Title => palette.text,
            TextStyle::Primary => palette.primary,
            TextStyle::Success => palette.success,
            TextStyle::Danger => palette.danger,
            TextStyle::Warning => palette.warning,
            TextStyle::Muted | TextStyle::Label => with_alpha(palette.text, 0.6),
            TextStyle::Subtitle => with_alpha(palette.text, 0.8),
        }
    }
}

impl Catalog for AppTheme {
    type Class<'a> = TextStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        class.get_style(self)
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}
