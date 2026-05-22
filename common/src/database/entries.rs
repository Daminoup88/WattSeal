use std::collections::HashMap;

use rusqlite::{Row, ToSql};

use crate::database::types::{
    AllTimeDataDB, CPUDataDB, DataDB, DiskDataDB, GPUDataDB, NetworkDataDB, ProcessDataDB, RamDataDB, TotalDataDB,
};

/// Maps a data type to its SQLite table schema and row conversion.
pub trait DatabaseEntry {
    fn generic_name() -> &'static str;
    fn table_name_static() -> &'static str;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
    fn columns_static() -> &'static [(&'static str, &'static str)];
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;

    fn zero() -> DataDB
    where
        Self: Default + Into<DataDB>,
    {
        Self::default().into()
    }

    fn insert_sql() -> String {
        let cols = Self::columns_static();
        let col_names: Vec<&str> = cols.iter().map(|(name, _)| *name).collect();
        let all_cols = format!("timestamp_id, {}", col_names.join(", "));
        let params: Vec<String> = (1..=cols.len() + 1).map(|i| format!("?{}", i)).collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::table_name_static(),
            all_cols,
            params.join(", ")
        )
    }

    fn create_table_sql() -> String {
        let mut col_defs = vec![
            "id INTEGER PRIMARY KEY".to_string(),
            "timestamp_id INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE".to_string(),
        ];
        for (name, type_) in Self::columns_static() {
            col_defs.push(format!("{} {}", name, type_));
        }
        let table_name = Self::table_name_static();
        format!(
            "CREATE TABLE IF NOT EXISTS {} ({});\
             CREATE INDEX IF NOT EXISTS idx_{}_timestamp_id ON {}(timestamp_id)",
            table_name,
            col_defs.join(", "),
            table_name,
            table_name,
        )
    }

    fn avg_columns_sql(prefix: &str) -> String {
        Self::columns_static()
            .iter()
            .map(|(col_name, _)| format!("AVG({}{}) AS {}", prefix, col_name, col_name))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl DatabaseEntry for CPUDataDB {
    fn generic_name() -> &'static str {
        "CPU"
    }

    fn table_name_static() -> &'static str {
        "cpu_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_consumption,
            &self.pp0_consumption,
            &self.pp1_consumption,
            &self.dram_consumption,
            &self.usage_percent,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_consumption", "REAL"),
            ("pp0_consumption", "REAL"),
            ("pp1_consumption", "REAL"),
            ("dram_consumption", "REAL"),
            ("usage_percent", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(CPUDataDB {
            total_consumption: row.get("total_consumption")?,
            pp0_consumption: row.get("pp0_consumption")?,
            pp1_consumption: row.get("pp1_consumption")?,
            dram_consumption: row.get("dram_consumption")?,
            usage_percent: row.get("usage_percent")?,
        })
    }
}

impl DatabaseEntry for GPUDataDB {
    fn generic_name() -> &'static str {
        "GPU"
    }

    fn table_name_static() -> &'static str {
        "gpu_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_consumption,
            &self.usage_percent,
            &self.vram_usage_percent,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_consumption", "REAL"),
            ("usage_percent", "REAL"),
            ("vram_usage_percent", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(GPUDataDB {
            total_consumption: row.get("total_consumption")?,
            usage_percent: row.get("usage_percent")?,
            vram_usage_percent: row.get("vram_usage_percent")?,
        })
    }
}

impl DatabaseEntry for DiskDataDB {
    fn generic_name() -> &'static str {
        "Disk"
    }

    fn table_name_static() -> &'static str {
        "disk_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_consumption,
            &self.read_usage_mb_s,
            &self.write_usage_mb_s,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_consumption", "REAL"),
            ("read_usage_mb_s", "REAL"),
            ("write_usage_mb_s", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(DiskDataDB {
            total_consumption: row.get("total_consumption")?,
            read_usage_mb_s: row.get("read_usage_mb_s")?,
            write_usage_mb_s: row.get("write_usage_mb_s")?,
        })
    }
}

impl DatabaseEntry for RamDataDB {
    fn generic_name() -> &'static str {
        "RAM"
    }

    fn table_name_static() -> &'static str {
        "ram_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![timestamp_id, &self.total_consumption, &self.usage_percent]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[("total_consumption", "REAL"), ("usage_percent", "REAL")]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(RamDataDB {
            total_consumption: row.get("total_consumption")?,
            usage_percent: row.get("usage_percent")?,
        })
    }
}

impl DatabaseEntry for NetworkDataDB {
    fn generic_name() -> &'static str {
        "Network"
    }

    fn table_name_static() -> &'static str {
        "network_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_consumption,
            &self.download_speed_mb_s,
            &self.upload_speed_mb_s,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_consumption", "REAL"),
            ("download_speed_mb_s", "REAL"),
            ("upload_speed_mb_s", "REAL"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(NetworkDataDB {
            total_consumption: row.get("total_consumption")?,
            download_speed_mb_s: row.get("download_speed_mb_s")?,
            upload_speed_mb_s: row.get("upload_speed_mb_s")?,
        })
    }
}

