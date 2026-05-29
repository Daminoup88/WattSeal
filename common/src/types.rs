use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

/// Timestamped collection of sensor readings.
#[derive(Debug, Clone)]
pub struct Event<T> {
    time: SystemTime,
    data: Vec<SensorData<T>>,
}

impl<T> Event<T> {
    /// Creates an event with the given timestamp and sensor data.
    pub fn new(time: SystemTime, data: Vec<SensorData<T>>) -> Self {
        Event { time, data }
    }

    /// Returns the event timestamp.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the list of sensor readings.
    pub fn data(&self) -> &Vec<SensorData<T>> {
        &self.data
    }

    /// Appends a sensor reading to this event.
    pub fn push_data(&mut self, data: SensorData<T>) {
        self.data.push(data);
    }
}

/// Cumulative per-component total consumption values.
#[derive(Debug, Clone, Default)]
pub struct AllTimeData<T> {
    pub components: HashMap<String, T>,
}

/// CPU consumption and usage readings.
#[derive(Debug, Clone, Serialize)]
pub struct CPUData<T> {
    pub total_consumption: Option<T>,
    pub pp0_consumption: Option<T>,
    pub pp1_consumption: Option<T>,
    pub dram_consumption: Option<T>,
    pub usage_percent: Option<f64>,
}

/// GPU consumption and usage readings.
#[derive(Debug, Clone, Serialize)]
pub struct GPUData<T> {
    pub total_consumption: Option<T>,
    pub usage_percent: Option<f64>,
    pub vram_usage_percent: Option<f64>,
}

/// RAM consumption and usage readings.
#[derive(Debug, Clone, Serialize)]
pub struct RamData<T> {
    pub total_consumption: Option<T>,
    pub usage_percent: Option<f64>,
}

/// Disk consumption and I/O throughput readings.
#[derive(Debug, Clone, Serialize)]
pub struct DiskData<T> {
    pub total_consumption: Option<T>,
    pub read_usage_mb_s: f64,
    pub write_usage_mb_s: f64,
}

/// Network consuption and throughput readings.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkData<T> {
    pub total_consumption: Option<T>,
    pub download_speed_mb_s: f64,
    pub upload_speed_mb_s: f64,
}

/// Tagged union of all sensor reading types.
#[derive(Debug, Clone, Serialize)]
pub enum SensorData<T> {
    CPU(CPUData<T>),
    GPU(GPUData<T>),
    Ram(RamData<T>),
    Disk(DiskData<T>),
    Network(NetworkData<T>),
}

/// Sensor component category type.
#[derive(Debug, Clone)]
pub enum SensorKind {
    CPU,
    GPU,
    Ram,
    Disk,
    Network,
}

/// Hardware information variant collected at startup.
pub enum InitialInfo {
    System(SystemInfo),
    CPU(CpuInfo),
    Memory(MemoryInfo),
    Gpus(Vec<String>),
    Disks(Vec<DiskInfo>),
    Displays(Vec<ScreenInfo>),
    Battery(BatteryInfo),
}

/// Complete hardware inventory of the system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareInfo {
    pub system: SystemInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<String>,
    pub disks: Vec<DiskInfo>,
    pub displays: Vec<ScreenInfo>,
    pub battery: BatteryInfo,
}

impl HardwareInfo {
    /// Serializes this hardware info to a JSON string.
    pub fn serialized(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json_string) => json_string,
            Err(e) => {
                crate::clog!("✗ Failed to serialize hardware info to JSON: {}", e);
                "{}".to_string()
            }
        }
    }
}

/// Metadata pairing entry list with serialized hardware info.
#[derive(Debug, Clone)]
pub struct GeneralData {
    pub sensors: Vec<SensorKind>,
    pub hardware_info: HardwareInfo,
}

impl From<Vec<InitialInfo>> for HardwareInfo {
    fn from(infos: Vec<InitialInfo>) -> Self {
        let mut system_info = None;
        let mut cpu_info = None;
        let mut memory_info = None;
        let mut gpu_list = None;
        let mut disk_infos = None;
        let mut display_infos = None;
        let mut battery_info = None;

        for info in infos {
            match info {
                InitialInfo::System(sys) => system_info = Some(sys),
                InitialInfo::CPU(cpu) => cpu_info = Some(cpu),
                InitialInfo::Memory(mem) => memory_info = Some(mem),
                InitialInfo::Gpus(gpus) => gpu_list = Some(gpus),
                InitialInfo::Disks(disks) => disk_infos = Some(disks),
                InitialInfo::Displays(displays) => display_infos = Some(displays),
                InitialInfo::Battery(battery) => battery_info = Some(battery),
            }
        }

        HardwareInfo {
            system: system_info.unwrap_or_default(),
            cpu: cpu_info.unwrap_or_default(),
            memory: memory_info.unwrap_or_default(),
            gpus: gpu_list.unwrap_or_default(),
            disks: disk_infos.unwrap_or_default(),
            displays: display_infos.unwrap_or_default(),
            battery: battery_info.unwrap_or_default(),
        }
    }
}

/// Operating system and host information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub os: String,
    pub hostname: String,
    pub is_virtual_machine: bool,
}

