use std::{collections::HashMap, fmt::Display, time::SystemTime};

use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};

use crate::DatabaseEntry;

pub const MICROJOULES_PER_JOULE: f64 = 1_000_000.0;
pub const SECONDS_PER_HOUR: f64 = 3600.0;

/// Timestamped collection of sensor readings.
#[derive(Debug, Clone)]
pub struct Event<T = SensorData> {
    time: SystemTime,
    data: Vec<T>,
}

impl<T> Event<T> {
    /// Creates an event with the given timestamp and sensor data.
    pub fn new(time: SystemTime, data: Vec<T>) -> Self {
        Event { time, data }
    }

    /// Returns the event timestamp.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the list of sensor readings.
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Appends a sensor reading to this event.
    pub fn push_data(&mut self, data: T) {
        self.data.push(data);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnergyUj(u64);

impl std::fmt::Display for EnergyUj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} µJ", self.0)
    }
}

impl EnergyUj {
    pub fn from_joules(joules: f64) -> Self {
        EnergyUj((joules * MICROJOULES_PER_JOULE) as u64)
    }

    pub fn from_millijoules(mj: u64) -> Self {
        EnergyUj(mj * 1_000)
    }

    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn as_joules(&self) -> f64 {
        self.as_f64() / MICROJOULES_PER_JOULE
    }

    pub fn as_watts_for_seconds(&self, seconds: f64) -> f64 {
        if seconds <= 0.0 {
            0.0
        } else {
            self.as_joules() / seconds
        }
    }

    pub fn as_watt_hours(&self) -> f64 {
        self.as_joules() / SECONDS_PER_HOUR
    }

    pub fn scale_by(self, factor: f64) -> Self {
        if !factor.is_finite() || factor <= 0.0 {
            EnergyUj(0)
        } else {
            EnergyUj::from_f64(self.as_f64() * factor)
        }
    }

    pub fn from_f64(value: f64) -> Self {
        EnergyUj(value.max(0.0) as u64)
    }

    pub fn from_u64(value: u64) -> Self {
        EnergyUj(value)
    }

    pub fn to_wh(self) -> EnergyWh {
        EnergyWh(self.as_watt_hours())
    }
}

impl ToSql for EnergyUj {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0 as i64))
    }
}

impl FromSql for EnergyUj {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_i64()
            .map(|v| EnergyUj(v as u64))
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

impl std::ops::AddAssign for EnergyUj {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl std::ops::SubAssign for EnergyUj {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl PartialEq<f64> for EnergyUj {
    fn eq(&self, other: &f64) -> bool {
        self.as_f64() == *other
    }
}

impl PartialOrd<f64> for EnergyUj {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self.as_f64().partial_cmp(other)
    }
}

impl std::ops::Mul<f64> for EnergyUj {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        self.scale_by(rhs)
    }
}

impl std::ops::MulAssign<f64> for EnergyUj {
    fn mul_assign(&mut self, rhs: f64) {
        *self = self.scale_by(rhs);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct EnergyWh(f64);

impl std::fmt::Display for EnergyWh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3} Wh", self.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PowerW(f64);

//byte unit
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Byte(u64);

impl Byte {
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn as_mb(&self) -> f64 {
        self.as_f64() / 1_000_000.0
    }

    pub fn from(value: u64) -> Self {
        Byte(value)
    }
}

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} B", self.0)
    }
}

impl std::ops::AddAssign for Byte {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl std::ops::AddAssign<u64> for Byte {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_add(rhs);
    }
}

impl ToSql for Byte {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0 as i64))
    }
}

impl FromSql for Byte {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_i64()
            .map(|v| Byte(v as u64))
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

/// Cumulative per-component energy totals.
#[derive(Debug, Clone, Default)]
pub struct AllTimeData<E = EnergyUj> {
    pub components: HashMap<String, E>,
}

/// CPU energy and usage readings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUData<E = EnergyUj> {
    pub total_energy: Option<E>,
    pub pp0_energy: Option<E>,
    pub pp1_energy: Option<E>,
    pub dram_energy: Option<E>,
    pub usage_percent: Option<f64>,
}

