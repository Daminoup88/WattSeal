use iced::{
    Background, Border, Color, Shadow,
    widget::scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style},
};

use super::colors::{ExtendedPalette, with_alpha};
use crate::themes::AppTheme;

const SCROLLBAR_RADIUS: f32 = 4.0;

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Standard,
}

impl Catalog for AppTheme {
    type Class<'a> = ScrollableStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        let ext = ExtendedPalette::from_theme(self);

        let (scrollbar_alpha, scroller_alpha, v_hovered, h_hovered) = match status {
            Status::Active { .. } => (0.3, 0.5, false, false),
            Status::Hovered {
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
                ..
            } => (0.4, 0.7, is_vertical_scrollbar_hovered, is_horizontal_scrollbar_hovered),
            Status::Dragged {
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
                ..
            } => (0.4, 0.7, is_vertical_scrollbar_dragged, is_horizontal_scrollbar_dragged),
        };

        let scrollbar_bg = with_alpha(ext.text, scrollbar_alpha);
        let scroller_default = with_alpha(ext.text, scroller_alpha);

        let rail = |hovered: bool| Rail {
            background: Some(Background::Color(scrollbar_bg)),
            border: Border {
                radius: SCROLLBAR_RADIUS.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            scroller: Scroller {
                background: Background::Color(if hovered { ext.primary } else { scroller_default }),
                border: Border {
                    radius: SCROLLBAR_RADIUS.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            },
        };

        Style {
            container: iced::widget::container::Style::default(),
            vertical_rail: rail(v_hovered),
            horizontal_rail: rail(h_hovered),
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(ext.background),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: ext.text,
            },
        }
    }
}
