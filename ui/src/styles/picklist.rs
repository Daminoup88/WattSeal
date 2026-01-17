use iced::{
    Background, Border, Shadow,
    overlay::menu::{Catalog as MenuCatalog, Style as MenuStyle},
    widget::pick_list::{Catalog as PickListCatalog, Status, Style as ListStyle},
};

use super::{colors::ExtendedPalette, style_constants::BORDER_WIDTH};
use crate::themes::AppTheme;

const PICKLIST_BORDER_RADIUS: f32 = 8.0;

#[derive(Default)]
pub enum PickListStyle {
    #[default]
    Standard,
}

impl PickListCatalog for AppTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as PickListCatalog>::Class<'a> {
        PickListStyle::default()
    }

    fn style(&self, _class: &<Self as PickListCatalog>::Class<'_>, status: Status) -> ListStyle {
        let ext = ExtendedPalette::from_theme(self);

        let border = match status {
            Status::Active => Border {
                radius: PICKLIST_BORDER_RADIUS.into(),
                width: 0.0,
                color: ext.border,
            },
            Status::Hovered | Status::Opened { .. } => Border {
                radius: PICKLIST_BORDER_RADIUS.into(),
                width: BORDER_WIDTH,
                color: ext.primary,
            },
        };

        ListStyle {
            text_color: ext.text,
            placeholder_color: ext.text_muted,
            handle_color: ext.text,
            background: Background::Color(ext.background),
            border,
        }
    }
}

impl MenuCatalog for AppTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as MenuCatalog>::Class<'a> {
        PickListStyle::default()
    }

    fn style(&self, _class: &<Self as MenuCatalog>::Class<'_>) -> MenuStyle {
        let ext = ExtendedPalette::from_theme(self);

        MenuStyle {
            text_color: ext.text,
            background: Background::Color(ext.background),
            border: Border {
                width: BORDER_WIDTH,
                radius: PICKLIST_BORDER_RADIUS.into(),
                color: ext.primary,
            },
            selected_text_color: ext.text,
            selected_background: Background::Color(ext.primary),
            shadow: Shadow::default(),
        }
    }
}
