use iced::{
    Color, Theme,
    theme::{Base, Mode, Palette, Style},
};

macro_rules! define_themes {
    (
        // builtin: [$($builtin:ident => $iced:ident),+ $(,)?]
        custom: [$($custom:ident => $name:literal: $palette:expr),+ $(,)?]
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum AppTheme {
            #[default]
            $($custom,)+
            // $($builtin,)+
        }
        impl AppTheme {
            pub fn to_iced_theme(self) -> Theme {
                match self {
                    // $(AppTheme::$builtin => Theme::$iced,)+
                    $(AppTheme::$custom => Theme::custom(String::from($name), $palette),)+
                }
            }
            pub fn name(self) -> &'static str {
                match self {
                    // $(AppTheme::$builtin => stringify!($builtin),)+
                    $(AppTheme::$custom => $name,)+
                }
            }
            pub const fn all() -> &'static [AppTheme] {
                &[/*$(AppTheme::$builtin,)+*/ $(AppTheme::$custom,)+]
            }
            pub const fn custom_themes() -> &'static [AppTheme] {
                &[$(AppTheme::$custom,)+]
            }
        }
    };
}

define_themes! {
    // builtin: [
    //     Dracula => Dracula,
    //     CatppuccinLatte => CatppuccinLatte,
    //     Light => Light,
    //     Dark => Dark,
    //     Nord => Nord,
    //     SolarizedLight => SolarizedLight,
    //     SolarizedDark => SolarizedDark,
    //     GruvboxLight => GruvboxLight,
    //     GruvboxDark => GruvboxDark,
    //     CatppuccinFrappe => CatppuccinFrappe,
    //     CatppuccinMacchiato => CatppuccinMacchiato,
    //     CatppuccinMocha => CatppuccinMocha,
    //     TokyoNight => TokyoNight,
    //     TokyoNightStorm => TokyoNightStorm,
    //     TokyoNightLight => TokyoNightLight,
    //     KanagawaWave => KanagawaWave,
    //     KanagawaDragon => KanagawaDragon,
    //     KanagawaLotus => KanagawaLotus,
    //     Moonfly => Moonfly,
    //     Nightfly => Nightfly,
    //     Oxocarbon => Oxocarbon,
    //     Ferra => Ferra,
    // ]
    custom: [
        DeepOcean => "Hunting": Palette {
            background: Color::from_rgb(0.05, 0.07, 0.13),      // #0d1221
            text: Color::from_rgb(0.9, 0.95, 1.0),              // #e6f2ff
            primary: Color::from_rgb(0.0, 0.80, 0.9),           // #00cce6
            success: Color::from_rgb(0.55, 1.0, 0.40),          // #8cff66
            danger: Color::from_rgb(0.9, 0.7, 0.0),             // #e6b300
            warning: Color::from_rgb(0.85, 1.0, 0.3),           // #d9ff4d
        },
        OceanLight => "Swimming": Palette {
            background: Color::from_rgb(0.94, 0.97, 1.0),      // #f0f7ff
            text: Color::from_rgb(0.08, 0.12, 0.20),           // #141f33
            primary: Color::from_rgb(0.0, 0.65, 0.78),         // #00a6c7
            success: Color::from_rgb(0.35, 0.75, 0.30),        // #59bf4d
            danger: Color::from_rgb(0.85, 0.55, 0.0),          // #d98c00
            warning: Color::from_rgb(0.78, 0.85, 0.25),        // #c7d940
        },
        EcoEnergy => "Sleeping": Palette {
            background: Color::from_rgb(0.12, 0.14, 0.16),
            text: Color::from_rgb(0.9, 0.92, 0.94),
            primary: Color::from_rgb(0.2, 0.78, 0.35),
            success: Color::from_rgb(0.2, 0.6, 0.86),
            danger: Color::from_rgb(0.95, 0.45, 0.25),
            warning: Color::from_rgb(0.95, 0.75, 0.25),
        },
        EcoEnergyLight => "Splashing": Palette {
            background: Color::from_rgb(0.96, 0.97, 0.98),
            text: Color::from_rgb(0.15, 0.18, 0.22),
            primary: Color::from_rgb(0.13, 0.58, 0.26),
            success: Color::from_rgb(0.15, 0.45, 0.65),
            danger: Color::from_rgb(0.85, 0.35, 0.15),
            warning: Color::from_rgb(0.85, 0.65, 0.15),
        },

        GeothermalCore => "Sunbathing": Palette {
            background: Color::from_rgb(0.10, 0.09, 0.09),
            text: Color::from_rgb(0.95, 0.93, 0.91),
            primary: Color::from_rgb(1.0, 0.40, 0.10),
            success: Color::from_rgb(0.30, 0.69, 0.31),
            danger: Color::from_rgb(0.90, 0.15, 0.15),
            warning: Color::from_rgb(1.0, 0.75, 0.05),
        },
        SolarUmbra => "Lounging": Palette {
            background: Color::from_rgb(0.08, 0.07, 0.06),
            text: Color::from_rgb(1.0, 0.99, 0.97),
            primary: Color::from_rgb(1.0, 0.84, 0.31),
            success: Color::from_rgb(0.30, 0.71, 0.67),
            danger: Color::from_rgb(0.90, 0.22, 0.21),
            warning: Color::from_rgb(1.0, 0.60, 0.0),
        },
    ]
}

impl Base for AppTheme {
    fn default(preference: Mode) -> Self {
        match preference {
            Mode::Light => AppTheme::OceanLight,
            Mode::Dark | Mode::None => AppTheme::DeepOcean,
        }
    }

    fn mode(&self) -> Mode {
        let pal = AppTheme::palette(*self);
        let luminance = 0.299 * pal.background.r + 0.587 * pal.background.g + 0.114 * pal.background.b;
        if luminance < 0.5 { Mode::Dark } else { Mode::Light }
    }

    fn base(&self) -> Style {
        let pal = AppTheme::palette(*self);
        Style {
            background_color: pal.background,
            text_color: pal.text,
        }
    }

    fn palette(&self) -> Option<Palette> {
        Some(self.to_iced_theme().palette())
    }

    fn name(&self) -> &str {
        AppTheme::name(*self)
    }
}

impl AppTheme {
    /// Resolves a theme by its display name, falling back to the default.
    pub fn from_name(name: &str) -> Self {
        Self::all()
            .iter()
            .copied()
            .find(|t| t.name() == name)
            .unwrap_or_default()
    }

    pub fn palette(self) -> Palette {
        self.to_iced_theme().palette()
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
