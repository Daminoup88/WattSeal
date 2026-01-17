use iced::{
    Background, Border, Color, Shadow, Vector,
    widget::container::{self, Catalog},
};

use super::style_constants::{
    ALPHA_LIGHT, ALPHA_MEDIUM, ALPHA_SUBTLE, BORDER_RADIUS_LARGE, BORDER_RADIUS_MEDIUM, BORDER_RADIUS_SMALL,
    BORDER_WIDTH,
};
use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum ContainerStyle {
    #[default]
    Transparent,
    Card,
    Panel,
    Highlighted,
    Badge,
    Header,
    Tooltip,
    ModalBackground,
    PowerCard,
    ComponentCard,
}

impl ContainerStyle {
    fn get_style(&self, theme: &AppTheme) -> container::Style {
        let palette = theme.palette();
        let is_dark = is_dark_theme(theme);

        match self {
            ContainerStyle::Transparent => container::Style::default(),

            ContainerStyle::Card => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.08)
                } else {
                    darken(palette.background, 0.03)
                };
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: with_alpha(palette.text, ALPHA_SUBTLE),
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_MEDIUM.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ContainerStyle::Panel => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.12)
                } else {
                    darken(palette.background, 0.05)
                };
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: with_alpha(palette.text, ALPHA_LIGHT),
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_LARGE.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow {
                        color: Color::BLACK,
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                    ..Default::default()
                }
            }

            ContainerStyle::Highlighted => container::Style {
                background: Some(Background::Color(with_alpha(palette.primary, ALPHA_LIGHT))),
                border: Border {
                    color: palette.primary,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                text_color: Some(palette.text),
                shadow: Shadow::default(),
                ..Default::default()
            },

            ContainerStyle::Badge => {
                let bg_color = with_alpha(palette.primary, ALPHA_MEDIUM);
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: BORDER_RADIUS_SMALL.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ContainerStyle::Header => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.05)
                } else {
                    darken(palette.background, 0.02)
                };
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: with_alpha(palette.text, ALPHA_SUBTLE),
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow {
                        color: Color::BLACK,
                        offset: Vector::new(0.0, 1.0),
                        blur_radius: 3.0,
                    },
                    ..Default::default()
                }
            }

            ContainerStyle::Tooltip => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.15)
                } else {
                    darken(palette.background, 0.08)
                };
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: with_alpha(palette.text, ALPHA_LIGHT),
                        width: BORDER_WIDTH / 2.0,
                        radius: BORDER_RADIUS_SMALL.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow {
                        color: Color::BLACK,
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 6.0,
                    },
                    ..Default::default()
                }
            }

            ContainerStyle::ModalBackground => container::Style {
                background: Some(Background::Color(Color { a: 0.7, ..Color::BLACK })),
                border: Border::default(),
                text_color: None,
                shadow: Shadow::default(),
                ..Default::default()
            },

            ContainerStyle::PowerCard => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.1)
                } else {
                    darken(palette.background, 0.04)
                };
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: with_alpha(palette.primary, ALPHA_MEDIUM),
                        width: BORDER_WIDTH * 1.5,
                        radius: BORDER_RADIUS_LARGE.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow {
                        color: with_alpha(palette.primary, ALPHA_LIGHT),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                }
            }

            ContainerStyle::ComponentCard => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.06)
                } else {
                    darken(palette.background, 0.02)
                };
                container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: with_alpha(palette.text, ALPHA_SUBTLE),
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_MEDIUM.into(),
                    },
                    text_color: Some(palette.text),
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }
        }
    }
}

impl Catalog for AppTheme {
    type Class<'a> = ContainerStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        class.get_style(self)
    }
}

fn is_dark_theme(theme: &AppTheme) -> bool {
    let palette = theme.palette();
    let luminance = 0.299 * palette.background.r + 0.587 * palette.background.g + 0.114 * palette.background.b;
    luminance < 0.5
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}
