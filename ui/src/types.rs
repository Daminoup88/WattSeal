use std::fmt::Display;

use chrono::{DateTime, Duration, Local};
use common::{ComputedSensorData, SECONDS_PER_HOUR};

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
            ProcessLimit::Custom => custom.unwrap_or(DEFAULT_CUSTOM_PROCESS_LIMIT).max(1),
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
    German,
    French,
    Chinese,
    Romanian,
}

impl AppLanguage {
    /// Returns all available languages.
    pub const fn all() -> &'static [AppLanguage] {
        &[
            AppLanguage::English,
            AppLanguage::German,
            AppLanguage::French,
            AppLanguage::Chinese,
            AppLanguage::Romanian,
        ]
    }

    /// Returns the ISO language code.
    pub fn code(self) -> &'static str {
        match self {
            AppLanguage::English => "EN",
            AppLanguage::German => "DE",
            AppLanguage::French => "FR",
            AppLanguage::Chinese => "ZH",
            AppLanguage::Romanian => "RO",
        }
    }

    /// Parses a language from its ISO code.
    pub fn from_code(code: &str) -> Self {
        match code {
            "EN" => AppLanguage::English,
            "DE" => AppLanguage::German,
            "FR" => AppLanguage::French,
            "ZH" => AppLanguage::Chinese,
            "RO" => AppLanguage::Romanian,
            _ => AppLanguage::English,
        }
    }

    /// Detects the OS locale and returns the best matching [`AppLanguage`] or defaults to English.
    pub fn from_os() -> Self {
        let tag = match sys_locale::get_locale() {
            Some(l) => l,
            None => return AppLanguage::default(),
        };
        // Extract the xx part from xx-XX or xx_XX (e.g. "en-US" -> "en").
        let primary = tag.split(['-', '_']).next().unwrap_or("");
        Self::from_code(&primary.to_uppercase())
    }
}

impl Display for AppLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppLanguage::English => write!(f, "English"),
            AppLanguage::German => write!(f, "Deutsch"),
            AppLanguage::French => write!(f, "Français"),
            AppLanguage::Chinese => write!(f, "简体中文"),
            AppLanguage::Romanian => write!(f, "Română"),
        }
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

macro_rules! country_registry {
    ($(($key:ident, $code:literal, $label:literal, $carbon:expr, $price:expr, $currency:expr)),+ $(,)?) => {
        /// Predefined country preset, world average, or custom user preset for compile-time exhaustive translations across all languages.
        #[repr(usize)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum CountryKey {
            $($key,)+
            Custom,
        }

        impl CountryKey {
            pub const fn code(self) -> &'static str {
                match self {
                    $(Self::$key => $code,)+
                    Self::Custom => "CUSTOM",
                }
            }

            pub fn from_code(code: &str) -> Self {
                match code {
                    $($code => Self::$key,)+
                    "CUSTOM" => Self::Custom,
                    _ => Self::World,
                }
            }
        }

        /// Centralized country preset containing country code, display label, carbon intensity, and electricity pricing.
        #[derive(Debug, Clone, Copy)]
        pub struct CountryPreset {
            pub key: CountryKey,
            pub code: &'static str,
            pub label: &'static str,
            pub carbon_intensity: Option<f64>,
            pub electricity_price: Option<f64>,
            pub currency: Option<Currency>,
        }

        impl CountryPreset {
            pub const ALL: &'static [Self] = &[
                $(Self {
                    key: CountryKey::$key,
                    code: $code,
                    label: $label,
                    carbon_intensity: $carbon,
                    electricity_price: $price,
                    currency: $currency,
                },)+
            ];

            pub const WORLD_AVERAGE: &'static Self = &Self::ALL[CountryKey::World as usize];

            pub fn by_key(key: CountryKey) -> &'static Self {
                Self::ALL.get(key as usize).unwrap_or(Self::WORLD_AVERAGE)
            }
        }
    };
}