impl DatabaseEntry for TotalDataDB {
    fn generic_name() -> &'static str {
        "Total"
    }

    fn table_name_static() -> &'static str {
        "total_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![timestamp_id, &self.total_consumption, &self.period_type]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[("total_consumption", "REAL"), ("period_type", "TEXT NOT NULL")]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(TotalDataDB {
            total_consumption: row.get("total_consumption")?,
            period_type: row.get("period_type")?,
        })
    }
}

impl DatabaseEntry for ProcessDataDB {
    fn generic_name() -> &'static str {
        "Processes"
    }

    fn table_name_static() -> &'static str {
        "process_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.app_name,
            &self.process_exe_path,
            &self.process_consumption,
            &self.process_cpu_usage,
            &self.process_gpu_usage,
            &self.process_mem_usage,
            &self.read_bytes_per_sec,
            &self.written_bytes_per_sec,
            &self.subprocess_count,
        ]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("app_name", "TEXT NOT NULL"),
            ("process_exe_path", "TEXT"),
            ("process_consumption", "REAL"),
            ("process_cpu_usage", "REAL"),
            ("process_gpu_usage", "REAL"),
            ("process_mem_usage", "REAL"),
            ("read_bytes_per_sec", "REAL"),
            ("written_bytes_per_sec", "REAL"),
            ("subprocess_count", "INTEGER"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(ProcessDataDB {
            app_name: row.get("app_name")?,
            process_exe_path: row.get("process_exe_path")?,
            process_consumption: row.get("process_consumption")?,
            process_cpu_usage: row.get("process_cpu_usage")?,
            process_gpu_usage: row.get("process_gpu_usage")?,
            process_mem_usage: row.get("process_mem_usage")?,
            read_bytes_per_sec: row.get("read_bytes_per_sec")?,
            written_bytes_per_sec: row.get("written_bytes_per_sec")?,
            subprocess_count: row.get("subprocess_count")?,
            icon: None,
        })
    }

    fn zero() -> DataDB {
        DataDB::Process(Vec::new())
    }
}

impl DatabaseEntry for AllTimeDataDB {
    fn generic_name() -> &'static str {
        "AllTime"
    }

    fn table_name_static() -> &'static str {
        "all_time_data"
    }

    fn insert_params<'a>(&'a self, _timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[]
    }

    fn from_row(_: &Row) -> rusqlite::Result<Self> {
        Ok(AllTimeDataDB {
            components: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CPUDataDB, DataDB, DatabaseEntry, DiskDataDB, GPUDataDB, NetworkDataDB, ProcessDataDB, RamDataDB, TotalDataDB,
    };
    use crate::SensorDataDB;

    #[test]
    fn zero_defaults_are_zero_filled() {
        // CPU
        match CPUDataDB::zero() {
            DataDB::Sensor(SensorDataDB::CPU(cpu)) => {
                assert_eq!(cpu.total_consumption, Some(0.0));
                assert_eq!(cpu.pp0_consumption, Some(0.0));
                assert_eq!(cpu.pp1_consumption, Some(0.0));
                assert_eq!(cpu.dram_consumption, Some(0.0));
                assert_eq!(cpu.usage_percent, Some(0.0));
            }
            _ => panic!("CPUData::zero() returned wrong SensorData variant"),
        }

        // GPU
        match GPUDataDB::zero() {
            DataDB::Sensor(SensorDataDB::GPU(gpu)) => {
                assert_eq!(gpu.total_consumption, Some(0.0));
                assert_eq!(gpu.usage_percent, Some(0.0));
                assert_eq!(gpu.vram_usage_percent, Some(0.0));
            }
            _ => panic!("GPUData::zero() returned wrong SensorData variant"),
        }

        // RAM
        match RamDataDB::zero() {
            DataDB::Sensor(SensorDataDB::Ram(ram)) => {
                assert_eq!(ram.total_consumption, Some(0.0));
                assert_eq!(ram.usage_percent, Some(0.0));
            }
            _ => panic!("RamData::zero() returned wrong SensorData variant"),
        }

        // Disk
        match DiskDataDB::zero() {
            DataDB::Sensor(SensorDataDB::Disk(disk)) => {
                assert_eq!(disk.total_consumption, Some(0.0));
                assert_eq!(disk.read_usage_mb_s, 0.0);
                assert_eq!(disk.write_usage_mb_s, 0.0);
            }
            _ => panic!("DiskData::zero() returned wrong SensorData variant"),
        }

        // Network
        match NetworkDataDB::zero() {
            DataDB::Sensor(SensorDataDB::Network(net)) => {
                assert_eq!(net.total_consumption, Some(0.0));
                assert_eq!(net.download_speed_mb_s, 0.0);
                assert_eq!(net.upload_speed_mb_s, 0.0);
            }
            _ => panic!("NetworkData::zero() returned wrong SensorData variant"),
        }

        // Total
        match TotalDataDB::zero() {
            DataDB::Total(total) => {
                assert_eq!(total.total_consumption, 0.0);
                assert_eq!(total.period_type, "second");
            }
            _ => panic!("TotalData::zero() returned wrong SensorData variant"),
        }

        // Process
        match ProcessDataDB::zero() {
            DataDB::Process(vec) => {
                assert!(vec.is_empty());
            }
            _ => panic!("ProcessData::zero() returned wrong SensorData variant"),
        }
    }
}
