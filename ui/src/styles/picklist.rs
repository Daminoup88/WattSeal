use iced::{
    Background, Border, Color, Shadow,
    widget::pick_list::{Catalog, Status, Style},
};

use super::style_constants::BORDER_WIDTH;
use crate::themes::AppTheme;

const PICKLIST_BORDER_RADIUS: f32 = 8.0;

#[derive(Default)]
pub enum PickListStyle {
    #[default]
    Standard,
}

impl PickListStyle {
    fn active(&self, theme: &AppTheme) -> Style {
        let palette = theme.palette();
        Style {
            text_color: palette.text,
            placeholder_color: palette.text,
            handle_color: palette.text,
            background: Background::Color(palette.background),
            border: Border {
                radius: PICKLIST_BORDER_RADIUS.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, theme: &AppTheme) -> Style {
        let palette = theme.palette();
        Style {
            text_color: palette.text,
            placeholder_color: palette.text,
            handle_color: palette.text,
            background: Background::Color(palette.background),
            border: Border {
                radius: PICKLIST_BORDER_RADIUS.into(),
                width: BORDER_WIDTH,
                color: palette.primary,
            },
        }
    }

    fn menu_style(&self, theme: &AppTheme) -> iced::overlay::menu::Style {
        let palette = theme.palette();
        iced::overlay::menu::Style {
            text_color: palette.text,
            background: Background::Color(palette.background),
            border: Border {
                width: BORDER_WIDTH,
                radius: PICKLIST_BORDER_RADIUS.into(),
                color: palette.primary,
            },
            selected_text_color: palette.text,
            selected_background: Background::Color(palette.primary),
            shadow: Shadow::default(),
        }
    }
}

impl iced::overlay::menu::Catalog for AppTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as iced::overlay::menu::Catalog>::Class<'a> {
        <Self as iced::overlay::menu::Catalog>::Class::default()
    }

    fn style(&self, class: &<Self as iced::overlay::menu::Catalog>::Class<'_>) -> iced::overlay::menu::Style {
        class.menu_style(self)
    }
}

impl Catalog for AppTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        <Self as Catalog>::Class::default()
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => class.active(self),
            Status::Hovered | Status::Opened { .. } => class.hovered(self),
        }
    }
}
