use std::fmt::Display;

use chrono::{DateTime, Duration, Local};

/// Selectable time window for chart data display.
#[derive(Default, Clone, PartialEq, Debug)]
pub enum TimeRange {
    #[default]
    LastMinute,
    LastHour,
    Last24Hours,
    LastWeek,
    LastMonth,
    LastYear,
}

impl TimeRange {
    /// Returns the total duration in seconds.
    pub fn seconds(&self) -> i64 {
        match self {
            TimeRange::LastMinute => 60,
            TimeRange::LastHour => 3_600,
            TimeRange::Last24Hours => 86_400,
            TimeRange::LastWeek => 604_800,    // 7 days
            TimeRange::LastMonth => 2_592_000, // 30 days
            TimeRange::LastYear => 31_536_000, // 365 days
        }
    }

    /// Returns the axis label unit for this range.
    pub fn unit(&self) -> &'static str {
        match self {
            TimeRange::LastMinute => "s",
            TimeRange::LastHour => "min",
            TimeRange::Last24Hours => "h",
            TimeRange::LastWeek => "h",
            TimeRange::LastMonth => "d",
            TimeRange::LastYear => "d",
        }
    }

    /// Returns the data aggregation window in seconds.
    pub fn granularity_seconds(&self) -> i64 {
        match self {
            TimeRange::LastMinute => 1,
            TimeRange::LastHour => 60,
            TimeRange::Last24Hours => 3_600, // 1 hour
            TimeRange::LastWeek => 3_600,    // 1 hour
            TimeRange::LastMonth => 86_400,  // 1 day
            TimeRange::LastYear => 604_800,  // 1 week
        }
    }

    /// Returns true for the real-time (1 Hz) range.
    pub fn is_real_time(&self) -> bool {
        matches!(self, TimeRange::LastMinute)
    }

    /// Returns true when the granularity is >= 1 hour,
    /// meaning we display energy (Wh) instead of average power (W).
    pub fn is_energy_mode(&self) -> bool {
        self.granularity_seconds() >= 3600
    }

    /// Returns the power/energy unit string for the current mode.
    pub fn power_unit(&self) -> &'static str {
        if self.is_energy_mode() { "Wh" } else { "W" }
    }

    /// Conversion factor from average watts to the display unit.
    /// For energy mode: avg_watts * window_hours = Wh.
    /// For power mode: factor is 1 (already watts).
    pub fn power_scale_factor(&self) -> f64 {
        if self.is_energy_mode() {
            self.granularity_seconds() as f64 / 3600.0
        } else {
            1.0
        }
    }

    /// Converts to a chrono Duration.
    pub fn duration_seconds(&self) -> Duration {
        Duration::seconds(self.seconds())
    }

    /// Returns the start of this range relative to now.
    pub fn start_time(&self) -> DateTime<Local> {
        Local::now() - self.duration_seconds()
    }

    /// Returns the current local time as end boundary.
    pub fn end_time(&self) -> DateTime<Local> {
        Local::now()
    }

    /// Returns all available ranges for total power charts.
    pub fn all_total() -> &'static [TimeRange] {
        &[
            TimeRange::LastMinute,
            TimeRange::LastHour,
            TimeRange::Last24Hours,
            TimeRange::LastWeek,
            TimeRange::LastMonth,
            TimeRange::LastYear,
        ]
    }

    /// Returns available ranges for per-component charts.
    pub fn all_component() -> &'static [TimeRange] {
        &[TimeRange::LastMinute, TimeRange::LastHour, TimeRange::Last24Hours]
    }
}

impl Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRange::LastMinute => write!(f, "Last Minute"),
            TimeRange::LastHour => write!(f, "Last Hour"),
            TimeRange::Last24Hours => write!(f, "Last 24 Hours"),
            TimeRange::LastWeek => write!(f, "Last Week"),
            TimeRange::LastMonth => write!(f, "Last Month"),
            TimeRange::LastYear => write!(f, "Last Year"),
        }
    }
}

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppLanguage {
    #[default]
    English,
    French,
}

impl AppLanguage {
    /// Returns all available languages.
    pub const fn all() -> &'static [AppLanguage] {
        &[AppLanguage::English, AppLanguage::French]
    }

    /// Returns the ISO language code.
    pub fn code(self) -> &'static str {
        match self {
            AppLanguage::English => "EN",
            AppLanguage::French => "FR",
        }
    }

    /// Parses a language from its ISO code.
    pub fn from_code(code: &str) -> Self {
        match code {
            "EN" => AppLanguage::English,
            "FR" => AppLanguage::French,
            _ => AppLanguage::English,
        }
    }
}

impl Display for AppLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppLanguage::English => write!(f, "English"),
            AppLanguage::French => write!(f, "Français"),
        }
    }
}

