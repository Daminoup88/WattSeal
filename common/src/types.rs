use std::{collections::HashMap, fmt::Display, time::SystemTime};

use crate::DatabaseEntry;

#[derive(Debug, Clone)]
pub struct Event {
    time: SystemTime,
    data: Vec<SensorData>,
}

impl Event {
    pub fn new(time: SystemTime, data: Vec<SensorData>) -> Self {
        Event { time, data }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn data(&self) -> &Vec<SensorData> {
        &self.data
    }

    pub fn push_data(&mut self, data: SensorData) {
        self.data.push(data);
    }
}

#[derive(Debug, Clone, Default)]
pub struct CPUData {
    pub total_power_watts: Option<f64>,
    pub pp0_power_watts: Option<f64>,
    pub pp1_power_watts: Option<f64>,
    pub dram_power_watts: Option<f64>,
    pub usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct GPUData {
    pub total_power_watts: Option<f64>,
    pub usage_percent: Option<f64>,
    pub vram_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct ScreenData {
    pub resolution: (u32, u32),
    pub refresh_rate_hz: u32,
    pub technology: String,
    pub luminosity_nits: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryData {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub design_capacity_mwh: u32,
    pub full_charge_capacity_mwh: u32,
    pub cycle_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PeripheralsData {
    pub device_name: String,
    pub device_type: String,
    pub manufacturer: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessData {
    pub app_name: String,
    pub vram_usage: f64,
    pub cpu_usage_watts: f64,
    pub subprocess_count: u32,
}

#[derive(Debug, Clone)]
pub enum SensorData {
    CPU(CPUData),
    GPU(GPUData),
    Screen(ScreenData),
    Battery(BatteryData),
    Peripherals(PeripheralsData),
    Total(TotalData),
    Process(Vec<ProcessData>),
}

#[derive(Debug, Clone, Default)]
pub struct TotalData {
    pub total_power_watts: f64,
}

impl SensorData {
    pub fn get_matching_sensor_data(table_name: &str) -> Option<SensorData> {
        match table_name {
            s if s == CPUData::table_name_static() => Some(SensorData::CPU(CPUData::default())),
            s if s == GPUData::table_name_static() => Some(SensorData::GPU(GPUData::default())),
            // s if s == ScreenData::table_name_static() => Some(SensorData::Screen(ScreenData::default())),
            // s if s == BatteryData::table_name_static() => Some(SensorData::Battery(BatteryData::default())),
            // s if s == PeripheralsData::table_name_static() => Some(SensorData::Peripherals(PeripheralsData::default())),
            s if s == TotalData::table_name_static() => Some(SensorData::Total(TotalData::default())),
            _ => None,
        }
    }

    pub fn sensor_type(&self) -> &'static str {
        match self {
            SensorData::CPU(_) => "CPU",
            SensorData::GPU(_) => "GPU",
            SensorData::Screen(_) => "Screen",
            SensorData::Battery(_) => "Battery",
            SensorData::Peripherals(_) => "Peripherals",
            SensorData::Total(_) => "Total",
            SensorData::Process(_) => "Processes",
        }
    }

    pub fn table_name(&self) -> &'static str {
        match self {
            SensorData::CPU(_) => CPUData::table_name_static(),
            SensorData::GPU(_) => GPUData::table_name_static(),
            SensorData::Total(_) => TotalData::table_name_static(),
            _ => "",
        }
    }

    pub fn total_power_watts(&self) -> Option<f64> {
        match self {
            SensorData::CPU(data) => data.total_power_watts,
            SensorData::GPU(data) => data.total_power_watts,
            SensorData::Total(power) => Some(power.total_power_watts),
            _ => None,
        }
    }

    pub fn usage_percent(&self) -> Option<f64> {
        match self {
            SensorData::CPU(data) => data.usage_percent,
            SensorData::GPU(data) => data.usage_percent,
            _ => None,
        }
    }
}

impl Display for SensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::CPU(data) => {
                writeln!(f, "  Power PKG:  {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Power PP0:  {:.3} W", data.pp0_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Power PP1:  {:.3} W", data.pp1_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Power DRAM: {:.3} W", data.dram_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Usage:      {:.2} %", data.usage_percent.unwrap_or(-1.0))?;
                Ok(())
            }
            SensorData::GPU(data) => {
                writeln!(f, "  Power:       {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Usage:       {:.2} %", data.usage_percent.unwrap_or(-1.0))?;
                writeln!(f, "  VRAM Usage:  {:.2} %", data.vram_usage_percent.unwrap_or(-1.0))?;
                Ok(())
            }
            SensorData::Screen(data) => write!(f, "Screen Data: {:?}", data),
            SensorData::Battery(data) => write!(f, "Battery Data: {:?}", data),
            SensorData::Peripherals(data) => write!(f, "Peripherals Data: {:?}", data),
            SensorData::Total(power) => write!(f, "Total Power: {:.3} W", power.total_power_watts),
            SensorData::Process(processes) => {
                writeln!(f, "Top Processes by CPU Usage:")?;
                writeln!(
                    f,
                    "{:<30} {:>10} {:>15} {:>10}",
                    "Application", "CPU (W)", "VRAM (MB)", "Subprocesses"
                )?;
                for process in processes {
                    writeln!(
                        f,
                        "{:<30} {:>10.2} {:>15.2} {:>10}",
                        process.app_name, process.cpu_usage_watts, process.vram_usage, process.subprocess_count
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl Display for ProcessData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "App: {} | VRAM: {:.2}MB | Power(CPU): {:.3}W | Nb: {}",
            self.app_name, self.vram_usage, self.cpu_usage_watts, self.subprocess_count
        )?;
        Ok(())
    }
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

impl From<TotalData> for SensorData {
    fn from(data: TotalData) -> Self {
        SensorData::Total(data)
    }
}
