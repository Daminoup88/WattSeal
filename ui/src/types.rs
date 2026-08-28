use std::fmt::Display;

use chrono::{DateTime, Duration, Local};
use common::{ComputedSensorData, MAX_TRACKED_PROCESSES, SECONDS_PER_HOUR};

const DEFAULT_CUSTOM_PROCESS_LIMIT: usize = 15;

/// Preset shown in the process-count selector.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessLimit {
    Five,
    #[default]
    Ten,
    Custom,
}

impl ProcessLimit {
    pub const fn all() -> &'static [ProcessLimit] {
        &[ProcessLimit::Five, ProcessLimit::Ten, ProcessLimit::Custom]
    }

    pub fn resolve(self, custom: Option<usize>) -> usize {
        match self {
            ProcessLimit::Five => 5,
            ProcessLimit::Ten => 10,
            ProcessLimit::Custom => custom
                .unwrap_or(DEFAULT_CUSTOM_PROCESS_LIMIT)
                .clamp(1, MAX_TRACKED_PROCESSES),
        }
    }
}

/// A sensor data point with the sampling duration.
#[derive(Debug, Clone)]
pub struct SensorRecord {
    pub timestamp: DateTime<Local>,
    pub duration_ms: i64,
    pub data: ComputedSensorData,
}

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

    /// Returns true outside the live range, where charts display energy (Wh)
    /// instead of instantaneous power (W).
    pub fn is_energy_mode(&self) -> bool {
        !self.is_real_time()
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
            self.granularity_seconds() as f64 / SECONDS_PER_HOUR
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
    Chinese,
    Romanian,
}

impl AppLanguage {
    /// Returns all available languages.
    pub const fn all() -> &'static [AppLanguage] {
        &[
            AppLanguage::English,
            AppLanguage::French,
            AppLanguage::Chinese,
            AppLanguage::Romanian,
        ]
    }

    /// Returns the ISO language code.
    pub fn code(self) -> &'static str {
        match self {
            AppLanguage::English => "EN",
            AppLanguage::French => "FR",
            AppLanguage::Chinese => "ZH",
            AppLanguage::Romanian => "RO",
        }
    }

    /// Parses a language from its ISO code.
    pub fn from_code(code: &str) -> Self {
        match code {
            "EN" => AppLanguage::English,
            "FR" => AppLanguage::French,
            "ZH" => AppLanguage::Chinese,
            "RO" => AppLanguage::Romanian,
            _ => AppLanguage::English,
        }
    }
}

impl Display for AppLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppLanguage::English => write!(f, "English"),
            AppLanguage::French => write!(f, "Français"),
            AppLanguage::Chinese => write!(f, "简体中文"),
            AppLanguage::Romanian => write!(f, "Română"),
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

    /// Resolves a stored string to a preset entry.
    pub fn from_label(label: &str) -> Self {
        if let Some(preset) = Self::PRESETS.iter().find(|p| !p.is_custom() && p.label == label) {
            return *preset;
        }
        if let Ok(value) = label.trim().parse::<f64>() {
            return CarbonIntensity {
                label: "Custom",
                g_per_kwh: value,
            };
        }
        *Self::PRESETS.iter().find(|p| p.label == "World average").unwrap()
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

/// Supported currencies for energy cost calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Currency {
    pub code: &'static str,
    pub symbol: &'static str,
}