/// GPU energy and usage readings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUData<E = EnergyUj> {
    pub total_energy: Option<E>,
    pub usage_percent: Option<f64>,
    pub vram_usage_percent: Option<f64>,
    pub name: Option<String>,
}

/// RAM energy and usage readings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamData<E = EnergyUj> {
    pub total_energy: Option<E>,
    pub usage_percent: Option<f64>,
}

/// Disk energy and I/O throughput readings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskData<E = EnergyUj> {
    pub total_energy: Option<E>,
    pub read_bytes: Byte,
    pub written_bytes: Byte,
}

/// Network energy and throughput readings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkData<E = EnergyUj> {
    pub total_energy: Option<E>,
    pub downloaded_bytes: Byte,
    pub uploaded_bytes: Byte,
}

/// Raw RGBA icon pixel data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Measured process details (raw resource metrics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredProcessData {
    pub pid: Option<u32>,
    pub app_name: String,
    pub process_exe_path: Option<String>,
    pub process_cpu_usage: f64,
    pub process_gpu_usage: Option<f64>,
    pub process_mem_usage: f64,
    pub read_bytes: Byte,
    pub written_bytes: Byte,
}

/// Per-application resource usage snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessData<E = EnergyUj> {
    pub measured: MeasuredProcessData,
    pub process_energy: E,
    pub subprocess_count: u32,
    pub icon: Option<IconData>,
}

/// Tagged union of all raw measured sensor reading types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorData<E = EnergyUj> {
    CPU(CPUData<E>),
    GPU(GPUData<E>),
    Ram(RamData<E>),
    Disk(DiskData<E>),
    Network(NetworkData<E>),
    Process(Vec<MeasuredProcessData>),
}

/// Tagged union of all sensor reading types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputedSensorData<E = EnergyUj> {
    CPU(CPUData<E>),
    GPU(GPUData<E>),
    Ram(RamData<E>),
    Disk(DiskData<E>),
    Network(NetworkData<E>),
    Total(TotalData<E>),
    Process(Vec<ProcessData<E>>),
}

/// Aggregated total energy across all components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalData<E = EnergyUj> {
    pub total_energy: E,
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

/// Database metadata pairing table list with serialized hardware info.
#[derive(Debug)]
pub struct GeneralData {
    pub tables: String,
    pub hardware_info: HardwareInfo,
}

/// Complete hardware inventory of the system.
#[derive(Debug, Serialize, Deserialize, Default)]
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
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub os: String,
    pub hostname: String,
    pub is_virtual_machine: bool,
}

/// CPU model, vendor, and core count.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub physical_cores: u16,
    pub logical_cores: u16,
    pub base_frequency_mhz: u64,
    pub architecture: String,
}

/// Total physical and swap memory sizes.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    pub total_ram_bytes: u64,
    pub total_swap_bytes: u64,
}

/// Disk name, mount point, and capacity.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub disk_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

/// Display model, resolution, and refresh rate.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScreenInfo {
    pub model: String,
    pub resolution: String,
    pub refresh_rate_hz: u32,
    pub is_primary: bool,
}

/// Battery presence, capacity, and cycle count.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BatteryInfo {
    pub present: bool,
    pub name: Option<String>,
    pub design_capacity_wh: Option<f32>,
    pub full_charge_capacity_wh: Option<f32>,
    pub cycle_count: Option<u32>,
}

/// Category of a sensor value (power, usage, or speed).
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum MetricKind {
    #[default]
    Power,
    Usage,
    Speed,
}

impl Display for MetricKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricKind::Power => write!(f, "Power"),
            MetricKind::Usage => write!(f, "Usage"),
            MetricKind::Speed => write!(f, "Speed"),
        }
    }
}

