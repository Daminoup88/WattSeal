use iced::{Color, Theme, theme::Palette};

macro_rules! define_themes {
    (
        builtin: [$($builtin:ident => $iced:ident),+ $(,)?]
        custom: [$($custom:ident => $name:literal: $palette:expr),+ $(,)?]
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum AppTheme {
            #[default]
            $($builtin,)+
            $($custom,)+
        }

        impl AppTheme {
            pub fn to_iced_theme(self) -> Theme {
                match self {
                    $(AppTheme::$builtin => Theme::$iced,)+
                    $(AppTheme::$custom => Theme::custom(String::from($name), $palette),)+
                }
            }

            pub fn name(self) -> &'static str {
                match self {
                    $(AppTheme::$builtin => stringify!($builtin),)+
                    $(AppTheme::$custom => $name,)+
                }
            }

            pub const fn all() -> &'static [AppTheme] {
                &[$(AppTheme::$builtin,)+ $(AppTheme::$custom,)+]
            }

            pub const fn custom_themes() -> &'static [AppTheme] {
                &[$(AppTheme::$custom,)+]
            }
        }
    };
}

define_themes! {
    builtin: [
        Light => Light,
        Dark => Dark,
        Dracula => Dracula,
        Nord => Nord,
        SolarizedLight => SolarizedLight,
        SolarizedDark => SolarizedDark,
        GruvboxLight => GruvboxLight,
        GruvboxDark => GruvboxDark,
        CatppuccinLatte => CatppuccinLatte,
        CatppuccinFrappe => CatppuccinFrappe,
        CatppuccinMacchiato => CatppuccinMacchiato,
        CatppuccinMocha => CatppuccinMocha,
        TokyoNight => TokyoNight,
        TokyoNightStorm => TokyoNightStorm,
        TokyoNightLight => TokyoNightLight,
        KanagawaWave => KanagawaWave,
        KanagawaDragon => KanagawaDragon,
        KanagawaLotus => KanagawaLotus,
        Moonfly => Moonfly,
        Nightfly => Nightfly,
        Oxocarbon => Oxocarbon,
        Ferra => Ferra,
    ]
    custom: [
        EcoEnergy => "Eco Energy": Palette {
            background: Color::from_rgb(0.12, 0.14, 0.16),
            text: Color::from_rgb(0.9, 0.92, 0.94),
            primary: Color::from_rgb(0.2, 0.78, 0.35),
            success: Color::from_rgb(0.2, 0.6, 0.86),
            danger: Color::from_rgb(0.95, 0.45, 0.25),
            warning: Color::from_rgb(0.95, 0.75, 0.25),
        },
        EcoEnergyLight => "Eco Energy Light": Palette {
            background: Color::from_rgb(0.96, 0.97, 0.98),
            text: Color::from_rgb(0.15, 0.18, 0.22),
            primary: Color::from_rgb(0.13, 0.58, 0.26),
            success: Color::from_rgb(0.15, 0.45, 0.65),
            danger: Color::from_rgb(0.85, 0.35, 0.15),
            warning: Color::from_rgb(0.85, 0.65, 0.15),
        },
        PowerSaver => "Power Saver": Palette {
            background: Color::BLACK,
            text: Color::from_rgb(0.6, 0.62, 0.64),
            primary: Color::from_rgb(0.15, 0.5, 0.25),
            success: Color::from_rgb(0.15, 0.4, 0.55),
            danger: Color::from_rgb(0.6, 0.3, 0.15),
            warning: Color::from_rgb(0.6, 0.5, 0.15),
        },
        HighContrast => "High Contrast": Palette {
            background: Color::BLACK,
            text: Color::WHITE,
            primary: Color::from_rgb(0.0, 1.0, 0.0),
            success: Color::from_rgb(0.0, 0.8, 1.0),
            danger: Color::from_rgb(1.0, 0.4, 0.0),
            warning: Color::from_rgb(1.0, 0.9, 0.0),
        },
    ]
}

impl AppTheme {
    pub fn palette(self) -> Palette {
        self.to_iced_theme().palette()
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
