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
pub struct Event {
    time: SystemTime,
    data: Vec<SensorData>,
}

impl Event {
    pub fn new(value: Vec<impl Into<SensorData>>) -> Self {
        Event {
            time: SystemTime::now(),
            data: value.into_iter().map(|v| v.into()).collect(),
        }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn data(&self) -> &Vec<SensorData> {
        &self.data
    }
}

#[derive(Debug, Clone)]
pub struct CPUData {
    pub total_power_watts: Option<f64>,
    pub pp0_power_watts: Option<f64>,
    pub pp1_power_watts: Option<f64>,
    pub dram_power_watts: Option<f64>,
    pub usage_percent: f64,
}

#[derive(Debug, Clone)]
pub struct GPUData {
    pub total_power_watts: Option<f64>,
    pub usage_percent: Option<f64>,
    pub vram_usage_percent: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ScreenData {
    pub resolution: (u32, u32),
    pub refresh_rate_hz: u32,
    pub technology: String,
    pub luminosity_nits: u32,
}

#[derive(Debug, Clone)]
pub struct BatteryData {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub design_capacity_mwh: u32,
    pub full_charge_capacity_mwh: u32,
    pub cycle_count: u32,
}

#[derive(Debug, Clone)]
pub struct PeripheralsData {
    pub device_name: String,
    pub device_type: String,
    pub manufacturer: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub enum SensorData {
    CPU(CPUData),
    GPU(GPUData),
    Screen(ScreenData),
    Battery(BatteryData),
    Peripherals(PeripheralsData),
}

impl From<CPUData> for SensorData {
    fn from(data: CPUData) -> Self {
        SensorData::CPU(data)
    }
}

impl From<GPUData> for SensorData {
    fn from(data: GPUData) -> Self {
        SensorData::GPU(data)
    }
}

impl From<ScreenData> for SensorData {
    fn from(data: ScreenData) -> Self {
        SensorData::Screen(data)
    }
}

impl From<BatteryData> for SensorData {
    fn from(data: BatteryData) -> Self {
        SensorData::Battery(data)
    }
}

impl From<PeripheralsData> for SensorData {
    fn from(data: PeripheralsData) -> Self {
        SensorData::Peripherals(data)
    }
}
