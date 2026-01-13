use rusqlite::{Row, ToSql};

use crate::types::{CPUData, GPUData, SensorData};

pub trait DatabaseEntry {
    fn table_name_static() -> &'static str;
    fn insert_sql(&self) -> String;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
    fn columns_static() -> &'static [(&'static str, &'static str)];
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;
}

impl DatabaseEntry for SensorData {
    fn table_name_static() -> &'static str {
        "sensor_data"
    }

    fn insert_sql(&self) -> String {
        match self {
            SensorData::CPU(data) => data.insert_sql(),
            SensorData::GPU(data) => data.insert_sql(),
            _ => "".to_string(),
        }
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        match self {
            SensorData::CPU(data) => data.insert_params(timestamp_id),
            SensorData::GPU(data) => data.insert_params(timestamp_id),
            _ => vec![],
        }
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[]
    }

    fn from_row(_row: &Row) -> rusqlite::Result<Self> {
        Err(rusqlite::Error::InvalidQuery)
    }
}

impl DatabaseEntry for CPUData {
    fn table_name_static() -> &'static str {
        "cpu_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.pp0_power_watts,
            &self.pp1_power_watts,
            &self.dram_power_watts,
            &self.usage_percent,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_power_watts", "REAL"),
            ("pp0_power_watts", "REAL"),
            ("pp1_power_watts", "REAL"),
            ("dram_power_watts", "REAL"),
            ("usage_percent", "REAL NOT NULL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(CPUData {
            total_power_watts: row.get("total_power_watts")?,
            pp0_power_watts: row.get("pp0_power_watts")?,
            pp1_power_watts: row.get("pp1_power_watts")?,
            dram_power_watts: row.get("dram_power_watts")?,
            usage_percent: row.get("usage_percent")?,
        })
    }
}

impl DatabaseEntry for GPUData {
    fn table_name_static() -> &'static str {
        "gpu_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, total_power_watts, usage_percent, vram_usage_percent) VALUES (?1, ?2, ?3, ?4)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.usage_percent,
            &self.vram_usage_percent,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_power_watts", "REAL"),
            ("usage_percent", "REAL"),
            ("vram_usage_percent", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(GPUData {
            total_power_watts: row.get("total_power_watts")?,
            usage_percent: row.get("usage_percent")?,
            vram_usage_percent: row.get("vram_usage_percent")?,
        })
    }
}
