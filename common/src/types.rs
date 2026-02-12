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
pub struct RamData {
    pub total_power_watts: Option<f64>,
    // pub total_gb: f64,
    pub usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct DiskData {
    pub total_power_watts: Option<f64>,
    // pub total_gb: f64,
    // pub used_gb: f64,
    // pub free_gb: f64,
    pub read_usage_mb_s: f64,
    pub write_usage_mb_s: f64,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkData {
    pub total_power_watts: Option<f64>,
    pub download_speed_mb_s: f64,
    pub upload_speed_mb_s: f64,
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
    Ram(RamData),
    Disk(DiskData),
    Network(NetworkData),
    Total(TotalData),
    Process(Vec<ProcessData>),
}

#[derive(Debug, Clone, Default)]
pub struct TotalData {
    pub total_power_watts: f64,
    pub period_type: String,
}

impl SensorData {
    pub fn get_matching_sensor_data(table_name: &str) -> Option<SensorData> {
        match table_name {
            s if s == CPUData::table_name_static() => Some(SensorData::CPU(CPUData::default())),
            s if s == GPUData::table_name_static() => Some(SensorData::GPU(GPUData::default())),
            s if s == RamData::table_name_static() => Some(SensorData::Ram(RamData::default())),
            s if s == DiskData::table_name_static() => Some(SensorData::Disk(DiskData::default())),
            s if s == NetworkData::table_name_static() => Some(SensorData::Network(NetworkData::default())),
            s if s == TotalData::table_name_static() => Some(SensorData::Total(TotalData::default())),
            _ => None,
        }
    }

    pub fn sensor_type(&self) -> &'static str {
        match self {
            SensorData::CPU(_) => "CPU",
            SensorData::GPU(_) => "GPU",
            SensorData::Ram(_) => "RAM",
            SensorData::Disk(_) => "Disk",
            SensorData::Network(_) => "Network",
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
            SensorData::Ram(data) => data.total_power_watts,
            SensorData::Disk(data) => data.total_power_watts,
            SensorData::Network(data) => data.total_power_watts,
            SensorData::Total(power) => Some(power.total_power_watts),
            _ => None,
        }
    }

    pub fn usage_percent(&self) -> Option<f64> {
        match self {
            SensorData::CPU(data) => data.usage_percent,
            SensorData::GPU(data) => data.usage_percent,
            SensorData::Ram(data) => data.usage_percent,
            _ => None,
        }
    }
}

impl Display for SensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::CPU(data) => {
                writeln!(f, "CPU Data:")?;
                writeln!(f, "  Power PKG:  {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Power PP0:  {:.3} W", data.pp0_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Power PP1:  {:.3} W", data.pp1_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Power DRAM: {:.3} W", data.dram_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Usage:      {:.2} %", data.usage_percent.unwrap_or(-1.0))?;
                Ok(())
            }
            SensorData::GPU(data) => {
                writeln!(f, "GPU Data:")?;
                writeln!(f, "  Power:       {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Usage:       {:.2} %", data.usage_percent.unwrap_or(-1.0))?;
                writeln!(f, "  VRAM Usage:  {:.2} %", data.vram_usage_percent.unwrap_or(-1.0))?;
                Ok(())
            }
            SensorData::Ram(data) => {
                writeln!(f, "RAM Data:")?;
                writeln!(f, "  Power: {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, " Usage: {:.2} %", data.usage_percent.unwrap_or(-1.0))?;
                Ok(())
            }
            SensorData::Disk(data) => {
                writeln!(f, "Disk Data:")?;
                writeln!(f, "  Power: {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Read Speed:  {:.2} MB/s", data.read_usage_mb_s)?;
                writeln!(f, "  Write Speed: {:.2} MB/s", data.write_usage_mb_s)?;
                Ok(())
            }
            SensorData::Network(data) => {
                writeln!(f, "Network Data:")?;
                writeln!(f, "  Power:        {:.3} W", data.total_power_watts.unwrap_or(-1.0))?;
                writeln!(f, "  Download Speed: {:.2} MB/s", data.download_speed_mb_s)?;
                writeln!(f, "  Upload Speed:   {:.2} MB/s", data.upload_speed_mb_s)?;
                Ok(())
            }
            SensorData::Total(total) => writeln!(
                f,
                "Total Power during 1 {}: {:.3} W",
                total.period_type, total.total_power_watts
            ),
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

impl From<TotalData> for SensorData {
    fn from(data: TotalData) -> Self {
        SensorData::Total(data)
    }
}
