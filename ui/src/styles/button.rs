use iced::{
    Background, Border, Color, Shadow, Vector,
    widget::button::{self, Catalog, Status},
};

use super::style_constants::{
    ALPHA_LIGHT, ALPHA_MEDIUM, ALPHA_SUBTLE, BORDER_RADIUS_MEDIUM, BORDER_RADIUS_ROUND, BORDER_RADIUS_SMALL,
    BORDER_WIDTH,
};
use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonStyle {
    #[default]
    Standard,
    Primary,
    Secondary,
    Danger,
    Success,
    Ghost,
    Nav,
    NavActive,
    Icon,
    Pill,
}

impl ButtonStyle {
    fn get_style(&self, theme: &AppTheme, status: Status) -> button::Style {
        match status {
            Status::Active => self.active(theme),
            Status::Hovered => self.hovered(theme),
            Status::Pressed => self.pressed(theme),
            Status::Disabled => self.disabled(theme),
        }
    }

    fn active(&self, theme: &AppTheme) -> button::Style {
        let palette = theme.palette();
        let is_dark = is_dark_theme(theme);

        match self {
            ButtonStyle::Standard => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.15)
                } else {
                    darken(palette.background, 0.08)
                };
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: palette.text,
                    border: Border {
                        color: with_alpha(palette.text, ALPHA_LIGHT),
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_MEDIUM.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ButtonStyle::Primary => button::Style {
                background: Some(Background::Color(palette.primary)),
                text_color: contrast_text_color(palette.primary),
                border: Border {
                    color: palette.primary,
                    width: 0.0,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                shadow: Shadow {
                    color: with_alpha(palette.primary, 0.3),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            },

            ButtonStyle::Secondary => {
                let bg_color = with_alpha(palette.primary, ALPHA_SUBTLE);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: palette.primary,
                    border: Border {
                        color: with_alpha(palette.primary, ALPHA_LIGHT),
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_MEDIUM.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ButtonStyle::Danger => button::Style {
                background: Some(Background::Color(palette.danger)),
                text_color: contrast_text_color(palette.danger),
                border: Border {
                    color: palette.danger,
                    width: 0.0,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                shadow: Shadow {
                    color: with_alpha(palette.danger, 0.3),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            },

            ButtonStyle::Success => button::Style {
                background: Some(Background::Color(palette.success)),
                text_color: contrast_text_color(palette.success),
                border: Border {
                    color: palette.success,
                    width: 0.0,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                shadow: Shadow {
                    color: with_alpha(palette.success, 0.3),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            },

            ButtonStyle::Ghost => button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: palette.text,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: BORDER_RADIUS_SMALL.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },

            ButtonStyle::Nav => {
                let bg_color = Color::TRANSPARENT;
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: with_alpha(palette.text, 0.8),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: BORDER_RADIUS_SMALL.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ButtonStyle::NavActive => {
                let bg_color = with_alpha(palette.primary, ALPHA_LIGHT);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: palette.primary,
                    border: Border {
                        color: palette.primary,
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_SMALL.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ButtonStyle::Icon => {
                let bg_color = Color::TRANSPARENT;
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: palette.text,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: BORDER_RADIUS_ROUND.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }

            ButtonStyle::Pill => {
                let bg_color = with_alpha(palette.primary, ALPHA_SUBTLE);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: palette.primary,
                    border: Border {
                        color: with_alpha(palette.primary, ALPHA_MEDIUM),
                        width: BORDER_WIDTH,
                        radius: BORDER_RADIUS_ROUND.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }
        }
    }

    fn hovered(&self, theme: &AppTheme) -> button::Style {
        let palette = theme.palette();
        let is_dark = is_dark_theme(theme);
        let base = self.active(theme);

        match self {
            ButtonStyle::Standard => {
                let bg_color = if is_dark {
                    lighten(palette.background, 0.2)
                } else {
                    darken(palette.background, 0.12)
                };
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow {
                        color: Color::BLACK,
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                    ..base
                }
            }

            ButtonStyle::Primary => {
                let bg_color = lighten(palette.primary, 0.1);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow {
                        color: with_alpha(palette.primary, 0.4),
                        offset: Vector::new(0.0, 3.0),
                        blur_radius: 6.0,
                    },
                    ..base
                }
            }

            ButtonStyle::Secondary => {
                let bg_color = with_alpha(palette.primary, ALPHA_LIGHT);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    ..base
                }
            }

            ButtonStyle::Danger => {
                let bg_color = lighten(palette.danger, 0.1);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow {
                        color: with_alpha(palette.danger, 0.4),
                        offset: Vector::new(0.0, 3.0),
                        blur_radius: 6.0,
                    },
                    ..base
                }
            }

            ButtonStyle::Success => {
                let bg_color = lighten(palette.success, 0.1);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow {
                        color: with_alpha(palette.success, 0.4),
                        offset: Vector::new(0.0, 3.0),
                        blur_radius: 6.0,
                    },
                    ..base
                }
            }

            ButtonStyle::Ghost => {
                let bg_color = with_alpha(palette.text, ALPHA_SUBTLE);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    ..base
                }
            }

            ButtonStyle::Nav => {
                let bg_color = with_alpha(palette.text, ALPHA_SUBTLE);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: palette.text,
                    ..base
                }
            }

            ButtonStyle::NavActive => base,

            ButtonStyle::Icon => {
                let bg_color = with_alpha(palette.text, ALPHA_SUBTLE);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    ..base
                }
            }

            ButtonStyle::Pill => {
                let bg_color = with_alpha(palette.primary, ALPHA_LIGHT);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    ..base
                }
            }
        }
    }

    fn pressed(&self, theme: &AppTheme) -> button::Style {
        let palette = theme.palette();
        let base = self.active(theme);

        match self {
            ButtonStyle::Primary => {
                let bg_color = darken(palette.primary, 0.1);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow::default(),
                    ..base
                }
            }
            ButtonStyle::Danger => {
                let bg_color = darken(palette.danger, 0.1);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow::default(),
                    ..base
                }
            }
            ButtonStyle::Success => {
                let bg_color = darken(palette.success, 0.1);
                button::Style {
                    background: Some(Background::Color(bg_color)),
                    shadow: Shadow::default(),
                    ..base
                }
            }
            _ => self.hovered(theme),
        }
    }

    fn disabled(&self, theme: &AppTheme) -> button::Style {
        let palette = theme.palette();
        let base = self.active(theme);

        button::Style {
            background: Some(Background::Color(with_alpha(palette.text, ALPHA_SUBTLE))),
            text_color: with_alpha(palette.text, ALPHA_MEDIUM),
            border: Border {
                color: Color::TRANSPARENT,
                ..base.border
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

impl Catalog for AppTheme {
    type Class<'a> = ButtonStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> button::Style {
        class.get_style(self, status)
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

fn contrast_text_color(bg: Color) -> Color {
    let luminance = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    if luminance > 0.5 { Color::BLACK } else { Color::WHITE }
}
