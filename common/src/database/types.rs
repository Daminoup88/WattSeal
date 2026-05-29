use std::{fmt::Display, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::{
    AllTimeData, CPUData, DatabaseEntry, DiskData, EnergyUJ, EnergyWH, GPUData, GeneralData, NetworkData, PowerWatt,
    RamData, SensorData, SensorKind,
};

/// Sensors data for database
pub struct EventDB {
    time: SystemTime,
    data: Vec<DataDB>,
}

pub type CPUDataDB = CPUData<EnergyWH>;
pub type GPUDataDB = GPUData<EnergyWH>;
pub type RamDataDB = RamData<EnergyWH>;
pub type DiskDataDB = DiskData<EnergyWH>;
pub type NetworkDataDB = NetworkData<EnergyWH>;
pub type SensorDataDB = SensorData<EnergyWH>;
pub type AllTimeDataDB = AllTimeData<EnergyWH>;

// Sensors dependant data used in database.
#[derive(Debug, Clone)]
pub enum DataDB {
    Sensor(SensorDataDB),
    Process(Vec<ProcessDataDB>),
    Total(TotalDataDB),
}

/// Raw RGBA icon pixel data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Per-application resource usage snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDataDB {
    pub app_name: String,
    pub process_exe_path: Option<String>,
    pub process_consumption: EnergyWH,
    pub process_cpu_usage: f64,
    pub process_gpu_usage: Option<f64>,
    pub process_mem_usage: f64,
    pub read_bytes_per_sec: f64,
    pub written_bytes_per_sec: f64,
    pub subprocess_count: u32,
    pub icon: Option<IconData>,
}

/// Aggregated total consumption across all components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalDataDB {
    pub total_consumption: EnergyWH,
    pub period_type: String,
}

#[derive(Debug)]
pub struct GeneralDataDB {
    pub tables: String,
    pub hardware_info_serialized: String,
}

/// Different kind of sensors dependent data kind.
#[derive(Debug, Clone)]
pub enum DataKindDB {
    CPU,
    GPU,
    Ram,
    Disk,
    Network,
    Process,
    Total,
}

impl EventDB {
    /// Creates an event with the given timestamp and data.
    pub fn new(time: SystemTime, data: Vec<DataDB>) -> Self {
        EventDB { time, data }
    }

    /// Returns the event timestamp.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the list of sensor readings.
    pub fn data(&self) -> &Vec<DataDB> {
        &self.data
    }

    /// Appends a sensor reading to this event.
    pub fn push_data(&mut self, data: DataDB) {
        self.data.push(data);
    }
}

impl From<SensorKind> for DataKindDB {
    fn from(sk: SensorKind) -> Self {
        match sk {
            SensorKind::CPU => DataKindDB::CPU,
            SensorKind::GPU => DataKindDB::GPU,
            SensorKind::Ram => DataKindDB::Ram,
            SensorKind::Disk => DataKindDB::Disk,
            SensorKind::Network => DataKindDB::Network,
        }
    }
}

impl From<CPUDataDB> for DataDB {
    fn from(data: CPUDataDB) -> Self {
        DataDB::Sensor(SensorDataDB::CPU(data))
    }
}

impl From<GPUDataDB> for DataDB {
    fn from(data: GPUDataDB) -> Self {
        DataDB::Sensor(SensorData::GPU(data))
    }
}

impl From<RamDataDB> for DataDB {
    fn from(data: RamDataDB) -> Self {
        DataDB::Sensor(SensorData::Ram(data))
    }
}
impl From<DiskDataDB> for DataDB {
    fn from(data: DiskDataDB) -> Self {
        DataDB::Sensor(SensorData::Disk(data))
    }
}
impl From<NetworkDataDB> for DataDB {
    fn from(data: NetworkDataDB) -> Self {
        DataDB::Sensor(SensorData::Network(data))
    }
}

impl From<ProcessDataDB> for DataDB {
    fn from(data: ProcessDataDB) -> Self {
        DataDB::Process(vec![data])
    }
}

impl From<TotalDataDB> for DataDB {
    fn from(data: TotalDataDB) -> Self {
        DataDB::Total(data)
    }
}

