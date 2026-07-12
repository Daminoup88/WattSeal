use std::collections::HashMap;

use rusqlite::{Row, ToSql};

use crate::types::{
    AllTimeData, CPUData, ComputedSensorData, DiskData, GPUData, NetworkData, ProcessData, RamData, TotalData,
};

/// Maps a data type to its SQLite table schema and row conversion.
pub trait DatabaseEntry {
    fn generic_name() -> &'static str;
    fn table_name_static() -> &'static str;
    fn insert_params<'a>(&'a self, sampling_period: &'a i64, timestamp: &'a i64) -> Vec<&'a dyn ToSql>;
    fn columns_static() -> &'static [(&'static str, &'static str)];
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;

    fn zero() -> ComputedSensorData
    where
        Self: Default + Into<ComputedSensorData>,
    {
        Self::default().into()
    }

    fn insert_sql() -> String {
        let cols = Self::columns_static();
        let col_names: Vec<&str> = cols.iter().map(|(name, _)| *name).collect();
        let all_cols = format!("sampling_period, timestamp, {}", col_names.join(", "));
        let params: Vec<String> = (1..=cols.len() + 2).map(|i| format!("?{}", i)).collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::table_name_static(),
            all_cols,
            params.join(", ")
        )
    }

    fn create_table_sql() -> String {
        String::new()
    }

    fn avg_columns_sql(prefix: &str) -> String {
        Self::columns_static()
            .iter()
            .map(|(col_name, _)| format!("AVG({}{}) AS {}", prefix, col_name, col_name))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

fn simple_table_sql(table_name: &str, col_defs_extra: &[(&str, &str)]) -> String {
    let mut cols = vec![
        "sampling_period INTEGER NOT NULL".to_string(),
        "timestamp       INTEGER NOT NULL".to_string(),
    ];
    for (name, type_) in col_defs_extra {
        cols.push(format!("{} {}", name, type_));
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {table} ({cols}, \
         FOREIGN KEY (sampling_period, timestamp) REFERENCES timestamp(sampling_period, timestamp) ON DELETE CASCADE, \
         PRIMARY KEY (sampling_period, timestamp)) WITHOUT ROWID",
        table = table_name,
        cols = cols.join(", "),
    )
}

macro_rules! impl_database_entry {
    (
        struct $type:ty {
            generic_name: $generic_name:expr,
            table_name: $table_name:expr,
            mappings: {
                $($field:ident : $col_name:literal => $sql_type:literal),* $(,)?
            }
            $(, extra_fields: { $($extra_field:ident : $extra_val:expr),* $(,)? } )?
        }
    ) => {
        impl DatabaseEntry for $type {
            fn generic_name() -> &'static str {
                $generic_name
            }

            fn table_name_static() -> &'static str {
                $table_name
            }

            fn columns_static() -> &'static [(&'static str, &'static str)] {
                &[ $(($col_name, $sql_type)),* ]
            }

            fn insert_params<'a>(&'a self, sampling_period: &'a i64, timestamp: &'a i64) -> Vec<&'a dyn ToSql> {
                let mut params: Vec<&'a dyn ToSql> = vec![sampling_period, timestamp];
                $( params.push(&self.$field); )*
                params
            }

            fn from_row(row: &Row) -> rusqlite::Result<Self> {
                Ok(Self {
                    $($field: row.get($col_name)?),*
                    $(, $($extra_field: $extra_val),* )?
                })
            }

            fn create_table_sql() -> String {
                simple_table_sql(
                    $table_name,
                    &[ $(($col_name, $sql_type)),* ],
                )
            }
        }
    };
}

impl_database_entry! {
    struct RamData {
        generic_name: "RAM",
        table_name: "ram_data",
        mappings: {
            total_energy:  "total_energy_uj" => "INTEGER",
            usage_percent: "usage_percent"   => "REAL",
        }
    }
}

impl_database_entry! {
    struct DiskData {
        generic_name: "Disk",
        table_name: "disk_data",
        mappings: {
            total_energy:  "total_energy_uj" => "INTEGER",
            read_bytes:    "read_bytes"       => "INTEGER",
            written_bytes: "written_bytes"    => "INTEGER",
        }
    }
}

impl_database_entry! {
    struct NetworkData {
        generic_name: "Network",
        table_name: "network_data",
        mappings: {
            total_energy:      "total_energy_uj"   => "INTEGER",
            downloaded_bytes:  "downloaded_bytes"  => "INTEGER",
            uploaded_bytes:    "uploaded_bytes"    => "INTEGER",
        }
    }
}

