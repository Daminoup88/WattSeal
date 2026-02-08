use iced::{
    Background, Border, Color, Shadow, Vector,
    overlay::menu::{Catalog as MenuCatalog, Style as MenuStyle},
    widget::pick_list::{Catalog as PickListCatalog, Status, Style as ListStyle},
};

use super::{
    colors::{ExtendedPalette, blend, with_alpha},
    style_constants::{BORDER_RADIUS_SMALL, BORDER_WIDTH},
};
use crate::themes::AppTheme;

const PICKLIST_BORDER_RADIUS: f32 = 8.0;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum PickListStyle {
    #[default]
    Standard,
    TimeRange,
}

impl PickListCatalog for AppTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as PickListCatalog>::Class<'a> {
        PickListStyle::default()
    }

    fn style(&self, class: &<Self as PickListCatalog>::Class<'_>, status: Status) -> ListStyle {
        let ext = ExtendedPalette::from_theme(self);

        match class {
            PickListStyle::Standard => {
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
            PickListStyle::TimeRange => {
                let base_bg = ext.card_background;
                let background = match status {
                    Status::Active => base_bg,
                    Status::Hovered | Status::Opened { .. } => blend(base_bg, ext.hover_overlay),
                };

                ListStyle {
                    text_color: ext.text,
                    placeholder_color: ext.text_muted,
                    handle_color: ext.text,
                    background: Background::Color(background),
                    border: Border {
                        radius: BORDER_RADIUS_SMALL.into(),
                        width: BORDER_WIDTH,
                        color: ext.border,
                    },
                }
            }
        }
    }
}

impl MenuCatalog for AppTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as MenuCatalog>::Class<'a> {
        PickListStyle::default()
    }

    fn style(&self, class: &<Self as MenuCatalog>::Class<'_>) -> MenuStyle {
        let ext = ExtendedPalette::from_theme(self);

        match class {
            PickListStyle::Standard => MenuStyle {
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
            },
            PickListStyle::TimeRange => MenuStyle {
                text_color: ext.text,
                background: Background::Color(ext.card_background),
                border: Border {
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_SMALL.into(),
                    color: ext.border,
                },
                selected_text_color: ext.text,
                selected_background: Background::Color(blend(ext.card_background, ext.hover_overlay)),
                shadow: Shadow::default(),
            },
        }
    }
}