impl From<&SensorData<EnergyUJ>> for DataDB {
    fn from(sensor_data: &SensorData<EnergyUJ>) -> Self {
        DataDB::Sensor(sensor_data.to_wh())
    }
}

impl From<GeneralData> for GeneralDataDB {
    fn from(data: GeneralData) -> Self {
        let mut tables: Vec<&str> = data
            .sensors
            .iter()
            .map(|s| DataKindDB::from(s.clone()).table_name())
            .collect();

        tables.push(ProcessDataDB::table_name_static());
        tables.push(TotalDataDB::table_name_static());

        Self {
            tables: tables.join(","),
            hardware_info_serialized: data.hardware_info.serialized(),
        }
    }
}

impl DataDB {
    /// Returns the total consumption value, if available.
    pub fn total_consumption(&self) -> Option<PowerWatt> {
        match self {
            DataDB::Sensor(s) => s.total_consumption(),
            DataDB::Total(power) => Some(power.total_consumption.clone()),
            DataDB::Process(_) => None,
        }
    }

    /// Returns the database table name for this variant.
    pub fn table_name(&self) -> &'static str {
        match self {
            DataDB::Sensor(SensorDataDB::CPU(_)) => CPUDataDB::table_name_static(),
            DataDB::Sensor(SensorDataDB::GPU(_)) => GPUDataDB::table_name_static(),
            DataDB::Sensor(SensorDataDB::Ram(_)) => RamDataDB::table_name_static(),
            DataDB::Sensor(SensorDataDB::Disk(_)) => DiskDataDB::table_name_static(),
            DataDB::Sensor(SensorDataDB::Network(_)) => NetworkDataDB::table_name_static(),
            DataDB::Process(_) => ProcessDataDB::table_name_static(),
            DataDB::Total(_) => TotalDataDB::table_name_static(),
        }
    }

    /// Returns secondary metrics (usage or speed) if applicable.
    pub fn secondary_values(&self) -> Option<SecondaryValues> {
        let metric_type = self.secondary_metric()?;
        if let DataDB::Sensor(s) = self {
            match s {
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
            }
        } else {
            None
        }
    }

    /// Returns the secondary metric kind for this sensor variant.
    pub fn secondary_metric(&self) -> Option<MetricKindDB> {
        if let DataDB::Sensor(s) = self {
            match s {
                SensorData::CPU(_) | SensorData::GPU(_) | SensorData::Ram(_) => Some(MetricKindDB::Usage),
                SensorData::Disk(_) | SensorData::Network(_) => Some(MetricKindDB::Speed),
            }
        } else {
            None
        }
    }

    pub fn power_to_energy(&mut self, factor: f64) {
        match self {
            DataDB::Sensor(SensorData::CPU(d)) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            DataDB::Sensor(SensorData::GPU(d)) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            DataDB::Sensor(SensorData::Ram(d)) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            DataDB::Sensor(SensorData::Disk(d)) => {
                d.total_consumption = d.total_consumption.clone().map(|w| w * factor)
            }
            DataDB::Sensor(SensorData::Network(d)) => {
                d.total_consumption = d.total_consumption.clone().map(|w| w * factor)
            }
            DataDB::Total(d) => {
                d.total_consumption *= factor;
            }
            DataDB::Process(procs) => {
                for p in procs {
                    p.process_consumption *= factor;
                }
            }
        }
    }
}

impl DataKindDB {
    pub fn table_name(&self) -> &'static str {
        match self {
            DataKindDB::CPU => CPUDataDB::table_name_static(),
            DataKindDB::GPU => GPUDataDB::table_name_static(),
            DataKindDB::Total => TotalDataDB::table_name_static(),
            DataKindDB::Ram => RamDataDB::table_name_static(),
            DataKindDB::Disk => DiskDataDB::table_name_static(),
            DataKindDB::Network => NetworkDataDB::table_name_static(),
            DataKindDB::Process => ProcessDataDB::table_name_static(),
        }
    }
}

impl Default for CPUDataDB {
    fn default() -> Self {
        CPUDataDB {
            total_consumption: Some(0.0),
            pp0_consumption: Some(0.0),
            pp1_consumption: Some(0.0),
            dram_consumption: Some(0.0),
            usage_percent: Some(0.0),
        }
    }
}