impl Currency {
    pub const USD: Currency = Currency {
        code: "USD",
        symbol: "$",
    };
    pub const EUR: Currency = Currency {
        code: "EUR",
        symbol: "€",
    };
    pub const GBP: Currency = Currency {
        code: "GBP",
        symbol: "£",
    };
    pub const CHF: Currency = Currency {
        code: "CHF",
        symbol: "CHF",
    };
    pub const CAD: Currency = Currency {
        code: "CAD",
        symbol: "CA$",
    };
    pub const AUD: Currency = Currency {
        code: "AUD",
        symbol: "A$",
    };
    pub const JPY: Currency = Currency {
        code: "JPY",
        symbol: "¥",
    };
    pub const CNY: Currency = Currency {
        code: "CNY",
        symbol: "¥",
    };
    pub const INR: Currency = Currency {
        code: "INR",
        symbol: "₹",
    };
    pub const BRL: Currency = Currency {
        code: "BRL",
        symbol: "R$",
    };
    pub const RUB: Currency = Currency {
        code: "RUB",
        symbol: "₽",
    };
    pub const KRW: Currency = Currency {
        code: "KRW",
        symbol: "₩",
    };
    pub const MXN: Currency = Currency {
        code: "MXN",
        symbol: "MX$",
    };
    pub const SGD: Currency = Currency {
        code: "SGD",
        symbol: "S$",
    };
    pub const HKD: Currency = Currency {
        code: "HKD",
        symbol: "HK$",
    };
    pub const SEK: Currency = Currency {
        code: "SEK",
        symbol: "kr",
    };
    pub const NOK: Currency = Currency {
        code: "NOK",
        symbol: "kr",
    };
    pub const DKK: Currency = Currency {
        code: "DKK",
        symbol: "kr",
    };
    pub const PLN: Currency = Currency {
        code: "PLN",
        symbol: "zł",
    };
    pub const TRY: Currency = Currency {
        code: "TRY",
        symbol: "₺",
    };
    pub const ZAR: Currency = Currency {
        code: "ZAR",
        symbol: "R",
    };
    pub const PHP: Currency = Currency {
        code: "PHP",
        symbol: "₱",
    };
    pub const IDR: Currency = Currency {
        code: "IDR",
        symbol: "Rp",
    };
    pub const THB: Currency = Currency {
        code: "THB",
        symbol: "฿",
    };
    pub const MYR: Currency = Currency {
        code: "MYR",
        symbol: "RM",
    };
    pub const VND: Currency = Currency {
        code: "VND",
        symbol: "₫",
    };
    pub const ILS: Currency = Currency {
        code: "ILS",
        symbol: "₪",
    };
    pub const AED: Currency = Currency {
        code: "AED",
        symbol: "AED",
    };
    pub const SAR: Currency = Currency {
        code: "SAR",
        symbol: "SAR",
    };
    pub const NZD: Currency = Currency {
        code: "NZD",
        symbol: "NZ$",
    };
    pub const CZK: Currency = Currency {
        code: "CZK",
        symbol: "Kč",
    };
    pub const HUF: Currency = Currency {
        code: "HUF",
        symbol: "Ft",
    };
    pub const RON: Currency = Currency {
        code: "RON",
        symbol: "lei",
    };
    pub const BGN: Currency = Currency {
        code: "BGN",
        symbol: "лв",
    };
    pub const ARS: Currency = Currency {
        code: "ARS",
        symbol: "AR$",
    };
    pub const CLP: Currency = Currency {
        code: "CLP",
        symbol: "CLP$",
    };
    pub const COP: Currency = Currency {
        code: "COP",
        symbol: "COL$",
    };
    pub const EGP: Currency = Currency {
        code: "EGP",
        symbol: "E£",
    };
    pub const NGN: Currency = Currency {
        code: "NGN",
        symbol: "₦",
    };
    pub const PKR: Currency = Currency {
        code: "PKR",
        symbol: "Rs",
    };
    pub const BTC: Currency = Currency {
        code: "BTC",
        symbol: "₿",
    };

    pub const ALL: &'static [Currency] = &[
        Currency::USD,
        Currency::EUR,
        Currency::GBP,
        Currency::CHF,
        Currency::CAD,
        Currency::AUD,
        Currency::JPY,
        Currency::CNY,
        Currency::INR,
        Currency::BRL,
        Currency::RUB,
        Currency::KRW,
        Currency::MXN,
        Currency::SGD,
        Currency::HKD,
        Currency::SEK,
        Currency::NOK,
        Currency::DKK,
        Currency::PLN,
        Currency::TRY,
        Currency::ZAR,
        Currency::PHP,
        Currency::IDR,
        Currency::THB,
        Currency::MYR,
        Currency::VND,
        Currency::ILS,
        Currency::AED,
        Currency::SAR,
        Currency::NZD,
        Currency::CZK,
        Currency::HUF,
        Currency::RON,
        Currency::BGN,
        Currency::ARS,
        Currency::CLP,
        Currency::COP,
        Currency::EGP,
        Currency::NGN,
        Currency::PKR,
        Currency::BTC,
    ];

    pub fn from_code(code: &str) -> Self {
        Self::ALL
            .iter()
            .find(|c| c.code.eq_ignore_ascii_case(code))
            .copied()
            .unwrap_or(Currency::USD)
    }
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.code, self.symbol)
    }
}

/// Electricity cost preset for common countries / regions.
#[derive(Debug, Clone, Copy)]
pub struct ElectricityCost {
    pub label: &'static str,
    /// Price per kWh in local currency.
    pub price_per_kwh: f64,
    /// Display currency symbol (e.g. "€", "$").
    pub currency_symbol: &'static str,
    /// ISO 4217 Currency code (e.g. "EUR", "USD").
    pub currency_code: &'static str,
}