impl MetricKind {
    /// Returns the human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            MetricKind::Power => "Power",
            MetricKind::Usage => "Usage",
            MetricKind::Speed => "Speed",
        }
    }

    /// Returns the measurement unit string.
    pub fn unit(&self) -> &'static str {
        match self {
            MetricKind::Power => "W",
            MetricKind::Usage => "%",
            MetricKind::Speed => "MB/s",
        }
    }

    /// Formats a chart legend label for the given component.
    pub fn legend(&self, component_name: &str) -> String {
        format!("{} {}", component_name, self.label())
    }

    /// Returns the display unit, swapping W for Wh in energy mode.
    pub fn effective_unit(&self, energy_mode: bool) -> &'static str {
        if *self == MetricKind::Power && energy_mode {
            "Wh"
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
    pub metric_type: MetricKind,
    pub values: Vec<LabeledValue>,
}

impl SecondaryValues {
    fn from_labeled_values(metric_type: MetricKind, values: Vec<LabeledValue>) -> Self {
        Self { metric_type, values }
    }

    /// Returns the list of labeled values.
    pub fn values(&self) -> &Vec<LabeledValue> {
        &self.values
    }

    /// Returns the metric type of these secondary values.
    pub fn metric_type(&self) -> MetricKind {
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
    /// Returns the display name of this sensor variant.
    pub fn sensor_type(&self) -> &'static str {
        match self {
            SensorData::CPU(_) => "CPU",
            SensorData::GPU(_) => "GPU",
            SensorData::Ram(_) => "RAM",
            SensorData::Disk(_) => "Disk",
            SensorData::Network(_) => "Network",
            SensorData::Process(_) => "Processes",
        }
    }

    /// Returns the total energy in µJ, if available.
    pub fn total_energy(&self) -> Option<EnergyUj> {
        match self {
            SensorData::CPU(data) => data.total_energy,
            SensorData::GPU(data) => data.total_energy,
            SensorData::Ram(data) => data.total_energy,
            SensorData::Disk(data) => data.total_energy,
            SensorData::Network(data) => data.total_energy,
            SensorData::Process(_) => None,
        }
    }

    /// Converts the sensor to SensorData<EnergyWh> by converting energy values to watt-hours.
    pub fn to_wh(&self) -> SensorData<EnergyWh> {
        match self {
            SensorData::CPU(data) => SensorData::CPU(CPUData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                pp0_energy: data.pp0_energy.map(|e| e.to_wh()),
                pp1_energy: data.pp1_energy.map(|e| e.to_wh()),
                dram_energy: data.dram_energy.map(|e| e.to_wh()),
                usage_percent: data.usage_percent,
            }),
            SensorData::GPU(data) => SensorData::GPU(GPUData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                usage_percent: data.usage_percent,
                vram_usage_percent: data.vram_usage_percent,
                name: data.name.clone(),
            }),
            SensorData::Ram(data) => SensorData::Ram(RamData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                usage_percent: data.usage_percent,
            }),
            SensorData::Disk(data) => SensorData::Disk(DiskData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                read_bytes: data.read_bytes,
                written_bytes: data.written_bytes,
            }),
            SensorData::Network(data) => SensorData::Network(NetworkData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                downloaded_bytes: data.downloaded_bytes,
                uploaded_bytes: data.uploaded_bytes,
            }),
            SensorData::Process(processes) => SensorData::Process(processes.to_vec()),
        }
    }
}

