use rusqlite::{Row, ToSql};

use crate::{
    database,
    types::{CPUData, DiskData, GPUData, NetworkData, ProcessData, RamData, SensorData, TotalData},
};

pub trait DatabaseEntry {
    fn generic_name() -> &'static str;
    fn table_name_static() -> &'static str;
    fn insert_sql(&self) -> String;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
    fn columns_static() -> &'static [(&'static str, &'static str)];
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;
}

impl DatabaseEntry for SensorData {
    fn generic_name() -> &'static str {
        "Sensor"
    }

    fn table_name_static() -> &'static str {
        "sensor_data"
    }

    fn insert_sql(&self) -> String {
        match self {
            SensorData::CPU(data) => data.insert_sql(),
            SensorData::GPU(data) => data.insert_sql(),
            SensorData::Ram(data) => data.insert_sql(),
            SensorData::Disk(data) => data.insert_sql(),
            SensorData::Network(data) => data.insert_sql(),
            SensorData::Total(data) => data.insert_sql(),
            SensorData::Process(_) => "".to_string(),
        }
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        match self {
            SensorData::CPU(data) => data.insert_params(timestamp_id),
            SensorData::GPU(data) => data.insert_params(timestamp_id),
            SensorData::Ram(data) => data.insert_params(timestamp_id),
            SensorData::Disk(data) => data.insert_params(timestamp_id),
            SensorData::Network(data) => data.insert_params(timestamp_id),
            SensorData::Total(data) => data.insert_params(timestamp_id),
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
    fn generic_name() -> &'static str {
        "CPU"
    }

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
            ("usage_percent", "REAL"),
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
    fn generic_name() -> &'static str {
        "GPU"
    }

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

impl DatabaseEntry for DiskData {
    fn generic_name() -> &'static str {
        "Disk"
    }

    fn table_name_static() -> &'static str {
        "disk_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, total_power_watts, read_usage_mb_s, write_usage_mb_s) VALUES (?1, ?2, ?3, ?4)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.read_usage_mb_s,
            &self.write_usage_mb_s,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_power_watts", "REAL"),
            ("read_usage_mb_s", "REAL"),
            ("write_usage_mb_s", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(DiskData {
            total_power_watts: row.get("total_power_watts")?,
            read_usage_mb_s: row.get("read_usage_mb_s")?,
            write_usage_mb_s: row.get("write_usage_mb_s")?,
        })
    }
}

impl DatabaseEntry for RamData {
    fn generic_name() -> &'static str {
        "RAM"
    }

    fn table_name_static() -> &'static str {
        "ram_data"
    }

    fn insert_sql(&self) -> String {
        format!(
            "INSERT INTO {} (timestamp_id, total_power_watts, usage_percent) VALUES (?1, ?2, ?3)",
            Self::table_name_static()
        )
        .to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![timestamp_id, &self.total_power_watts, &self.usage_percent]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[("total_power_watts", "REAL"), ("usage_percent", "REAL")]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(RamData {
            total_power_watts: row.get("total_power_watts")?,
            usage_percent: row.get("usage_percent")?,
        })
    }
}

impl DatabaseEntry for NetworkData {
    fn generic_name() -> &'static str {
        "Network"
    }

    fn table_name_static() -> &'static str {
        "network_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, total_power_watts, download_speed_mb_s, upload_speed_mb_s) VALUES (?1, ?2, ?3, ?4)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.download_speed_mb_s,
            &self.upload_speed_mb_s,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_power_watts", "REAL"),
            ("download_speed_mb_s", "REAL"),
            ("upload_speed_mb_s", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(NetworkData {
            total_power_watts: row.get("total_power_watts")?,
            download_speed_mb_s: row.get("download_speed_mb_s")?,
            upload_speed_mb_s: row.get("upload_speed_mb_s")?,
        })
    }
}

impl DatabaseEntry for TotalData {
    fn generic_name() -> &'static str {
        "Total"
    }

    fn table_name_static() -> &'static str {
        "total_data"
    }

    fn insert_sql(&self) -> String {
        format!(
            "INSERT INTO {} (timestamp_id, total_power_watts, period_type) VALUES (?1, ?2, ?3)",
            Self::table_name_static()
        )
        .to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![timestamp_id, &self.total_power_watts, &self.period_type]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[("total_power_watts", "REAL"), ("period_type", "TEXT NOT NULL")]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(TotalData {
            total_power_watts: row.get("total_power_watts")?,
            period_type: row.get("period_type")?,
        })
    }
}

impl DatabaseEntry for ProcessData {
    fn generic_name() -> &'static str {
        "Process"
    }

    fn table_name_static() -> &'static str {
        "process_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, app_name, vram_usage, cpu_usage_watts, subprocess_count) VALUES (?1, ?2, ?3, ?4, ?5)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.app_name,
            &self.vram_usage,
            &self.cpu_usage_watts,
            &(self.subprocess_count),
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("app_name", "TEXT NOT NULL"),
            ("vram_usage", "REAL"),
            ("cpu_usage_watts", "REAL"),
            ("subprocess_count", "INTEGER"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(ProcessData {
            app_name: row.get("app_name")?,
            vram_usage: row.get("vram_usage")?,
            cpu_usage_watts: row.get("cpu_usage_watts")?,
            subprocess_count: row.get::<_, i64>("subprocess_count")? as u32,
        })
    }
}
