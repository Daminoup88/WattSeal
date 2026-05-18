use crate::{
    AllTimeData, CPUData, DatabaseEntry, DiskData, GPUData, GeneralData, NetworkData, ProcessData, RamData, SensorData,
    SensorKind, TotalData, types::Event,
};

pub type PowerUnit = f64; // Power unit in Watts (TODO Move it in a energy module)

/// Sensors data for database
pub type EventDB = Event<PowerUnit>;
pub type CPUDataDB = CPUData<PowerUnit>;
pub type GPUDataDB = GPUData<PowerUnit>;
pub type RamDataDB = RamData<PowerUnit>;
pub type DiskDataDB = DiskData<PowerUnit>;
pub type NetworkDataDB = NetworkData<PowerUnit>;
pub type ProcessDataDB = ProcessData<PowerUnit>;
pub type TotalDataDB = TotalData<PowerUnit>;
pub type SensorDataDB = SensorData<PowerUnit>;
pub type AllTimeDataDB = AllTimeData<PowerUnit>;

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