impl ElectricityCost {
    /// Household electricity prices and currencies by country/region (prices and change rates updated in 2026).
    // Source:
    // Statista, “Electricity prices around the world 2018 | Statista,” Statista, 2018. https://www.statista.com/statistics/263492/electricity-prices-in-selected-countries/
    // World average:
    // Global Petrol Prices, "Household electricity prices around the world," GlobalPetrolPrices.com.
    // Source URL: https://www.globalpetrolprices.com/electricity_prices/
    pub const PRESETS: &[ElectricityCost] = &[
        ElectricityCost {
            label: "France",
            price_per_kwh: 0.24,
            currency_symbol: "€",
            currency_code: "EUR",
        },
        ElectricityCost {
            label: "China",
            price_per_kwh: 0.51,
            currency_symbol: "¥",
            currency_code: "CNY",
        },
        ElectricityCost {
            label: "India",
            price_per_kwh: 7.33,
            currency_symbol: "₹",
            currency_code: "INR",
        },
        ElectricityCost {
            label: "Indonesia",
            price_per_kwh: 1_602.0,
            currency_symbol: "Rp",
            currency_code: "IDR",
        },
        ElectricityCost {
            label: "Philippines",
            price_per_kwh: 12.69,
            currency_symbol: "₱",
            currency_code: "PHP",
        },
        ElectricityCost {
            label: "Germany",
            price_per_kwh: 0.35,
            currency_symbol: "€",
            currency_code: "EUR",
        },
        ElectricityCost {
            label: "Spain",
            price_per_kwh: 0.22,
            currency_symbol: "€",
            currency_code: "EUR",
        },
        ElectricityCost {
            label: "Italy",
            price_per_kwh: 0.36,
            currency_symbol: "€",
            currency_code: "EUR",
        },
        ElectricityCost {
            label: "Netherlands",
            price_per_kwh: 0.25,
            currency_symbol: "€",
            currency_code: "EUR",
        },
        ElectricityCost {
            label: "Switzerland",
            price_per_kwh: 0.3,
            currency_symbol: "CHF",
            currency_code: "CHF",
        },
        ElectricityCost {
            label: "UK",
            price_per_kwh: 0.3,
            currency_symbol: "£",
            currency_code: "GBP",
        },
        ElectricityCost {
            label: "USA (average)",
            price_per_kwh: 0.19,
            currency_symbol: "$",
            currency_code: "USD",
        },
        ElectricityCost {
            label: "Sweden",
            price_per_kwh: 2.3,
            currency_symbol: "kr",
            currency_code: "SEK",
        },
        ElectricityCost {
            label: "Poland",
            price_per_kwh: 0.88,
            currency_symbol: "zł",
            currency_code: "PLN",
        },
        ElectricityCost {
            label: "World average",
            price_per_kwh: 0.17,
            currency_symbol: "$",
            currency_code: "USD",
        },
        ElectricityCost {
            label: "Custom",
            price_per_kwh: 0.0,
            currency_symbol: "$",
            currency_code: "USD",
        },
    ];

    pub fn is_custom(self) -> bool {
        self.label == "Custom"
    }

    /// Returns the Currency object for this cost setting.
    pub fn currency(self) -> Currency {
        Currency::from_code(self.currency_code)
    }

    /// Finds the matching preset or creates a custom entry.
    pub fn from_price_per_kwh(value: f64) -> Self {
        Self::PRESETS
            .iter()
            .find(|p| !p.is_custom() && (p.price_per_kwh - value).abs() < 0.001)
            .copied()
            .unwrap_or(ElectricityCost {
                label: "Custom",
                price_per_kwh: value,
                currency_symbol: "$",
                currency_code: "USD",
            })
    }

    /// Resolves stored label and optional currency code to a preset entry.
    pub fn from_label_and_currency(label: &str, currency: Option<&str>) -> Self {
        if let Some(preset) = Self::PRESETS.iter().find(|p| !p.is_custom() && p.label == label) {
            return *preset;
        }
        if let Ok(value) = label.trim().parse::<f64>() {
            let curr = currency.map(Currency::from_code).unwrap_or(Currency::USD);
            return ElectricityCost {
                label: "Custom",
                price_per_kwh: value,
                currency_symbol: curr.symbol,
                currency_code: curr.code,
            };
        }
        *Self::PRESETS.iter().find(|p| p.label == "World average").unwrap()
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
            write!(
                f,
                "{} ({:.2} {}/kWh)",
                self.label, self.price_per_kwh, self.currency_symbol
            )
        }
    }
}
