use std::fmt::Display;

use chrono::{DateTime, Duration, Local};

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
    pub fn seconds(&self) -> i64 {
        match self {
            TimeRange::LastMinute => 60,
            TimeRange::LastHour => 3_600,
            TimeRange::Last24Hours => 86_400,
            TimeRange::LastWeek => 604_800,
            TimeRange::LastMonth => 2_592_000,
            TimeRange::LastYear => 31_536_000,
        }
    }

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

    pub fn granularity_seconds(&self) -> i64 {
        match self {
            TimeRange::LastMinute => 1,
            TimeRange::LastHour => 60,
            TimeRange::Last24Hours => 3600,
            TimeRange::LastWeek => 3600,
            TimeRange::LastMonth => 21600,
            TimeRange::LastYear => 86400,
        }
    }

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

    pub fn duration_seconds(&self) -> Duration {
        Duration::seconds(self.seconds())
    }

    pub fn start_time(&self) -> DateTime<Local> {
        Local::now() - self.duration_seconds()
    }

    pub fn end_time(&self) -> DateTime<Local> {
        Local::now()
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppLanguage {
    #[default]
    English,
    French,
}

impl AppLanguage {
    pub const fn all() -> &'static [AppLanguage] {
        &[AppLanguage::English, AppLanguage::French]
    }

    pub fn code(self) -> &'static str {
        match self {
            AppLanguage::English => "EN",
            AppLanguage::French => "FR",
        }
    }

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
    pub const PRESETS: &[CarbonIntensity] = &[
        CarbonIntensity {
            label: "France",
            g_per_kwh: 56.0,
        },
        CarbonIntensity {
            label: "Germany",
            g_per_kwh: 385.0,
        },
        CarbonIntensity {
            label: "UK",
            g_per_kwh: 230.0,
        },
        CarbonIntensity {
            label: "USA (average)",
            g_per_kwh: 390.0,
        },
        CarbonIntensity {
            label: "China",
            g_per_kwh: 555.0,
        },
        CarbonIntensity {
            label: "India",
            g_per_kwh: 710.0,
        },
        CarbonIntensity {
            label: "Sweden",
            g_per_kwh: 45.0,
        },
        CarbonIntensity {
            label: "Poland",
            g_per_kwh: 635.0,
        },
        CarbonIntensity {
            label: "World average",
            g_per_kwh: 475.0,
        },
        CarbonIntensity {
            label: "Custom",
            g_per_kwh: 0.0,
        },
    ];

    pub fn is_custom(self) -> bool {
        self.label == "Custom"
    }

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