impl Display for SensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::CPU(data) => write!(f, "{}", data),
            SensorData::GPU(data) => write!(f, "{}", data),
            SensorData::Ram(data) => write!(f, "{}", data),
            SensorData::Disk(data) => write!(f, "{}", data),
            SensorData::Network(data) => write!(f, "{}", data),
            SensorData::Process(processes) => {
                writeln!(f, "Processes:")?;
                for p in processes {
                    writeln!(
                        f,
                        "- {} (PID: {:?}): CPU {}%, GPU {}%, Mem {}%, Read {}, Write {}",
                        p.app_name,
                        p.pid,
                        p.process_cpu_usage,
                        p.process_gpu_usage.map_or(0.0, |gpu| gpu),
                        p.process_mem_usage,
                        p.read_bytes,
                        p.written_bytes
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl ComputedSensorData {
    /// Returns the database table name for this variant.
    pub fn table_name(&self) -> &'static str {
        match self {
            ComputedSensorData::CPU(_) => CPUData::table_name_static(),
            ComputedSensorData::GPU(_) => GPUData::table_name_static(),
            ComputedSensorData::Total(_) => TotalData::table_name_static(),
            ComputedSensorData::Ram(_) => RamData::table_name_static(),
            ComputedSensorData::Disk(_) => DiskData::table_name_static(),
            ComputedSensorData::Network(_) => NetworkData::table_name_static(),
            ComputedSensorData::Process(_) => ProcessData::table_name_static(),
        }
    }

    pub fn sensor_type(&self) -> &'static str {
        match self {
            ComputedSensorData::CPU(_) => "CPU",
            ComputedSensorData::GPU(_) => "GPU",
            ComputedSensorData::Ram(_) => "RAM",
            ComputedSensorData::Disk(_) => "Disk",
            ComputedSensorData::Network(_) => "Network",
            ComputedSensorData::Total(_) => "Total",
            ComputedSensorData::Process(_) => "Processes",
        }
    }

    pub fn to_wh(&self) -> ComputedSensorData<EnergyWh> {
        match self {
            ComputedSensorData::CPU(data) => ComputedSensorData::CPU(CPUData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                pp0_energy: data.pp0_energy.map(|e| e.to_wh()),
                pp1_energy: data.pp1_energy.map(|e| e.to_wh()),
                dram_energy: data.dram_energy.map(|e| e.to_wh()),
                usage_percent: data.usage_percent,
            }),
            ComputedSensorData::GPU(data) => ComputedSensorData::GPU(GPUData {
                name: data.name.clone(),
                total_energy: data.total_energy.map(|e| e.to_wh()),
                usage_percent: data.usage_percent,
                vram_usage_percent: data.vram_usage_percent,
            }),
            ComputedSensorData::Ram(data) => ComputedSensorData::Ram(RamData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                usage_percent: data.usage_percent,
            }),
            ComputedSensorData::Disk(data) => ComputedSensorData::Disk(DiskData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                read_bytes: data.read_bytes,
                written_bytes: data.written_bytes,
            }),
            ComputedSensorData::Network(data) => ComputedSensorData::Network(NetworkData {
                total_energy: data.total_energy.map(|e| e.to_wh()),
                downloaded_bytes: data.downloaded_bytes,
                uploaded_bytes: data.uploaded_bytes,
            }),
            ComputedSensorData::Total(total) => ComputedSensorData::Total(TotalData {
                total_energy: total.total_energy.to_wh(),
            }),
            ComputedSensorData::Process(processes) => {
                let converted_processes = processes
                    .iter()
                    .map(|p| ProcessData {
                        measured: p.measured.clone(),
                        process_energy: p.process_energy.to_wh(),
                        subprocess_count: p.subprocess_count,
                        icon: p.icon.clone(),
                    })
                    .collect();
                ComputedSensorData::Process(converted_processes)
            }
        }
    }

    /// Returns the total energy in µJ, if available.
    pub fn total_energy(&self) -> Option<EnergyUj> {
        match self {
            ComputedSensorData::CPU(data) => data.total_energy,
            ComputedSensorData::GPU(data) => data.total_energy,
            ComputedSensorData::Ram(data) => data.total_energy,
            ComputedSensorData::Disk(data) => data.total_energy,
            ComputedSensorData::Network(data) => data.total_energy,
            ComputedSensorData::Total(power) => Some(power.total_energy),
            ComputedSensorData::Process(_) => None,
        }
    }

    /// Scales all energy fields by `factor`.
    pub fn scale_energy(&mut self, factor: f64) {
        match self {
            ComputedSensorData::CPU(d) => d.total_energy = d.total_energy.map(|w| w * factor),
            ComputedSensorData::GPU(d) => d.total_energy = d.total_energy.map(|w| w * factor),
            ComputedSensorData::Ram(d) => d.total_energy = d.total_energy.map(|w| w * factor),
            ComputedSensorData::Disk(d) => d.total_energy = d.total_energy.map(|w| w * factor),
            ComputedSensorData::Network(d) => d.total_energy = d.total_energy.map(|w| w * factor),
            ComputedSensorData::Total(d) => d.total_energy *= factor,
            ComputedSensorData::Process(procs) => {
                for p in procs {
                    p.process_energy *= factor;
                }
            }
        }
    }

    /// Returns secondary metrics (usage or speed) if applicable.
    pub fn secondary_values(&self) -> Option<SecondaryValues> {
        let metric_type = self.secondary_metric()?;
        match self {
            ComputedSensorData::CPU(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![LabeledValue::from_usage_percent(data.usage_percent)],
            )),
            ComputedSensorData::GPU(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![LabeledValue::from_usage_percent(data.usage_percent)],
            )),
            ComputedSensorData::Ram(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![LabeledValue::from_usage_percent(data.usage_percent)],
            )),
            ComputedSensorData::Disk(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![
                    LabeledValue::from_mb_s(Some(data.read_bytes.as_mb()), "Read"),
                    LabeledValue::from_mb_s(Some(data.written_bytes.as_mb()), "Write"),
                ],
            )),
            ComputedSensorData::Network(data) => Some(SecondaryValues::from_labeled_values(
                metric_type,
                vec![
                    LabeledValue::from_mb_s(Some(data.downloaded_bytes.as_mb()), "Download"),
                    LabeledValue::from_mb_s(Some(data.uploaded_bytes.as_mb()), "Upload"),
                ],
            )),
            _ => None,
        }
    }

    /// Returns the secondary metric type for this sensor variant.
    pub fn secondary_metric(&self) -> Option<MetricKind> {
        match self {
            ComputedSensorData::CPU(_) | ComputedSensorData::GPU(_) | ComputedSensorData::Ram(_) => {
                Some(MetricKind::Usage)
            }
            ComputedSensorData::Disk(_) | ComputedSensorData::Network(_) => Some(MetricKind::Speed),
            _ => None,
        }
    }
}