/// Carbon intensity preset for common countries / mixes.
#[derive(Debug, Clone, Copy)]
pub struct CarbonIntensity {
    pub label: &'static str,
    pub g_per_kwh: f64,
}

impl CarbonIntensity {
    /// Carbon intensity presets for various countries and the world average (updated in 2026).
    // Source:
    // Our World in Data, “Carbon intensity of electricity,” Our World in Data, 2022. https://ourworldindata.org/grapher/carbon-intensity-electricity
    // World average:
    // Emissions – Electricity 2025 – Analysis - IEA, “Emissions – Electricity 2025 – Analysis - IEA,” IEA, 2025. https://www.iea.org/reports/electricity-2025/emissions
    pub const PRESETS: &[CarbonIntensity] = &[
        CarbonIntensity {
            label: "France",
            g_per_kwh: 42.0,
        },
        CarbonIntensity {
            label: "Germany",
            g_per_kwh: 332.0,
        },
        CarbonIntensity {
            label: "UK",
            g_per_kwh: 217.0,
        },
        CarbonIntensity {
            label: "USA (average)",
            g_per_kwh: 384.0,
        },
        CarbonIntensity {
            label: "China",
            g_per_kwh: 555.0,
        },
        CarbonIntensity {
            label: "India",
            g_per_kwh: 707.0,
        },
        CarbonIntensity {
            label: "Sweden",
            g_per_kwh: 35.0,
        },
        CarbonIntensity {
            label: "Poland",
            g_per_kwh: 592.0,
        },
        CarbonIntensity {
            label: "World average",
            g_per_kwh: 399.0,
        },
        CarbonIntensity {
            label: "Custom",
            g_per_kwh: 0.0,
        },
    ];

    /// Returns true if this is a user-defined value.
    pub fn is_custom(self) -> bool {
        self.label == "Custom"
    }

    /// Finds the matching preset or creates a custom entry.
    pub fn from_g_per_kwh(value: f64) -> Self {
        Self::PRESETS
            .iter()
            .find(|p| (p.g_per_kwh - value).abs() < 0.5)
            .copied()
            .unwrap_or(CarbonIntensity {
                label: "Custom",
                g_per_kwh: value,
            })
    }
}

impl PartialEq for CarbonIntensity {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Display for CarbonIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:.0} g/kWh)", self.label, self.g_per_kwh)
    }
}

/// Electricity cost preset for common countries / regions.
#[derive(Debug, Clone, Copy)]
pub struct ElectricityCost {
    pub label: &'static str,
    /// Price in $/kWh.
    pub usd_per_kwh: f64,
}

impl ElectricityCost {
    /// Electricity cost presets for various countries and the world average (updated in 2026).
    // Source:
    // Statista, “Electricity prices around the world 2018 | Statista,” Statista, 2018. https://www.statista.com/statistics/263492/electricity-prices-in-selected-countries/
    // World average:
    // Global Petrol Prices, “Electricity prices around the world, March 2019 | GlobalPetrolPrices.com,” GlobalPetrpPrices.com, 2023. https://www.globalpetrolprices.com/electricity_prices/
    pub const PRESETS: &[ElectricityCost] = &[
        ElectricityCost {
            label: "France",
            usd_per_kwh: 0.28,
        },
        ElectricityCost {
            label: "Germany",
            usd_per_kwh: 0.4,
        },
        ElectricityCost {
            label: "Spain",
            usd_per_kwh: 0.25,
        },
        ElectricityCost {
            label: "Italy",
            usd_per_kwh: 0.42,
        },
        ElectricityCost {
            label: "Netherlands",
            usd_per_kwh: 0.28,
        },
        ElectricityCost {
            label: "Switzerland",
            usd_per_kwh: 0.37,
        },
        ElectricityCost {
            label: "UK",
            usd_per_kwh: 0.4,
        },
        ElectricityCost {
            label: "USA (average)",
            usd_per_kwh: 0.18,
        },
        ElectricityCost {
            label: "World average",
            usd_per_kwh: 0.17,
        },
        ElectricityCost {
            label: "Custom",
            usd_per_kwh: 0.0,
        },
    ];

    pub fn is_custom(self) -> bool {
        self.label == "Custom"
    }

    /// Finds the matching preset or creates a custom entry.
    pub fn from_usd_per_kwh(value: f64) -> Self {
        Self::PRESETS
            .iter()
            .find(|p| !p.is_custom() && (p.usd_per_kwh - value).abs() < 0.001)
            .copied()
            .unwrap_or(ElectricityCost {
                label: "Custom",
                usd_per_kwh: value,
            })
    }
}

impl PartialEq for ElectricityCost {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Display for ElectricityCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_custom() {
            write!(f, "Custom")
        } else {
            write!(f, "{} ({:.2} $/kWh)", self.label, self.usd_per_kwh)
        }
    }
}
