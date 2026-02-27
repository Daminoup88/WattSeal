use iced::widget::rule::{self, Catalog};

use super::colors::{ExtendedPalette, with_alpha};
use crate::themes::AppTheme;

pub enum RuleStyle<'a> {
    Standard,
    Subtle,
    Strong,
    Custom(rule::StyleFn<'a, AppTheme>),
}

impl<'a> Default for RuleStyle<'a> {
    fn default() -> Self {
        Self::Standard
    }
}

impl<'a> From<rule::StyleFn<'a, AppTheme>> for RuleStyle<'a> {
    fn from(style_fn: rule::StyleFn<'a, AppTheme>) -> Self {
        Self::Custom(style_fn)
    }
}

impl Catalog for AppTheme {
    type Class<'a> = RuleStyle<'a>;

    fn default<'a>() -> Self::Class<'a> {
        RuleStyle::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        let ext = ExtendedPalette::from_theme(self);

        match class {
            RuleStyle::Standard => rule::Style {
                color: ext.border,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            },
            RuleStyle::Subtle => rule::Style {
                color: ext.border_subtle,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            },
            RuleStyle::Strong => rule::Style {
                color: with_alpha(ext.text, 0.35),
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            },
            RuleStyle::Custom(style_fn) => style_fn(self),
        }
    }
}