impl Display for ComputedSensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputedSensorData::CPU(data) => write!(f, "{}", data),
            ComputedSensorData::GPU(data) => write!(f, "{}", data),
            ComputedSensorData::Ram(data) => write!(f, "{}", data),
            ComputedSensorData::Disk(data) => write!(f, "{}", data),
            ComputedSensorData::Network(data) => write!(f, "{}", data),
            ComputedSensorData::Total(total) => writeln!(f, "Total Energy: {}", total.total_energy),
            ComputedSensorData::Process(processes) => {
                writeln!(f, "Top Processes by CPU Usage:")?;
                writeln!(
                    f,
                    "{:<30.30} {:>10} {:>10} {:>10} {:>16} {:>15} {:>15} {:>20}",
                    "App Name", "CPU %", "GPU %", "Mem %", "Energy", "Read MB", "Write MB", "Subprocesses"
                )?;
                for process in processes {
                    write!(f, "{}", process)?;
                }
                Ok(())
            }
        }
    }
}

pub trait PublishableSensor: Serialize {
    type Wh: Serialize;
    fn sensor_type(&self) -> &'static str;
    fn to_wh(&self) -> Self::Wh;
}

impl PublishableSensor for SensorData {
    type Wh = SensorData<EnergyWh>;
    fn sensor_type(&self) -> &'static str {
        SensorData::sensor_type(self)
    }
    fn to_wh(&self) -> Self::Wh {
        SensorData::to_wh(self)
    }
}

