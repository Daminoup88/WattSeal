use rusqlite::{Row, ToSql};

use crate::{
    types::{
        AllTimeData, CPUData, DiskData, GPUData, GeneralData, NetworkData, ProcessData, RamData, SensorData, TotalData,
    },
    utils::load_icon_and_name,
};

pub trait DatabaseEntry {
    fn generic_name() -> &'static str;
    fn table_name_static() -> &'static str;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
    fn columns_static() -> &'static [(&'static str, &'static str)];
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;

    fn zero() -> SensorData
    where
        Self: Default + Into<SensorData>,
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
        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            Self::table_name_static(),
            col_defs.join(", ")
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

impl DatabaseEntry for CPUData {
    fn generic_name() -> &'static str {
        "CPU"
    }

    fn table_name_static() -> &'static str {
        "cpu_data"
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

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.app_name,
            &self.process_exe_path,
            &self.process_usage_watt,
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
            ("process_usage_watt", "REAL"),
            ("process_cpu_usage", "REAL"),
            ("process_gpu_usage", "REAL"),
            ("process_mem_usage", "REAL"),
            ("read_bytes_per_sec", "REAL"),
            ("written_bytes_per_sec", "REAL"),
            ("subprocess_count", "INTEGER"),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let proc = ProcessData {
            app_name: row.get("app_name")?,
            process_exe_path: row.get("process_exe_path")?,
            process_usage_watt: row.get("process_usage_watt")?,
            process_cpu_usage: row.get("process_cpu_usage")?,
            process_gpu_usage: row.get("process_gpu_usage")?,
            process_mem_usage: row.get("process_mem_usage")?,
            read_bytes_per_sec: row.get("read_bytes_per_sec")?,
            written_bytes_per_sec: row.get("written_bytes_per_sec")?,
            subprocess_count: row.get("subprocess_count")?,
            icon: None,
        };
        let (icon, friendly_name) = if let Some(exe_path) = &proc.process_exe_path {
            load_icon_and_name(exe_path)
        } else {
            (None, None)
        };
        Ok(ProcessData {
            icon,
            app_name: friendly_name.unwrap_or(proc.app_name),
            ..proc
        })
    }

    fn zero() -> SensorData {
        SensorData::Process(Vec::new())
    }
}

impl DatabaseEntry for AllTimeData {
    fn generic_name() -> &'static str {
        "AllTime"
    }

    fn table_name_static() -> &'static str {
        "all_time_data"
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![timestamp_id, &self.total_power_watts, &self.duration_seconds]
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[("total_power_watts", "REAL"), ("duration_seconds", "INTEGER")]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(AllTimeData {
            total_power_watts: row.get("total_power_watts")?,
            duration_seconds: row.get("duration_seconds")?,
        })
    }
}

mod tests {
    use super::{
        AllTimeData, CPUData, DatabaseEntry, DiskData, GPUData, NetworkData, ProcessData, RamData, SensorData,
        TotalData,
    };

    #[test]
    fn zero_defaults_are_zero_filled() {
        // CPU
        match CPUData::zero() {
            SensorData::CPU(cpu) => {
                assert_eq!(cpu.total_power_watts, Some(0.0));
                assert_eq!(cpu.pp0_power_watts, Some(0.0));
                assert_eq!(cpu.pp1_power_watts, Some(0.0));
                assert_eq!(cpu.dram_power_watts, Some(0.0));
                assert_eq!(cpu.usage_percent, Some(0.0));
            }
            _ => panic!("CPUData::zero() returned wrong SensorData variant"),
        }

        // GPU
        match GPUData::zero() {
            SensorData::GPU(gpu) => {
                assert_eq!(gpu.total_power_watts, Some(0.0));
                assert_eq!(gpu.usage_percent, Some(0.0));
                assert_eq!(gpu.vram_usage_percent, Some(0.0));
            }
            _ => panic!("GPUData::zero() returned wrong SensorData variant"),
        }

        // RAM
        match RamData::zero() {
            SensorData::Ram(ram) => {
                assert_eq!(ram.total_power_watts, Some(0.0));
                assert_eq!(ram.usage_percent, Some(0.0));
            }
            _ => panic!("RamData::zero() returned wrong SensorData variant"),
        }

        // Disk
        match DiskData::zero() {
            SensorData::Disk(disk) => {
                assert_eq!(disk.total_power_watts, Some(0.0));
                assert_eq!(disk.read_usage_mb_s, 0.0);
                assert_eq!(disk.write_usage_mb_s, 0.0);
            }
            _ => panic!("DiskData::zero() returned wrong SensorData variant"),
        }

        // Network
        match NetworkData::zero() {
            SensorData::Network(net) => {
                assert_eq!(net.total_power_watts, Some(0.0));
                assert_eq!(net.download_speed_mb_s, 0.0);
                assert_eq!(net.upload_speed_mb_s, 0.0);
            }
            _ => panic!("NetworkData::zero() returned wrong SensorData variant"),
        }

        // Total
        match TotalData::zero() {
            SensorData::Total(total) => {
                assert_eq!(total.total_power_watts, 0.0);
                assert_eq!(total.period_type, "second");
            }
            _ => panic!("TotalData::zero() returned wrong SensorData variant"),
        }

        // Process
        match ProcessData::zero() {
            SensorData::Process(vec) => {
                assert!(vec.is_empty());
            }
            _ => panic!("ProcessData::zero() returned wrong SensorData variant"),
        }

        // AllTime
        match AllTimeData::zero() {
            SensorData::AllTime(all_time) => {
                assert_eq!(all_time.total_power_watts, 0.0);
                assert_eq!(all_time.duration_seconds, 0);
            }
            _ => panic!("AllTimeData::zero() returned wrong SensorData variant"),
        }
    }
}