impl_database_entry! {
    struct TotalData {
        generic_name: "Total",
        table_name: "total_data",
        mappings: {
            total_energy: "total_energy_uj" => "INTEGER",
        }
    }
}

impl DatabaseEntry for CPUData {
    fn generic_name() -> &'static str {
        "CPU"
    }

    fn table_name_static() -> &'static str {
        "cpu_data"
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("total_energy_uj", "INTEGER"),
            ("pp0_energy_uj", "INTEGER"),
            ("dram_energy_uj", "INTEGER"),
            ("usage_percent", "REAL"),
        ]
    }

    fn insert_params<'a>(&'a self, sampling_period: &'a i64, timestamp: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            sampling_period,
            timestamp,
            &self.total_energy,
            &self.pp0_energy,
            &self.dram_energy,
            &self.usage_percent,
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(CPUData {
            total_energy: row.get("total_energy_uj")?,
            pp0_energy: row.get("pp0_energy_uj")?,
            pp1_energy: None,
            dram_energy: row.get("dram_energy_uj")?,
            usage_percent: row.get("usage_percent")?,
        })
    }

    fn create_table_sql() -> String {
        "CREATE TABLE IF NOT EXISTS cpu_data (\
            sampling_period    INTEGER NOT NULL, \
            timestamp          INTEGER NOT NULL, \
            total_energy_uj    INTEGER, \
            pp0_energy_uj      INTEGER, \
            dram_energy_uj     INTEGER, \
            usage_percent      REAL, \
            FOREIGN KEY (sampling_period, timestamp) REFERENCES timestamp(sampling_period, timestamp) ON DELETE CASCADE, \
            PRIMARY KEY (sampling_period, timestamp)) WITHOUT ROWID"
            .to_string()
    }
}

impl DatabaseEntry for GPUData {
    fn generic_name() -> &'static str {
        "GPU"
    }

    fn table_name_static() -> &'static str {
        "gpu_data"
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("device_id", "INTEGER NOT NULL"),
            ("total_energy_uj", "INTEGER"),
            ("usage_percent", "REAL"),
            ("vram_usage_percent", "REAL"),
        ]
    }

    fn insert_params<'a>(&'a self, sampling_period: &'a i64, timestamp: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            sampling_period,
            timestamp,
            &self.total_energy,
            &self.usage_percent,
            &self.vram_usage_percent,
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(GPUData {
            total_energy: row.get("total_energy_uj")?,
            usage_percent: row.get("usage_percent")?,
            vram_usage_percent: row.get("vram_usage_percent")?,
            name: None, // resolved from devices table when needed
        })
    }

    fn create_table_sql() -> String {
        "CREATE TABLE IF NOT EXISTS gpu_data (\
            sampling_period    INTEGER NOT NULL, \
            timestamp          INTEGER NOT NULL, \
            device_id          INTEGER NOT NULL REFERENCES devices(id), \
            total_energy_uj    INTEGER, \
            usage_percent      REAL, \
            vram_usage_percent REAL, \
            FOREIGN KEY (sampling_period, timestamp) REFERENCES timestamp(sampling_period, timestamp) ON DELETE CASCADE, \
            PRIMARY KEY (sampling_period, timestamp, device_id)) WITHOUT ROWID"
            .to_string()
    }
}

