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
    pub vram_usage_percent: f64,
}

#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub resolution: (u32, u32),
    pub refresh_rate_hz: u32,
    pub technology: String,
    pub luminosity_nits: u32,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub design_capacity_mwh: u32,
    pub full_charge_capacity_mwh: u32,
    pub cycle_count: u32,
    pub health_percent: u8,
}

pub struct HardwareInfo {
    pub os: OS,
    pub total_ram_gb: u8,
    pub cpu_list:Vec<String>,
    pub gpu_list: Vec<String>,
    pub disk_list: Vec<String>,
    pub peripherals: Vec<String>,
    pub network_card: String,
    pub screen_info: ScreenInfo,
    pub battery_info: BatteryInfo,
}

pub enum OS {
    Windows,
    Linux,
    MacOS,
}
