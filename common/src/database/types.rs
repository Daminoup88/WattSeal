use std::fmt::Display;

use crate::{
    AllTimeData, CPUData, DatabaseEntry, DiskData, GPUData, GeneralData, NetworkData, ProcessData, RamData, SensorData,
    SensorKind, TotalData, types::Event,
};

pub type PowerWatt = f64;

/// Sensors data for database
pub type EventDB = Event<PowerWatt>;
pub type CPUDataDB = CPUData<PowerWatt>;
pub type GPUDataDB = GPUData<PowerWatt>;
pub type RamDataDB = RamData<PowerWatt>;
pub type DiskDataDB = DiskData<PowerWatt>;
pub type NetworkDataDB = NetworkData<PowerWatt>;
pub type ProcessDataDB = ProcessData<PowerWatt>;
pub type TotalDataDB = TotalData<PowerWatt>;
pub type SensorDataDB = SensorData<PowerWatt>;
pub type AllTimeDataDB = AllTimeData<PowerWatt>;

#[derive(Debug)]
pub struct GeneralDataDB {
    pub tables: String,
    pub hardware_info_serialized: String,
}

impl From<GeneralData> for GeneralDataDB {
    fn from(data: GeneralData) -> Self {
        let tables: Vec<&str> = data.sensors.iter().map(|s| s.table_name()).collect();
        Self {
            tables: tables.join(","),
            hardware_info_serialized: data.hardware_info.serialized(),
        }
    }
}

impl SensorDataDB {
    /// Returns the database table name for this variant.
    pub fn table_name(&self) -> &'static str {
        match self {
            SensorDataDB::CPU(_) => CPUDataDB::table_name_static(),
            SensorDataDB::GPU(_) => GPUDataDB::table_name_static(),
            SensorDataDB::Total(_) => TotalDataDB::table_name_static(),
            SensorDataDB::Ram(_) => RamDataDB::table_name_static(),
            SensorDataDB::Disk(_) => DiskDataDB::table_name_static(),
            SensorDataDB::Network(_) => NetworkDataDB::table_name_static(),
            SensorDataDB::Process(_) => ProcessDataDB::table_name_static(),
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

    /// Returns the secondary metric kind for this sensor variant.
    pub fn secondary_metric(&self) -> Option<MetricKindDB> {
        match self {
            SensorData::CPU(_) | SensorData::GPU(_) | SensorData::Ram(_) => Some(MetricKindDB::Usage),
            SensorData::Disk(_) | SensorData::Network(_) => Some(MetricKindDB::Speed),
            _ => None,
        }
    }

    pub fn power_to_energy(&mut self, factor: f64) {
        match self {
            SensorData::CPU(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::GPU(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Ram(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Disk(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Network(d) => d.total_consumption = d.total_consumption.clone().map(|w| w * factor),
            SensorData::Total(d) => {
                d.total_consumption *= factor;
            }
            SensorData::Process(procs) => {
                for p in procs {
                    p.process_consumption *= factor;
                }
            }
        }
    }
}

impl SensorKind {
    pub fn table_name(&self) -> &'static str {
        match self {
            SensorKind::CPU => CPUDataDB::table_name_static(),
            SensorKind::GPU => GPUDataDB::table_name_static(),
            SensorKind::Total => TotalDataDB::table_name_static(),
            SensorKind::Ram => RamDataDB::table_name_static(),
            SensorKind::Disk => DiskDataDB::table_name_static(),
            SensorKind::Network => NetworkDataDB::table_name_static(),
            SensorKind::Process => ProcessDataDB::table_name_static(),
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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum MetricKindDB {
    #[default]
    Power,
    Usage,
    Speed,
}

impl Display for MetricKindDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricKindDB::Power => write!(f, "Power"),
            MetricKindDB::Usage => write!(f, "Usage"),
            MetricKindDB::Speed => write!(f, "Speed"),
        }
    }
}

impl MetricKindDB {
    /// Returns the human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            MetricKindDB::Power => "Power",
            MetricKindDB::Usage => "Usage",
            MetricKindDB::Speed => "Speed",
        }
    }

    /// Returns the measurement unit string.
    pub fn unit_label(&self) -> &'static str {
        match self {
            MetricKindDB::Power => "W",
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
        if *self == MetricKindDB::Power {
            if energy_mode { "W" } else { "Wh" }
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
