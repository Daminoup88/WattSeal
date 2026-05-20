use std::{collections::HashMap, fmt::Display, time::SystemTime};

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

/// Raw RGBA icon pixel data.
#[derive(Debug, Clone, Serialize)]
pub struct IconData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Per-application resource usage snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessData<T> {
    pub app_name: String,
    pub process_exe_path: Option<String>,
    pub process_consumption: T,
    pub process_cpu_usage: f64,
    pub process_gpu_usage: Option<f64>,
    pub process_mem_usage: f64,
    pub read_bytes_per_sec: f64,
    pub written_bytes_per_sec: f64,
    pub subprocess_count: u32,
    pub icon: Option<IconData>,
}

/// Aggregated total consumption across all components.
#[derive(Debug, Clone, Serialize)]
pub struct TotalData<T> {
    pub total_consumption: T,
    pub period_type: String,
}

/// Tagged union of all sensor reading types.
#[derive(Debug, Clone, Serialize)]
pub enum SensorData<T> {
    CPU(CPUData<T>),
    GPU(GPUData<T>),
    Ram(RamData<T>),
    Disk(DiskData<T>),
    Network(NetworkData<T>),
    Total(TotalData<T>),
    Process(Vec<ProcessData<T>>),
}

/// Sensor component category type.
#[derive(Debug, Clone)]
pub enum SensorKind {
    CPU,
    GPU,
    Ram,
    Disk,
    Network,
    Total,
    Process,
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

/// Possible energy unit of sensors results.
pub enum EnergyUnit {
    WattHeure,
    UJoul,
}

/// Possible power unit of sensors results.
pub enum PowerUnit {
    Watt,
}

/// Possible consumption unit of sensors results.
pub enum ConsumptionUnit {
    Energy(EnergyUnit),
    Power(PowerUnit),
}

pub struct ConsumptionMetric {
    pub value: f64,
    pub unit: ConsumptionUnit,
}

impl Display for EnergyUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnergyUnit::WattHeure => write!(f, "Wh"),
            EnergyUnit::UJoul => write!(f, "uj"),
        }
    }
}

impl Display for PowerUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerUnit::Watt => write!(f, "W"),
        }
    }
}

impl Display for ConsumptionUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumptionUnit::Energy(e) => write!(f, "{}", e),
            ConsumptionUnit::Power(p) => write!(f, "{}", p),
        }
    }
}

impl Display for ConsumptionMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.unit)
    }
}

impl<T: Clone> SensorData<T> {
    /// Returns the sensor kind of this sensor variant.
    pub fn sensor_kind(&self) -> SensorKind {
        match self {
            SensorData::CPU(_) => SensorKind::CPU,
            SensorData::GPU(_) => SensorKind::GPU,
            SensorData::Ram(_) => SensorKind::Ram,
            SensorData::Disk(_) => SensorKind::Disk,
            SensorData::Network(_) => SensorKind::Network,
            SensorData::Total(_) => SensorKind::Total,
            SensorData::Process(_) => SensorKind::Process,
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
            SensorData::Total(power) => Some(power.total_consumption.clone()),
            SensorData::Process(_) => None,
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
            SensorKind::Total => write!(f, "Total"),
            SensorKind::Process => write!(f, "Process"),
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
            SensorData::Total(total) => writeln!(
                f,
                "Total Consumption during 1 {}: {}",
                total.period_type, total.total_consumption
            ),
            SensorData::Process(processes) => {
                writeln!(f, "Top Processes by CPU Usage:")?;
                writeln!(
                    f,
                    "{:<30} {:>10} {:>10} {:>10} {:>10} {:>15} {:>15} {:>20}",
                    "App Name", "CPU %", "GPU %", "Mem %", "Consumption", "Read MB/s", "Write MB/s", "Subprocesses"
                )?;
                for process in processes {
                    write!(f, "{}", process)?;
                }
                Ok(())
            }
        }
    }
}

impl<T: Display> Display for ProcessData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:<30} {:>10.2} {:>10} {:>10.2} {:>10} {:>15.2} {:>15.2} {:>20}",
            self.app_name,
            self.process_cpu_usage,
            self.process_gpu_usage
                .map(|u| format!("{:.2} %", u))
                .unwrap_or_else(|| "N/A".to_string()),
            self.process_mem_usage,
            self.process_consumption,
            self.read_bytes_per_sec / 1_000_000.0,    // Convert to MB/s
            self.written_bytes_per_sec / 1_000_000.0, // Convert to MB/s
            self.subprocess_count
        )?;
        Ok(())
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

impl<T> From<TotalData<T>> for SensorData<T> {
    fn from(data: TotalData<T>) -> Self {
        SensorData::Total(data)
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

impl<T> From<ProcessData<T>> for SensorData<T> {
    fn from(data: ProcessData<T>) -> Self {
        SensorData::Process(vec![data])
    }
}