// Centralized registry of countries and regional presets (updated in 2026).
// Sources:
// Carbon: Our World in Data, “Carbon intensity of electricity,” 2022. https://ourworldindata.org/grapher/carbon-intensity-electricity
// World avg carbon: Emissions – Electricity 2025 – Analysis - IEA, 2025. https://www.iea.org/reports/electricity-2025/emissions
// Electricity prices: Global Petrol Prices, "Household electricity prices around the world," GlobalPetrolPrices.com, 2026. https://www.globalpetrolprices.com/electricity_prices/
#[rustfmt::skip]
country_registry!(
    // CountryKey, ISO code, Display label, Carbon intensity (g/kWh), Electricity price (local currency/kWh), Currency
    (France, "FR", "France", Some(42.0), Some(0.24), Some(Currency::EUR)),
    (Germany, "DE", "Germany", Some(332.0), Some(0.35), Some(Currency::EUR)),
    (Spain, "ES", "Spain", Some(153.6), Some(0.22), Some(Currency::EUR)),
    (Italy, "IT", "Italy", Some(284.78), Some(0.36), Some(Currency::EUR)),
    (Netherlands, "NL", "Netherlands", Some(253.56), Some(0.25), Some(Currency::EUR)),
    (Switzerland, "CH", "Switzerland", Some(39.22), Some(0.3), Some(Currency::CHF)),
    (Belgium, "BE", "Belgium", Some(149.82), Some(0.36), Some(Currency::EUR)),
    (Portugal, "PT", "Portugal", Some(127.91), Some(0.21), Some(Currency::EUR)),
    (Brazil, "BR", "Brazil", Some(109.95), Some(2.37), Some(Currency::BRL)),
    (UK, "GB", "UK", Some(217.0), Some(0.3), Some(Currency::GBP)),
    (USA, "US", "USA (average)", Some(384.0), Some(0.19), Some(Currency::USD)),
    (China, "CN", "China", Some(555.0), Some(0.51), Some(Currency::CNY)),
    (India, "IN", "India", Some(707.0), Some(7.33), Some(Currency::INR)),
    (Indonesia, "ID", "Indonesia", Some(680.25), Some(1_602.0), Some(Currency::IDR)),
    (Philippines, "PH", "Philippines", Some(588.29), Some(12.69), Some(Currency::PHP)),
    (Australia, "AU", "Australia", Some(525.18), Some(0.36), Some(Currency::AUD)),
    (Sweden, "SE", "Sweden", Some(35.0), Some(2.3), Some(Currency::SEK)),
    (Poland, "PL", "Poland", Some(592.0), Some(0.88), Some(Currency::PLN)),
    (World, "WORLD", "World average", Some(399.0), Some(0.18), Some(Currency::USD)),
);

impl CountryPreset {
    /// Extracts the country/region code from a locale tag (e.g. "en-US" -> "US").
    pub fn extract_country_code(locale: &str) -> Option<&str> {
        let parts: Vec<&str> = locale.split(['-', '_']).collect();
        if parts.len() >= 2 {
            // Check the last segment
            let candidate = parts[parts.len() - 1];
            if candidate.len() == 2 && candidate.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some(candidate);
            }
            // If the second segment is 2 alpha chars
            let candidate = parts[1];
            if candidate.len() == 2 && candidate.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some(candidate);
            }
        }
        None
    }

    /// Detects the OS country code from the locale or returns None.
    pub fn from_os() -> Option<Self> {
        let locale = sys_locale::get_locale()?;
        let code = Self::extract_country_code(&locale)?;
        Self::ALL.iter().find(|c| c.code.eq_ignore_ascii_case(code)).copied()
    }
}

/// Carbon intensity data structure (g CO₂ per kWh).
#[derive(Debug, Clone, Copy)]
pub struct CarbonIntensity {
    pub country: CountryKey,
    pub g_per_kwh: f64,
}

impl CarbonIntensity {
    /// Returns all carbon intensity presets derived from [`CountryPreset::ALL`] plus Custom.
    pub fn presets() -> Vec<CarbonIntensity> {
        let mut list: Vec<CarbonIntensity> = CountryPreset::ALL
            .iter()
            .filter_map(|c| {
                c.carbon_intensity.map(|g_per_kwh| CarbonIntensity {
                    country: c.key,
                    g_per_kwh,
                })
            })
            .collect();
        list.push(CarbonIntensity {
            country: CountryKey::Custom,
            g_per_kwh: 0.0,
        });
        list
    }

    /// Returns the default preset based on OS locale, falling back to World average.
    pub fn from_os() -> Self {
        if let Some(country) = CountryPreset::from_os() {
            if let Some(intensity) = country.carbon_intensity {
                return CarbonIntensity {
                    country: country.key,
                    g_per_kwh: intensity,
                };
            }
        }
        CarbonIntensity {
            country: CountryKey::World,
            g_per_kwh: CountryPreset::WORLD_AVERAGE.carbon_intensity.unwrap_or(399.0),
        }
    }

    /// Returns true if this is a user-defined value.
    pub fn is_custom(self) -> bool {
        self.country == CountryKey::Custom
    }

    /// Resolves a stored country code or custom value.
    pub fn from_stored(value: &str) -> Self {
        if let Ok(value) = value.trim().parse::<f64>() {
            return CarbonIntensity {
                country: CountryKey::Custom,
                g_per_kwh: value,
            };
        }
        let country = CountryKey::from_code(value);
        CarbonIntensity {
            country,
            g_per_kwh: CountryPreset::by_key(country).carbon_intensity.unwrap_or(399.0),
        }
    }
}

impl PartialEq for CarbonIntensity {
    fn eq(&self, other: &Self) -> bool {
        self.country == other.country
    }
}