impl PublishableSensor for ComputedSensorData {
    type Wh = ComputedSensorData<EnergyWh>;
    fn sensor_type(&self) -> &'static str {
        ComputedSensorData::sensor_type(self)
    }
    fn to_wh(&self) -> Self::Wh {
        ComputedSensorData::to_wh(self)
    }
}

impl Display for ProcessData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Force conversion for alignment
        let energy_str = self.process_energy.to_string();
        writeln!(
            f,
            "{:<30.30} {:>10.2} {:>10.2} {:>10.2} {:>16} {:>15.2} {:>15.2} {:>20}",
            self.measured.app_name,
            self.measured.process_cpu_usage,
            self.measured.process_gpu_usage.unwrap_or(0.0),
            self.measured.process_mem_usage,
            energy_str,
            self.measured.read_bytes.as_mb(),
            self.measured.written_bytes.as_mb(),
            self.subprocess_count
        )?;
        Ok(())
    }
}

impl Display for CPUData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CPU Data:")?;
        writeln!(f, "  Energy PKG:  {}", self.total_energy.unwrap_or_default())?;
        writeln!(f, "  Energy PP0:  {}", self.pp0_energy.unwrap_or_default())?;
        writeln!(f, "  Energy PP1:  {}", self.pp1_energy.unwrap_or_default())?;
        writeln!(f, "  Energy DRAM: {}", self.dram_energy.unwrap_or_default())?;
        writeln!(f, "  Usage:       {:.2} %", self.usage_percent.unwrap_or(-1.0))?;
        Ok(())
    }
}

impl Display for GPUData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GPU Data:")?;
        writeln!(f, "  Energy:      {}", self.total_energy.unwrap_or_default())?;
        writeln!(f, "  Usage:       {:.2} %", self.usage_percent.unwrap_or(-1.0))?;
        writeln!(f, "  VRAM Usage:  {:.2} %", self.vram_usage_percent.unwrap_or(-1.0))?;
        Ok(())
    }
}

impl Display for RamData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RAM Data:")?;
        writeln!(f, "  Energy: {}", self.total_energy.unwrap_or_default())?;
        writeln!(f, "  Usage:  {:.2} %", self.usage_percent.unwrap_or(-1.0))?;
        Ok(())
    }
}

impl Display for DiskData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Disk Data:")?;
        writeln!(f, "  Energy: {}", self.total_energy.unwrap_or_default())?;
        writeln!(f, "  Read:   {:.2} MB", self.read_bytes.as_mb())?;
        writeln!(f, "  Write:  {:.2} MB", self.written_bytes.as_mb())?;
        Ok(())
    }
}

impl Display for NetworkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Network Data:")?;
        writeln!(f, "  Energy:   {}", self.total_energy.unwrap_or_default())?;
        writeln!(f, "  Download: {:.2} MB", self.downloaded_bytes.as_mb())?;
        writeln!(f, "  Upload:   {:.2} MB", self.uploaded_bytes.as_mb())?;
        Ok(())
    }
}

impl From<SensorData> for ComputedSensorData {
    fn from(data: SensorData) -> Self {
        match data {
            SensorData::CPU(cpu) => ComputedSensorData::CPU(cpu),
            SensorData::GPU(gpu) => ComputedSensorData::GPU(gpu),
            SensorData::Ram(ram) => ComputedSensorData::Ram(ram),
            SensorData::Disk(disk) => ComputedSensorData::Disk(disk),
            SensorData::Network(network) => ComputedSensorData::Network(network),
            SensorData::Process(processes) => {
                let computed_processes = processes
                    .into_iter()
                    .map(|p| ProcessData {
                        measured: p,
                        ..Default::default()
                    })
                    .collect();
                ComputedSensorData::Process(computed_processes)
            }
        }
    }
}

