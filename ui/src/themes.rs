use iced::{Color, Theme, theme::Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    Light,
    #[default]
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
    // Custom themes
    EcoEnergy,
    EcoEnergyLight,
    PowerSaver,
    HighContrast,
}

impl AppTheme {
    pub fn to_iced_theme(self) -> Theme {
        match self {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
            AppTheme::Dracula => Theme::Dracula,
            AppTheme::Nord => Theme::Nord,
            AppTheme::SolarizedLight => Theme::SolarizedLight,
            AppTheme::SolarizedDark => Theme::SolarizedDark,
            AppTheme::GruvboxLight => Theme::GruvboxLight,
            AppTheme::GruvboxDark => Theme::GruvboxDark,
            AppTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            AppTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            AppTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            AppTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            AppTheme::TokyoNight => Theme::TokyoNight,
            AppTheme::TokyoNightStorm => Theme::TokyoNightStorm,
            AppTheme::TokyoNightLight => Theme::TokyoNightLight,
            AppTheme::KanagawaWave => Theme::KanagawaWave,
            AppTheme::KanagawaDragon => Theme::KanagawaDragon,
            AppTheme::KanagawaLotus => Theme::KanagawaLotus,
            AppTheme::Moonfly => Theme::Moonfly,
            AppTheme::Nightfly => Theme::Nightfly,
            AppTheme::Oxocarbon => Theme::Oxocarbon,
            AppTheme::Ferra => Theme::Ferra,
            // Custom themes
            AppTheme::EcoEnergy => Theme::custom(
                "Eco Energy".to_string(),
                Palette {
                    background: Color::from_rgb(0.12, 0.14, 0.16),
                    text: Color::from_rgb(0.9, 0.92, 0.94),
                    primary: Color::from_rgb(0.2, 0.78, 0.35),
                    success: Color::from_rgb(0.2, 0.6, 0.86),
                    danger: Color::from_rgb(0.95, 0.45, 0.25),
                },
            ),
            AppTheme::EcoEnergyLight => Theme::custom(
                "Eco Energy Light".to_string(),
                Palette {
                    background: Color::from_rgb(0.96, 0.97, 0.98),
                    text: Color::from_rgb(0.15, 0.18, 0.22),
                    primary: Color::from_rgb(0.13, 0.58, 0.26),
                    success: Color::from_rgb(0.15, 0.45, 0.65),
                    danger: Color::from_rgb(0.85, 0.35, 0.15),
                },
            ),
            AppTheme::PowerSaver => Theme::custom(
                "Power Saver".to_string(),
                Palette {
                    background: Color::BLACK,
                    text: Color::from_rgb(0.6, 0.62, 0.64),
                    primary: Color::from_rgb(0.15, 0.5, 0.25),
                    success: Color::from_rgb(0.15, 0.4, 0.55),
                    danger: Color::from_rgb(0.6, 0.3, 0.15),
                },
            ),
            AppTheme::HighContrast => Theme::custom(
                "High Contrast".to_string(),
                Palette {
                    background: Color::BLACK,
                    text: Color::WHITE,
                    primary: Color::from_rgb(0.0, 1.0, 0.0),
                    success: Color::from_rgb(0.0, 0.8, 1.0),
                    danger: Color::from_rgb(1.0, 0.4, 0.0),
                },
            ),
        }
    }

    pub fn palette(self) -> Palette {
        self.to_iced_theme().palette()
    }

    pub fn name(self) -> &'static str {
        match self {
            AppTheme::Light => "Light",
            AppTheme::Dark => "Dark",
            AppTheme::Dracula => "Dracula",
            AppTheme::Nord => "Nord",
            AppTheme::SolarizedLight => "Solarized Light",
            AppTheme::SolarizedDark => "Solarized Dark",
            AppTheme::GruvboxLight => "Gruvbox Light",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::CatppuccinLatte => "Catppuccin Latte",
            AppTheme::CatppuccinFrappe => "Catppuccin Frappé",
            AppTheme::CatppuccinMacchiato => "Catppuccin Macchiato",
            AppTheme::CatppuccinMocha => "Catppuccin Mocha",
            AppTheme::TokyoNight => "Tokyo Night",
            AppTheme::TokyoNightStorm => "Tokyo Night Storm",
            AppTheme::TokyoNightLight => "Tokyo Night Light",
            AppTheme::KanagawaWave => "Kanagawa Wave",
            AppTheme::KanagawaDragon => "Kanagawa Dragon",
            AppTheme::KanagawaLotus => "Kanagawa Lotus",
            AppTheme::Moonfly => "Moonfly",
            AppTheme::Nightfly => "Nightfly",
            AppTheme::Oxocarbon => "Oxocarbon",
            AppTheme::Ferra => "Ferra",
            // Custom themes
            AppTheme::EcoEnergy => "Eco Energy",
            AppTheme::EcoEnergyLight => "Eco Energy Light",
            AppTheme::PowerSaver => "Power Saver",
            AppTheme::HighContrast => "High Contrast",
        }
    }

    pub const fn all() -> &'static [AppTheme] {
        &[
            AppTheme::Light,
            AppTheme::Dark,
            AppTheme::Dracula,
            AppTheme::Nord,
            AppTheme::SolarizedLight,
            AppTheme::SolarizedDark,
            AppTheme::GruvboxLight,
            AppTheme::GruvboxDark,
            AppTheme::CatppuccinLatte,
            AppTheme::CatppuccinFrappe,
            AppTheme::CatppuccinMacchiato,
            AppTheme::CatppuccinMocha,
            AppTheme::TokyoNight,
            AppTheme::TokyoNightStorm,
            AppTheme::TokyoNightLight,
            AppTheme::KanagawaWave,
            AppTheme::KanagawaDragon,
            AppTheme::KanagawaLotus,
            AppTheme::Moonfly,
            AppTheme::Nightfly,
            AppTheme::Oxocarbon,
            AppTheme::Ferra,
            // Custom themes
            AppTheme::EcoEnergy,
            AppTheme::EcoEnergyLight,
            AppTheme::PowerSaver,
            AppTheme::HighContrast,
        ]
    }

    pub const fn custom_themes() -> &'static [AppTheme] {
        &[
            AppTheme::EcoEnergy,
            AppTheme::EcoEnergyLight,
            AppTheme::PowerSaver,
            AppTheme::HighContrast,
        ]
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
