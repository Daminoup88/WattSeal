use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum TimeRange {
    #[default]
    LastMinute = 60,
    LastHour = 3600,
    Last24Hours = 86400,
}

impl TimeRange {
    pub fn unit(&self) -> &'static str {
        match self {
            TimeRange::LastMinute => "s",
            TimeRange::LastHour => "min",
            TimeRange::Last24Hours => "h",
        }
    }

    pub fn granularity_seconds(&self) -> i64 {
        match self {
            TimeRange::LastMinute => 1,
            TimeRange::LastHour => 60,
            TimeRange::Last24Hours => 3600,
        }
    }

    pub fn is_real_time(&self) -> bool {
        matches!(self, TimeRange::LastMinute)
    }

    pub fn duration_seconds(&self) -> Duration {
        chrono::Duration::seconds(self.clone() as i64)
    }
}

impl Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRange::LastMinute => write!(f, "Last Minute"),
            TimeRange::LastHour => write!(f, "Last Hour"),
            TimeRange::Last24Hours => write!(f, "Last 24 Hours"),
        }
    }
}
