use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};

use crate::core::types::{BatteryData, CPUData, Event, GPUData, PeripheralsData, ScreenData};