impl Default for GPUDataDB {
    fn default() -> Self {
        GPUDataDB {
            total_consumption: Some(0.0),
            usage_percent: Some(0.0),
            vram_usage_percent: Some(0.0),
        }
    }
}

impl Default for RamDataDB {
    fn default() -> Self {
        RamDataDB {
            total_consumption: Some(0.0),
            usage_percent: Some(0.0),
        }
    }
}

impl Default for DiskDataDB {
    fn default() -> Self {
        DiskDataDB {
            total_consumption: Some(0.0),
            read_usage_mb_s: 0.0,
            write_usage_mb_s: 0.0,
        }
    }
}

impl Default for NetworkDataDB {
    fn default() -> Self {
        NetworkDataDB {
            total_consumption: Some(0.0),
            download_speed_mb_s: 0.0,
            upload_speed_mb_s: 0.0,
        }
    }
}

impl Default for ProcessDataDB {
    fn default() -> Self {
        ProcessDataDB {
            app_name: String::new(),
            process_exe_path: None,
            process_consumption: 0.0,
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

impl Default for TotalDataDB {
    fn default() -> Self {
        TotalDataDB {
            total_consumption: 0.0,
            period_type: "second".to_string(),
        }
    }
}

impl Display for DataDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sensor(s) => {
                writeln!(f, "{}", s)?;
                Ok(())
            }
            Self::Process(p) => {
                writeln!(f, "Top Processes by CPU Usage:")?;
                writeln!(
                    f,
                    "{:<30} {:>10} {:>10} {:>10} {:>10} {:>15} {:>15} {:>20}",
                    "App Name", "CPU %", "GPU %", "Mem %", "Power W", "Read MB/s", "Write MB/s", "Subprocesses"
                )?;
                for process in p {
                    write!(f, "{}", process)?;
                }
                Ok(())
            }
            Self::Total(t) => {
                writeln!(f, "{}", t)?;
                Ok(())
            }
        }
    }
}

impl Display for ProcessDataDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:<30} {:>10.2} {:>10} {:>10.2} {:>10} W {:>15.2} {:>15.2} {:>20}",
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

impl Display for TotalDataDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Total Power during 1 {}: {:.3} W",
            self.period_type, self.total_consumption
        )?;
        Ok(())
    }
}

/// Different kind of metric in database.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum MetricKindDB {
    #[default]
    Energy,
    Usage,
    Speed,
}

impl Display for MetricKindDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricKindDB::Energy => write!(f, "Energy"),
            MetricKindDB::Usage => write!(f, "Usage"),
            MetricKindDB::Speed => write!(f, "Speed"),
        }
    }
}

impl MetricKindDB {
    /// Returns the human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            MetricKindDB::Energy => "Energy",
            MetricKindDB::Usage => "Usage",
            MetricKindDB::Speed => "Speed",
        }
    }

    /// Returns the measurement unit string.
    pub fn unit_label(&self) -> &'static str {
        match self {
            MetricKindDB::Energy => "Wh",
            MetricKindDB::Usage => "%",
            MetricKindDB::Speed => "MB/s",
        }
    }

    /// Formats a chart legend label for the given component.
    pub fn legend(&self, component_name: &str) -> String {
        format!("{} {}", component_name, self.label())
    }

    /// Returns the display unit, swapping uj for Wh when energy mode, and W otherwise.
    pub fn effective_unit(&self, energy_mode: bool) -> &'static str {
        if *self == MetricKindDB::Energy {
            if energy_mode { "Wh" } else { "W" }
        } else {
            self.unit_label()
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
    pub metric_kind: MetricKindDB,
    pub values: Vec<LabeledValue>,
}

impl SecondaryValues {
    fn from_labeled_values(metric_kind: MetricKindDB, values: Vec<LabeledValue>) -> Self {
        Self { metric_kind, values }
    }

    /// Returns the list of labeled values.
    pub fn values(&self) -> &Vec<LabeledValue> {
        &self.values
    }

    /// Returns the metric type of these secondary values.
    pub fn metric_type(&self) -> MetricKindDB {
        self.metric_kind
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