/// CPU model, vendor, and core count.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub physical_cores: u16,
    pub logical_cores: u16,
    pub base_frequency_mhz: u64,
    pub architecture: String,
}

/// Total physical and swap memory sizes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    pub total_ram_bytes: u64,
    pub total_swap_bytes: u64,
}

/// Disk name, mount point, and capacity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub disk_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

/// Display model, resolution, and refresh rate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScreenInfo {
    pub model: String,
    pub resolution: String,
    pub refresh_rate_hz: u32,
    pub is_primary: bool,
}

/// Battery presence, capacity, and cycle count.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatteryInfo {
    pub present: bool,
    pub name: Option<String>,
    pub design_capacity_wh: Option<f32>,
    pub full_charge_capacity_wh: Option<f32>,
    pub cycle_count: Option<u32>,
}

pub type PowerWatt = f64;
pub type EnergyWH = f64;
pub type EnergyUJ = u64;

impl<T: Clone> SensorData<T> {
    /// Returns the sensor kind of this sensor variant.
    pub fn sensor_kind(&self) -> SensorKind {
        match self {
            SensorData::CPU(_) => SensorKind::CPU,
            SensorData::GPU(_) => SensorKind::GPU,
            SensorData::Ram(_) => SensorKind::Ram,
            SensorData::Disk(_) => SensorKind::Disk,
            SensorData::Network(_) => SensorKind::Network,
        }
    }

    /// Returns the total consumption value, if available.
    pub fn total_consumption(&self) -> Option<T> {
        match self {
            SensorData::CPU(data) => data.total_consumption.clone(),
            SensorData::GPU(data) => data.total_consumption.clone(),
            SensorData::Ram(data) => data.total_consumption.clone(),
            SensorData::Disk(data) => data.total_consumption.clone(),
            SensorData::Network(data) => data.total_consumption.clone(),
        }
    }
}

impl Display for SensorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorKind::CPU => write!(f, "CPU"),
            SensorKind::GPU => write!(f, "GPU"),
            SensorKind::Ram => write!(f, "Ram"),
            SensorKind::Disk => write!(f, "Disk"),
            SensorKind::Network => write!(f, "Network"),
        }
    }
}

impl<T: Display> Display for SensorData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::CPU(data) => {
                writeln!(f, "CPU Data:")?;
                writeln!(
                    f,
                    "  Consumption PKG:  {}",
                    data.total_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    "  Consumption PP0:  {}",
                    data.pp0_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    "  Consumption PP1:  {}",
                    data.pp1_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    "  Consumption DRAM: {}",
                    data.dram_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    "  Usage:      {}",
                    data.usage_percent
                        .map(|u| format!("{:.2} %", u))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                Ok(())
            }
            SensorData::GPU(data) => {
                writeln!(f, "GPU Data:")?;
                writeln!(
                    f,
                    "  Consumption:       {}",
                    data.total_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    "  Usage:       {}",
                    data.usage_percent
                        .map(|u| format!("{:.2} %", u))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    "  VRAM Usage:  {}",
                    data.vram_usage_percent
                        .map(|u| format!("{:.2} %", u))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                Ok(())
            }
            SensorData::Ram(data) => {
                writeln!(f, "RAM Data:")?;
                writeln!(
                    f,
                    "  Consumption: {}",
                    data.total_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(
                    f,
                    " Usage: {}",
                    data.usage_percent
                        .map(|u| format!("{:.2} %", u))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                Ok(())
            }
            SensorData::Disk(data) => {
                writeln!(f, "Disk Data:")?;
                writeln!(
                    f,
                    "  Consumption: {}",
                    data.total_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(f, "  Read Speed:  {:.2} MB/s", data.read_usage_mb_s)?;
                writeln!(f, "  Write Speed: {:.2} MB/s", data.write_usage_mb_s)?;
                Ok(())
            }
            SensorData::Network(data) => {
                writeln!(f, "Network Data:")?;
                writeln!(
                    f,
                    "  Consumption:        {}",
                    data.total_consumption
                        .as_ref()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| "N/A".to_string())
                )?;
                writeln!(f, "  Download Speed: {:.2} MB/s", data.download_speed_mb_s)?;
                writeln!(f, "  Upload Speed:   {:.2} MB/s", data.upload_speed_mb_s)?;
                Ok(())
            }
        }
    }
}

impl<T> From<CPUData<T>> for SensorData<T> {
    fn from(data: CPUData<T>) -> Self {
        SensorData::CPU(data)
    }
}

impl<T> From<GPUData<T>> for SensorData<T> {
    fn from(data: GPUData<T>) -> Self {
        SensorData::GPU(data)
    }
}

impl<T> From<RamData<T>> for SensorData<T> {
    fn from(data: RamData<T>) -> Self {
        SensorData::Ram(data)
    }
}
impl<T> From<DiskData<T>> for SensorData<T> {
    fn from(data: DiskData<T>) -> Self {
        SensorData::Disk(data)
    }
}
impl<T> From<NetworkData<T>> for SensorData<T> {
    fn from(data: NetworkData<T>) -> Self {
        SensorData::Network(data)
    }
}

fn uj_to_wh(uj: EnergyUJ) -> EnergyWH {
    (uj as f64) / 3_600_000_000.0
}

fn uj_to_watts(uj: u64, duration: Duration) -> f64 {
    let j = (uj as f64) / 1_000_000.0;
    let secs = duration.as_secs_f64().max(0.001);
    j / secs
}

impl CPUData<EnergyUJ> {
    fn to_watts(&self, d: Duration) -> CPUData<PowerWatt> {
        CPUData {
            total_consumption: self.total_consumption.map(|t| uj_to_watts(t, d)),
            pp0_consumption: self.pp0_consumption.map(|pp0| uj_to_watts(pp0, d)),
            pp1_consumption: self.pp1_consumption.map(|pp1| uj_to_watts(pp1, d)),
            dram_consumption: self.dram_consumption.map(|dram| uj_to_watts(dram, d)),
            usage_percent: self.usage_percent,
        }
    }

