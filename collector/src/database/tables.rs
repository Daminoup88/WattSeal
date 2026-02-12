use std::time;

use common::{DatabaseTable, ProcessData};

use super::{CPUData, DatabaseEntry, DiskData, GPUData, NetworkData, RamData, SensorData, TotalData};
use crate::sensors::{CPUSensor, DiskSensor, GPUSensor, NetworkSensor, RamSensor, SensorType};

impl DatabaseTable for SensorType {
    fn table_name(&self) -> &'static str {
        match self {
            SensorType::CPU(s) => s.table_name(),
            SensorType::GPU(s) => s.table_name(),
            SensorType::RAM(s) => s.table_name(),
            SensorType::Disk(s) => s.table_name(),
            SensorType::Network(s) => s.table_name(),
            SensorType::Total => TotalData::table_name_static(),
            SensorType::Process => "process_data",
        }
    }

    fn columns(&self) -> Vec<String> {
        match self {
            SensorType::CPU(s) => s.columns(),
            SensorType::GPU(s) => s.columns(),
            SensorType::RAM(s) => s.columns(),
            SensorType::Disk(s) => s.columns(),
            SensorType::Network(s) => s.columns(),
            SensorType::Total => {
                let mut cols = timestamp_columns();
                for (name, type_) in TotalData::columns_static() {
                    cols.push(format!("{} {}", name, type_));
                }
                cols
            }
            SensorType::Process => {
                let mut cols = timestamp_columns();
                cols.push("app_name TEXT NOT NULL".to_string());
                cols.push("cpu_usage_watts REAL NOT NULL".to_string());
                cols.push("vram_usage REAL NOT NULL".to_string());
                cols.push("subprocess_count INTEGER NOT NULL".to_string());
                cols
            }
        }
    }
}

fn timestamp_columns() -> Vec<String> {
    vec![
        "id           INTEGER PRIMARY KEY".to_string(),
        "timestamp_id INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE".to_string(),
    ]
}

impl DatabaseTable for CPUSensor {
    fn table_name(&self) -> &'static str {
        CPUData::table_name_static()
    }

    fn columns(&self) -> Vec<String> {
        let mut cols = timestamp_columns();
        for (name, type_) in CPUData::columns_static() {
            cols.push(format!("{} {}", name, type_));
        }
        cols
    }
}

impl DatabaseTable for GPUSensor {
    fn table_name(&self) -> &'static str {
        GPUData::table_name_static()
    }

    fn columns(&self) -> Vec<String> {
        let mut cols = timestamp_columns();
        for (name, type_) in GPUData::columns_static() {
            cols.push(format!("{} {}", name, type_));
        }
        cols
    }
}

impl DatabaseTable for RamSensor {
    fn table_name(&self) -> &'static str {
        "ram_data"
    }
    fn columns(&self) -> Vec<String> {
        let mut cols = timestamp_columns();
        for (name, type_) in RamData::columns_static() {
            cols.push(format!("{} {}", name, type_));
        }
        cols
    }
}

impl DatabaseTable for DiskSensor {
    fn table_name(&self) -> &'static str {
        "disk_data"
    }
    fn columns(&self) -> Vec<String> {
        let mut cols = timestamp_columns();
        for (name, type_) in DiskData::columns_static() {
            cols.push(format!("{} {}", name, type_));
        }
        cols
    }
}

impl DatabaseTable for NetworkSensor {
    fn table_name(&self) -> &'static str {
        "network_data"
    }
    fn columns(&self) -> Vec<String> {
        let mut cols = timestamp_columns();
        for (name, type_) in NetworkData::columns_static() {
            cols.push(format!("{} {}", name, type_));
        }
        cols
    }
}