impl Display for CarbonIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:.0} g/kWh)",
            CountryPreset::by_key(self.country).label,
            self.g_per_kwh
        )
    }
}

/// Electricity cost data structure (price per kWh in local currency).
#[derive(Debug, Clone, Copy)]
pub struct ElectricityCost {
    pub country: CountryKey,
    /// Price per kWh in local currency.
    pub price_per_kwh: f64,
    /// Display currency symbol (e.g. "€", "$").
    pub currency_symbol: &'static str,
    /// ISO 4217 Currency code (e.g. "EUR", "USD").
    pub currency_code: &'static str,
}

impl ElectricityCost {
    /// Returns all electricity cost presets derived from [`CountryPreset::ALL`] plus Custom.
    pub fn presets() -> Vec<ElectricityCost> {
        let mut list: Vec<ElectricityCost> = CountryPreset::ALL
            .iter()
            .filter_map(|c| {
                if let (Some(price), Some(curr)) = (c.electricity_price, c.currency) {
                    Some(ElectricityCost {
                        country: c.key,
                        price_per_kwh: price,
                        currency_symbol: curr.symbol,
                        currency_code: curr.code,
                    })
                } else {
                    None
                }
            })
            .collect();
        list.push(ElectricityCost {
            country: CountryKey::Custom,
            price_per_kwh: 0.0,
            currency_symbol: "$",
            currency_code: "USD",
        });
        list
    }

    /// Returns the default preset based on OS locale, falling back to World average.
    pub fn from_os() -> Self {
        if let Some(country) = CountryPreset::from_os() {
            if let (Some(price), Some(curr)) = (country.electricity_price, country.currency) {
                return ElectricityCost {
                    country: country.key,
                    price_per_kwh: price,
                    currency_symbol: curr.symbol,
                    currency_code: curr.code,
                };
            }
        }
        let world = CountryPreset::WORLD_AVERAGE;
        let curr = world.currency.unwrap_or(Currency::USD);
        ElectricityCost {
            country: world.key,
            price_per_kwh: world.electricity_price.unwrap_or(0.18),
            currency_symbol: curr.symbol,
            currency_code: curr.code,
        }
    }

    pub fn is_custom(self) -> bool {
        self.country == CountryKey::Custom
    }

    /// Returns the Currency object for this cost setting.
    pub fn currency(self) -> Currency {
        Currency::from_code(self.currency_code)
    }

    /// Resolves a stored country code or custom value with an optional currency code.
    pub fn from_stored(value: &str, currency: Option<&str>) -> Self {
        if let Ok(value) = value.trim().parse::<f64>() {
            let curr = currency.map(Currency::from_code).unwrap_or(Currency::USD);
            return ElectricityCost {
                country: CountryKey::Custom,
                price_per_kwh: value,
                currency_symbol: curr.symbol,
                currency_code: curr.code,
            };
        }
        let country = CountryKey::from_code(value);
        let preset = CountryPreset::by_key(country);
        let curr = preset.currency.unwrap_or(Currency::USD);
        ElectricityCost {
            country,
            price_per_kwh: preset.electricity_price.unwrap_or(0.18),
            currency_symbol: curr.symbol,
            currency_code: curr.code,
        }
    }
}

impl PartialEq for ElectricityCost {
    fn eq(&self, other: &Self) -> bool {
        self.country == other.country
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
                CountryPreset::by_key(self.country).label,
                self.price_per_kwh,
                self.currency_symbol
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_country_code() {
        assert_eq!(CountryPreset::extract_country_code("en-US"), Some("US"));
        assert_eq!(CountryPreset::extract_country_code("fr_FR"), Some("FR"));
        assert_eq!(CountryPreset::extract_country_code("zh-Hans-CN"), Some("CN"));
        assert_eq!(CountryPreset::extract_country_code("de-DE"), Some("DE"));
        assert_eq!(CountryPreset::extract_country_code("en"), None);
    }

    #[test]
    fn test_country_presets_consistency() {
        let mut codes = std::collections::HashSet::new();
        let mut labels = std::collections::HashSet::new();
        let mut keys = std::collections::HashSet::new();
        for country in CountryPreset::ALL {
            assert!(!country.code.is_empty());
            assert!(!country.label.is_empty());
            if let Some(intensity) = country.carbon_intensity {
                assert!(intensity > 0.0);
            }
            if let Some(price) = country.electricity_price {
                assert!(price > 0.0);
                assert!(country.currency.is_some());
            }
            assert!(codes.insert(country.code), "Duplicate country code: {}", country.code);
            assert!(
                labels.insert(country.label),
                "Duplicate country label: {}",
                country.label
            );
            assert!(keys.insert(country.key), "Duplicate country key: {:?}", country.key);
        }
    }

    #[test]
    fn test_country_lookup_defaults_to_world() {
        assert_eq!(CountryPreset::by_key(CountryKey::Custom).key, CountryKey::World);
    }
}