impl DatabaseEntry for ProcessData {
    fn generic_name() -> &'static str {
        "Processes"
    }

    fn table_name_static() -> &'static str {
        "process_data"
    }

    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[
            ("app_id", "INTEGER NOT NULL"),
            ("process_energy_uj", "INTEGER"),
            ("process_cpu_usage", "REAL"),
            ("process_gpu_usage", "REAL"),
            ("process_mem_usage", "REAL"),
            ("read_bytes", "INTEGER"),
            ("written_bytes", "INTEGER"),
            ("subprocess_count", "INTEGER"),
        ]
    }

    fn insert_params<'a>(&'a self, sampling_period: &'a i64, timestamp: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            sampling_period,
            timestamp,
            &self.process_energy,
            &self.measured.process_cpu_usage,
            &self.measured.process_gpu_usage,
            &self.measured.process_mem_usage,
            &self.measured.read_bytes,
            &self.measured.written_bytes,
            &self.subprocess_count,
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        use crate::types::MeasuredProcessData;
        Ok(Self {
            measured: MeasuredProcessData {
                pid: None,
                app_name: row.get("app_name")?,
                process_exe_path: row.get("exe_path")?,
                process_cpu_usage: row.get("process_cpu_usage")?,
                process_gpu_usage: row.get("process_gpu_usage")?,
                process_mem_usage: row.get("process_mem_usage")?,
                read_bytes: row.get("read_bytes")?,
                written_bytes: row.get("written_bytes")?,
            },
            process_energy: row.get("process_energy_uj")?,
            subprocess_count: row.get("subprocess_count")?,
            icon: None,
        })
    }

    fn create_table_sql() -> String {
        "CREATE TABLE IF NOT EXISTS process_data (\
            sampling_period    INTEGER NOT NULL, \
            timestamp          INTEGER NOT NULL, \
            app_id             INTEGER NOT NULL REFERENCES apps(id), \
            process_energy_uj  INTEGER, \
            process_cpu_usage  REAL, \
            process_gpu_usage  REAL, \
            process_mem_usage  REAL, \
            read_bytes         INTEGER, \
            written_bytes      INTEGER, \
            subprocess_count   INTEGER, \
            FOREIGN KEY (sampling_period, timestamp) REFERENCES timestamp(sampling_period, timestamp) ON DELETE CASCADE, \
            PRIMARY KEY (sampling_period, timestamp, app_id)) WITHOUT ROWID"
            .to_string()
    }
}

// Manual fallback block preserved for custom structure handling
impl DatabaseEntry for AllTimeData {
    fn generic_name() -> &'static str {
        "AllTime"
    }
    fn table_name_static() -> &'static str {
        "all_time_data"
    }
    fn insert_params<'a>(&'a self, _sampling_period: &'a i64, _timestamp: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![]
    }
    fn columns_static() -> &'static [(&'static str, &'static str)] {
        &[]
    }
    fn from_row(_: &Row) -> rusqlite::Result<Self> {
        Ok(AllTimeData {
            components: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CPUData, ComputedSensorData, DatabaseEntry, DiskData, GPUData, NetworkData, ProcessData, RamData, TotalData,
    };
    use crate::types::{Byte, EnergyUj};

    #[test]
    fn zero_defaults_are_zero_filled() {
        // CPU
        match CPUData::zero() {
            ComputedSensorData::CPU(cpu) => {
                assert_eq!(cpu.total_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(cpu.pp0_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(cpu.pp1_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(cpu.dram_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(cpu.usage_percent, Some(0.0));
            }
            _ => panic!("CPUData::zero() returned wrong SensorData variant"),
        }

        // GPU
        match GPUData::zero() {
            ComputedSensorData::GPU(gpu) => {
                assert_eq!(gpu.total_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(gpu.usage_percent, Some(0.0));
                assert_eq!(gpu.vram_usage_percent, Some(0.0));
            }
            _ => panic!("GPUData::zero() returned wrong SensorData variant"),
        }

        // RAM
        match RamData::zero() {
            ComputedSensorData::Ram(ram) => {
                assert_eq!(ram.total_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(ram.usage_percent, Some(0.0));
            }
            _ => panic!("RamData::zero() returned wrong SensorData variant"),
        }

        // Disk
        match DiskData::zero() {
            ComputedSensorData::Disk(disk) => {
                assert_eq!(disk.total_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(disk.read_bytes, Byte::from(0));
                assert_eq!(disk.written_bytes, Byte::from(0));
            }
            _ => panic!("DiskData::zero() returned wrong SensorData variant"),
        }

        // Network
        match NetworkData::zero() {
            ComputedSensorData::Network(net) => {
                assert_eq!(net.total_energy, Some(EnergyUj::from_u64(0)));
                assert_eq!(net.downloaded_bytes, Byte::from(0));
                assert_eq!(net.uploaded_bytes, Byte::from(0));
            }
            _ => panic!("NetworkData::zero() returned wrong SensorData variant"),
        }

        // Total
        match TotalData::zero() {
            ComputedSensorData::Total(total) => {
                assert_eq!(total.total_energy, EnergyUj::from_u64(0));
            }
            _ => panic!("TotalData::zero() returned wrong SensorData variant"),
        }

        // Process
        match ProcessData::zero() {
            ComputedSensorData::Process(vec) => {
                assert!(vec.is_empty());
            }
            _ => panic!("ProcessData::zero() returned wrong SensorData variant"),
        }
    }
}
