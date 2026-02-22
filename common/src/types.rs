use std::{fmt::Display, time::SystemTime};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub struct CPUData {
    pub total_power_watts: Option<f64>,
    pub pp0_power_watts: Option<f64>,
    pub pp1_power_watts: Option<f64>,
    pub dram_power_watts: Option<f64>,
    pub usage_percent: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct GPUData {
    pub total_power_watts: Option<f64>,
    pub usage_percent: Option<f64>,
    pub vram_usage_percent: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct RamData {
    pub total_power_watts: Option<f64>,
    // pub total_gb: f64,
    pub usage_percent: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DiskData {
    pub total_power_watts: Option<f64>,
    // pub total_gb: f64,
    // pub used_gb: f64,
    // pub free_gb: f64,
    pub read_usage_mb_s: f64,
    pub write_usage_mb_s: f64,
}

#[derive(Debug, Clone)]
pub struct NetworkData {
    pub total_power_watts: Option<f64>,
    pub download_speed_mb_s: f64,
    pub upload_speed_mb_s: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessData {
    pub app_name: String,
    pub process_exe_path: Option<String>,
    pub process_usage_watt: f64,
    pub process_cpu_usage: f64,
    pub process_gpu_usage: Option<f64>,
    pub process_mem_usage: f64,
    pub read_bytes_per_sec: f64,
    pub written_bytes_per_sec: f64,
    pub subprocess_count: u32,
    pub icon: Option<Vec<u8>>,
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

#[derive(Debug, Clone)]
pub struct TotalData {
    pub total_power_watts: f64,
    pub period_type: String,
}

pub enum InitialInfo {
    System(SystemInfo),
    CPU(CpuInfo),
    Memory(MemoryInfo),
    Gpus(Vec<String>),
    Disks(Vec<DiskInfo>),
    Displays(Vec<ScreenInfo>),
    Battery(BatteryInfo),
}

#[derive(Debug, Clone)]
pub struct GeneralData {
    pub tables: String,
    pub detected_hardware: String,
    pub hardware_info_serialized: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub system: SystemInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<String>,
    pub disks: Vec<DiskInfo>,
    pub displays: Vec<ScreenInfo>,
    pub battery: BatteryInfo,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub os: String,
    pub hostname: String,
    pub architecture: String,
    pub is_virtual_machine: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub physical_cores: u16,
    pub logical_cores: u16,
    pub base_frequency_mhz: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    pub total_ram_bytes: u64,
    pub total_swap_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub disk_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScreenInfo {
    pub model: String,
    pub resolution: String,
    pub refresh_rate_hz: u32,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BatteryInfo {
    pub present: bool,
    pub name: Option<String>,
    pub design_capacity_wh: Option<f32>,
    pub full_charge_capacity_wh: Option<f32>,
    pub cycle_count: Option<u32>,
}

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum MetricType {
    #[default]
    Power,
    Usage,
    Speed,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Power => write!(f, "Power"),
            MetricType::Usage => write!(f, "Usage"),
            MetricType::Speed => write!(f, "Speed"),
        }
    }
}

impl MetricType {
    pub fn label(&self) -> &'static str {
        match self {
            MetricType::Power => "Power",
            MetricType::Usage => "Usage",
            MetricType::Speed => "Speed",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            MetricType::Power => "W",
            MetricType::Usage => "%",
            MetricType::Speed => "MB/s",
        }
    }

    pub fn legend(&self, component_name: &str) -> String {
        format!("{} {}", component_name, self.label())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LabeledValue {
    pub label: &'static str,
    pub value: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct SecondaryValues {
    pub metric_type: MetricType,
    pub values: Vec<LabeledValue>,
}

impl SecondaryValues {
    fn from_labeled_values(metric_type: MetricType, values: Vec<LabeledValue>) -> Self {
        Self { metric_type, values }
    }

    pub fn values(&self) -> &Vec<LabeledValue> {
        &self.values
    }

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

impl SensorData {
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
            SensorData::Ram(_) => RamData::table_name_static(),
            SensorData::Disk(_) => DiskData::table_name_static(),
            SensorData::Network(_) => NetworkData::table_name_static(),
            SensorData::Process(_) => ProcessData::table_name_static(),
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
            SensorData::Process(_) => None,
        }
    }

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

    pub fn secondary_metric(&self) -> Option<MetricType> {
        match self {
            SensorData::CPU(_) | SensorData::GPU(_) | SensorData::Ram(_) => Some(MetricType::Usage),
            SensorData::Disk(_) | SensorData::Network(_) => Some(MetricType::Speed),
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
                    "{:<30} {:>10} {:>10} {:>10} {:>10} {:>15} {:>15} {:>20}",
                    "App Name", "CPU %", "GPU %", "Mem %", "Power W", "Read MB/s", "Write MB/s", "Subprocesses"
                )?;
                for process in processes {
                    write!(f, "{}", process)?;
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
            "{:<30} {:>10.2} {:>10.2} {:>10.2} {:>10.3} {:>15.2} {:>15.2} {:>20}",
            self.app_name,
            self.process_cpu_usage,
            self.process_gpu_usage.unwrap_or(0.0),
            self.process_mem_usage,
            self.process_usage_watt,
            self.read_bytes_per_sec / 1_000_000.0,    // Convert to MB/s
            self.written_bytes_per_sec / 1_000_000.0, // Convert to MB/s
            self.subprocess_count
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

impl From<RamData> for SensorData {
    fn from(data: RamData) -> Self {
        SensorData::Ram(data)
    }
}
impl From<DiskData> for SensorData {
    fn from(data: DiskData) -> Self {
        SensorData::Disk(data)
    }
}
impl From<NetworkData> for SensorData {
    fn from(data: NetworkData) -> Self {
        SensorData::Network(data)
    }
}

impl From<ProcessData> for SensorData {
    fn from(data: ProcessData) -> Self {
        SensorData::Process(vec![data])
    }
}

impl Default for CPUData {
    fn default() -> Self {
        CPUData {
            total_power_watts: Some(0.0),
            pp0_power_watts: Some(0.0),
            pp1_power_watts: Some(0.0),
            dram_power_watts: Some(0.0),
            usage_percent: Some(0.0),
        }
    }
}

impl Default for GPUData {
    fn default() -> Self {
        GPUData {
            total_power_watts: Some(0.0),
            usage_percent: Some(0.0),
            vram_usage_percent: Some(0.0),
        }
    }
}

impl Default for RamData {
    fn default() -> Self {
        RamData {
            total_power_watts: Some(0.0),
            usage_percent: Some(0.0),
        }
    }
}

impl Default for DiskData {
    fn default() -> Self {
        DiskData {
            total_power_watts: Some(0.0),
            read_usage_mb_s: 0.0,
            write_usage_mb_s: 0.0,
        }
    }
}

impl Default for NetworkData {
    fn default() -> Self {
        NetworkData {
            total_power_watts: Some(0.0),
            download_speed_mb_s: 0.0,
            upload_speed_mb_s: 0.0,
        }
    }
}

impl Default for ProcessData {
    fn default() -> Self {
        ProcessData {
            app_name: String::new(),
            process_exe_path: None,
            process_usage_watt: 0.0,
            process_cpu_usage: 0.0,
            process_gpu_usage: None,
            process_mem_usage: 0.0,
            read_bytes_per_sec: 0.0,
            written_bytes_per_sec: 0.0,
            subprocess_count: 0,
            icon: None,
        }
    }
}

impl Default for TotalData {
    fn default() -> Self {
        TotalData {
            total_power_watts: 0.0,
            period_type: "second".to_string(),
        }
    }
}