    fn to_wh(&self) -> CPUData<EnergyWH> {
        CPUData {
            total_consumption: self.total_consumption.map(|t| uj_to_wh(t)),
            pp0_consumption: self.pp0_consumption.map(|pp0| uj_to_wh(pp0)),
            pp1_consumption: self.pp1_consumption.map(|pp1| uj_to_wh(pp1)),
            dram_consumption: self.dram_consumption.map(|dram| uj_to_wh(dram)),
            usage_percent: self.usage_percent,
        }
    }
}

impl GPUData<EnergyUJ> {
    fn to_watts(&self, d: Duration) -> GPUData<PowerWatt> {
        GPUData {
            total_consumption: self.total_consumption.map(|t| uj_to_watts(t, d)),
            usage_percent: self.usage_percent,
            vram_usage_percent: self.vram_usage_percent,
        }
    }

    fn to_wh(&self) -> GPUData<EnergyWH> {
        GPUData {
            total_consumption: self.total_consumption.map(|t| uj_to_wh(t)),
            usage_percent: self.usage_percent,
            vram_usage_percent: self.vram_usage_percent,
        }
    }
}

impl RamData<EnergyUJ> {
    fn to_watts(&self, d: Duration) -> RamData<PowerWatt> {
        RamData {
            total_consumption: self.total_consumption.map(|t| uj_to_watts(t, d)),
            usage_percent: self.usage_percent,
        }
    }

    fn to_wh(&self) -> RamData<EnergyWH> {
        RamData {
            total_consumption: self.total_consumption.map(|t| uj_to_wh(t)),
            usage_percent: self.usage_percent,
        }
    }
}

impl DiskData<EnergyUJ> {
    fn to_watts(&self, d: Duration) -> DiskData<PowerWatt> {
        DiskData {
            total_consumption: self.total_consumption.map(|t| uj_to_watts(t, d)),
            read_usage_mb_s: self.read_usage_mb_s,
            write_usage_mb_s: self.write_usage_mb_s,
        }
    }

    fn to_wh(&self) -> DiskData<EnergyWH> {
        DiskData {
            total_consumption: self.total_consumption.map(|t| uj_to_wh(t)),
            read_usage_mb_s: self.read_usage_mb_s,
            write_usage_mb_s: self.write_usage_mb_s,
        }
    }
}

impl NetworkData<EnergyUJ> {
    fn to_watts(&self, d: Duration) -> NetworkData<PowerWatt> {
        NetworkData {
            total_consumption: self.total_consumption.map(|t| uj_to_watts(t, d)),
            download_speed_mb_s: self.download_speed_mb_s,
            upload_speed_mb_s: self.upload_speed_mb_s,
        }
    }

    fn to_wh(&self) -> NetworkData<EnergyWH> {
        NetworkData {
            total_consumption: self.total_consumption.map(|t| uj_to_wh(t)),
            download_speed_mb_s: self.download_speed_mb_s,
            upload_speed_mb_s: self.upload_speed_mb_s,
        }
    }
}

impl SensorData<EnergyUJ> {
    pub fn to_watts(&self, d: Duration) -> SensorData<PowerWatt> {
        match self {
            SensorData::CPU(cpudata) => SensorData::CPU(cpudata.to_watts(d)),
            SensorData::GPU(gpudata) => SensorData::GPU(gpudata.to_watts(d)),
            SensorData::Ram(ramdata) => SensorData::Ram(ramdata.to_watts(d)),
            SensorData::Disk(diskdata) => SensorData::Disk(diskdata.to_watts(d)),
            SensorData::Network(networkdata) => SensorData::Network(networkdata.to_watts(d)),
        }
    }

    pub fn to_wh(&self) -> SensorData<EnergyWH> {
        match self {
            SensorData::CPU(cpudata) => SensorData::CPU(cpudata.to_wh()),
            SensorData::GPU(gpudata) => SensorData::GPU(gpudata.to_wh()),
            SensorData::Ram(ramdata) => SensorData::Ram(ramdata.to_wh()),
            SensorData::Disk(diskdata) => SensorData::Disk(diskdata.to_wh()),
            SensorData::Network(networkdata) => SensorData::Network(networkdata.to_wh()),
        }
    }
}
