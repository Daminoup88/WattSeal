// ## Architecture des données

// Événements (timestamp, valeur):
//     - POWER :
//         - Intel RAPL (PKG, PP0, PP1, DRAM)
//         - AMD RAPL
//         - NVSMI
//         - RAM (estimation)
//         - Disques, périphériques (estimation)
//         - Autres
//         - TOTAL
//     - UTILISATION :
//         - CPU
//         - GPU (NVSMI)
//         - RAM

// Configuration

use std::{fmt::Display, time::SystemTime};

#[derive(Debug, Clone)]
pub struct Event<T> {
    time: SystemTime,
    data: T,
}

impl Display for Event<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:?}", self.data, self.time)
    }
}

impl<T> Event<T> {
    pub fn new(value: T) -> Self {
        Event {
            time: SystemTime::now(),
            data: value,
        }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

#[derive(Debug, Clone)]
pub struct CPUData {
    pub total_power_watts: f64,
    pub pp0_power_watts: Option<f64>,
    pub pp1_power_watts: Option<f64>,
    pub dram_power_watts: Option<f64>,
    pub usage_percent: f64,
}

#[derive(Debug, Clone)]
pub struct GPUData {
    pub total_power_watts: f64,
    pub usage_percent: f64,
    pub memory_usage_percent: f64,
}

pub enum OS {
    Windows,
    Linux,
    MacOS,
}
