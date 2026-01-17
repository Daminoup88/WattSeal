//! Scrollable style

use iced::{
    Background, Border, Color, Shadow,
    widget::scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style},
};

use crate::themes::AppTheme;

const SCROLLBAR_RADIUS: f32 = 4.0;

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Standard,
}

impl ScrollableStyle {
    fn active(&self, theme: &AppTheme) -> Style {
        let palette = theme.palette();
        let scrollbar_color = Color { a: 0.3, ..palette.text };
        let scroller_color = Color { a: 0.5, ..palette.text };

        Style {
            container: iced::widget::container::Style::default(),
            vertical_rail: Rail {
                background: Some(Background::Color(scrollbar_color)),
                border: Border {
                    radius: SCROLLBAR_RADIUS.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        radius: SCROLLBAR_RADIUS.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            horizontal_rail: Rail {
                background: Some(Background::Color(scrollbar_color)),
                border: Border {
                    radius: SCROLLBAR_RADIUS.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        radius: SCROLLBAR_RADIUS.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(palette.background),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: palette.text,
            },
        }
    }

    fn hovered(&self, theme: &AppTheme, is_horizontal_hovered: bool, is_vertical_hovered: bool) -> Style {
        let palette = theme.palette();
        let scrollbar_color = Color { a: 0.4, ..palette.text };
        let scroller_color = Color { a: 0.7, ..palette.text };
        let scroller_hovered_color = palette.primary;

        let vertical_scroller_color = if is_vertical_hovered {
            scroller_hovered_color
        } else {
            scroller_color
        };

        let horizontal_scroller_color = if is_horizontal_hovered {
            scroller_hovered_color
        } else {
            scroller_color
        };

        Style {
            container: iced::widget::container::Style::default(),
            vertical_rail: Rail {
                background: Some(Background::Color(scrollbar_color)),
                border: Border {
                    radius: SCROLLBAR_RADIUS.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: Scroller {
                    background: Background::Color(vertical_scroller_color),
                    border: Border {
                        radius: SCROLLBAR_RADIUS.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            horizontal_rail: Rail {
                background: Some(Background::Color(scrollbar_color)),
                border: Border {
                    radius: SCROLLBAR_RADIUS.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: Scroller {
                    background: Background::Color(horizontal_scroller_color),
                    border: Border {
                        radius: SCROLLBAR_RADIUS.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(palette.background),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: palette.text,
            },
        }
    }
}

impl Catalog for AppTheme {
    type Class<'a> = ScrollableStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active { .. } => class.active(self),
            Status::Hovered {
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
                ..
            } => class.hovered(self, is_horizontal_scrollbar_hovered, is_vertical_scrollbar_hovered),
            Status::Dragged {
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
                ..
            } => class.hovered(self, is_horizontal_scrollbar_dragged, is_vertical_scrollbar_dragged),
        }
    }
}
