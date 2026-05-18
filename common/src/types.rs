use std::{collections::HashMap, fmt::Display, ops::Mul, time::SystemTime};

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

/// Category of a sensor value (power, usage, or speed).
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum MetricType {
    #[default]
    Power,
    Usage,
    Speed,
}

impl Display for MetricType {
    // TODO Change metrics
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Power => write!(f, "Power"),
            MetricType::Usage => write!(f, "Usage"),
            MetricType::Speed => write!(f, "Speed"),
        }
    }
}

impl MetricType {
    /// Returns the human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            MetricType::Power => "Power",
            MetricType::Usage => "Usage",
            MetricType::Speed => "Speed",
        }
    }

    /// Returns the measurement unit string.
    pub fn unit(&self) -> &'static str {
        match self {
            MetricType::Power => "W",
            MetricType::Usage => "%",
            MetricType::Speed => "MB/s",
        }
    }

    /// Formats a chart legend label for the given component.
    pub fn legend(&self, component_name: &str) -> String {
        format!("{} {}", component_name, self.label())
    }

    /// Returns the display unit, swapping uj for Wh when energy mode, and W otherwise.
    pub fn effective_unit(&self, energy_mode: bool) -> &'static str {
        if *self == MetricType::Power {
            if energy_mode { "W" } else { "Wh" }
        } else {
            self.unit()
        }
    }
}

/// Named optional numeric value for secondary metrics.
#[derive(Debug, Clone, Copy)]
pub struct LabeledValue {
    pub label: &'static str,
    pub value: Option<f64>,
}

/// Collection of secondary metric values with their type.
#[derive(Debug, Clone)]
pub struct SecondaryValues {
    pub metric_type: MetricType,
    pub values: Vec<LabeledValue>,
}

impl SecondaryValues {
    fn from_labeled_values(metric_type: MetricType, values: Vec<LabeledValue>) -> Self {
        Self { metric_type, values }
    }

    /// Returns the list of labeled values.
    pub fn values(&self) -> &Vec<LabeledValue> {
        &self.values
    }

    /// Returns the metric type of these secondary values.
    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }
}

impl LabeledValue {
    fn from_percent(percent: Option<f64>, label: &'static str) -> Self {
        Self { label, value: percent }
    }

    fn from_usage_percent(percent: Option<f64>) -> Self {
        Self::from_percent(percent, "Usage")
    }

    fn from_mb_s(speed: Option<f64>, label: &'static str) -> Self {
        Self {
            label: label,
            value: speed,
        }
    }
}

impl<T: Clone + Mul<f64, Output = T>> SensorData<T> {
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

    /// Multiply all power fields by `factor`.
    /// Used to convert average watts → Wh when switching to energy mode.
    pub fn scale_power(&mut self, factor: f64) {
        match self {
            SensorData::CPU(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::GPU(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Ram(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Disk(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Network(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Total(d) => d.total_consumption = d.total_consumption.clone() * factor,
            SensorData::Process(procs) => {
                for p in procs {
                    p.process_consumption = p.process_consumption.clone() * factor;
                }
            }
        }
    }

    /// Returns secondary metrics (usage or speed) if applicable.
    pub fn secondary_values(&self) -> Option<SecondaryValues> {
        let metric_type = self.secondary_metric()?;
        match self {
            SensorData::CPU(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![LabeledValue::from_usage_percent(data.usage_percent)],
            )),
            SensorData::GPU(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![LabeledValue::from_usage_percent(data.usage_percent)],
            )),
            SensorData::Ram(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![LabeledValue::from_usage_percent(data.usage_percent)],
            )),
            SensorData::Disk(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![
                    LabeledValue::from_mb_s(Some(data.read_usage_mb_s), "Read"),
                    LabeledValue::from_mb_s(Some(data.write_usage_mb_s), "Write"),
                ],
            )),
            SensorData::Network(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![
                    LabeledValue::from_mb_s(Some(data.download_speed_mb_s), "Download"),
                    LabeledValue::from_mb_s(Some(data.upload_speed_mb_s), "Upload"),
                ],
            )),
            _ => None,
        }
    }

    /// Returns the secondary metric type for this sensor variant.
    pub fn secondary_metric(&self) -> Option<MetricType> {
        match self {
            SensorData::CPU(_) | SensorData::GPU(_) | SensorData::Ram(_) => Some(MetricType::Usage),
            SensorData::Disk(_) | SensorData::Network(_) => Some(MetricType::Speed),
            _ => None,
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