impl From<CPUData> for SensorData {
    fn from(data: CPUData) -> Self {
        SensorData::CPU(data)
    }
}

impl From<CPUData> for ComputedSensorData {
    fn from(data: CPUData) -> Self {
        ComputedSensorData::CPU(data)
    }
}

impl From<GPUData> for SensorData {
    fn from(data: GPUData) -> Self {
        SensorData::GPU(data)
    }
}

impl From<GPUData> for ComputedSensorData {
    fn from(data: GPUData) -> Self {
        ComputedSensorData::GPU(data)
    }
}

impl From<TotalData> for ComputedSensorData {
    fn from(data: TotalData) -> Self {
        ComputedSensorData::Total(data)
    }
}

impl From<RamData> for SensorData {
    fn from(data: RamData) -> Self {
        SensorData::Ram(data)
    }
}

impl From<RamData> for ComputedSensorData {
    fn from(data: RamData) -> Self {
        ComputedSensorData::Ram(data)
    }
}

impl From<DiskData> for SensorData {
    fn from(data: DiskData) -> Self {
        SensorData::Disk(data)
    }
}

impl From<DiskData> for ComputedSensorData {
    fn from(data: DiskData) -> Self {
        ComputedSensorData::Disk(data)
    }
}

impl From<NetworkData> for SensorData {
    fn from(data: NetworkData) -> Self {
        SensorData::Network(data)
    }
}

impl From<NetworkData> for ComputedSensorData {
    fn from(data: NetworkData) -> Self {
        ComputedSensorData::Network(data)
    }
}

impl From<MeasuredProcessData> for SensorData {
    fn from(data: MeasuredProcessData) -> Self {
        if data.app_name.is_empty() {
            SensorData::Process(Vec::new())
        } else {
            SensorData::Process(vec![data])
        }
    }
}

impl From<ProcessData> for ComputedSensorData {
    fn from(data: ProcessData) -> Self {
        ComputedSensorData::Process(vec![data])
    }
}

impl Default for CPUData {
    fn default() -> Self {
        CPUData {
            total_energy: Some(EnergyUj(0)),
            pp0_energy: Some(EnergyUj(0)),
            pp1_energy: Some(EnergyUj(0)),
            dram_energy: Some(EnergyUj(0)),
            usage_percent: Some(0.0),
        }
    }
}

impl Default for GPUData {
    fn default() -> Self {
        GPUData {
            total_energy: Some(EnergyUj(0)),
            usage_percent: Some(0.0),
            vram_usage_percent: Some(0.0),
            name: None,
        }
    }
}

impl Default for RamData {
    fn default() -> Self {
        RamData {
            total_energy: Some(EnergyUj(0)),
            usage_percent: Some(0.0),
        }
    }
}

impl Default for DiskData {
    fn default() -> Self {
        DiskData {
            total_energy: Some(EnergyUj(0)),
            read_bytes: Byte(0),
            written_bytes: Byte(0),
        }
    }
}

impl Default for NetworkData {
    fn default() -> Self {
        NetworkData {
            total_energy: Some(EnergyUj(0)),
            downloaded_bytes: Byte(0),
            uploaded_bytes: Byte(0),
        }
    }
}

impl Default for MeasuredProcessData {
    fn default() -> Self {
        MeasuredProcessData {
            pid: None,
            app_name: String::new(),
            process_exe_path: None,
            process_cpu_usage: 0.0,
            process_gpu_usage: None,
            process_mem_usage: 0.0,
            read_bytes: Byte(0),
            written_bytes: Byte(0),
        }
    }
}

impl Default for ProcessData {
    fn default() -> Self {
        ProcessData {
            measured: MeasuredProcessData::default(),
            process_energy: EnergyUj(0),
            subprocess_count: 0,
            icon: None,
        }
    }
}

impl Default for TotalData {
    fn default() -> Self {
        TotalData {
            total_energy: EnergyUj(0),
        }
    }
}
