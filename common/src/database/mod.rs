pub mod entries;
pub mod purge;

use core::time;
use std::{collections::HashMap, time::SystemTime};

pub use entries::DatabaseEntry;
pub use purge::averaging_and_purging_data;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::{
    AllTimeData,
    types::{
        CPUData, DiskData, Event, GPUData, GeneralData, HardwareInfo, NetworkData, ProcessData, RamData, SensorData,
        TotalData,
    },
};

pub static DATABASE_PATH: &str = "power_monitoring.db";

macro_rules! dispatch_entry {
    ($table_name:expr, $method:ident ( $($arg:expr),* )) => {{
        if $table_name == CPUData::table_name_static() { Some(CPUData::$method($($arg),*)) }
        else if $table_name == GPUData::table_name_static() { Some(GPUData::$method($($arg),*)) }
        else if $table_name == RamData::table_name_static() { Some(RamData::$method($($arg),*)) }
        else if $table_name == DiskData::table_name_static() { Some(DiskData::$method($($arg),*)) }
        else if $table_name == NetworkData::table_name_static() { Some(NetworkData::$method($($arg),*)) }
        else if $table_name == TotalData::table_name_static() { Some(TotalData::$method($($arg),*)) }
        else if $table_name == ProcessData::table_name_static() { Some(ProcessData::$method($($arg),*)) }
        else { None }
    }};
}

/// Returns the display name for a given table name (e.g. "cpu_data" -> "CPU").
pub fn generic_name_for_table(table_name: &str) -> Option<&'static str> {
    dispatch_entry!(table_name, generic_name())
}

pub struct Database {
    pub(crate) conn: Connection,
    tables: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum DatabaseError {
    TimeError(String),
    QueryError(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::QueryError(msg) | DatabaseError::TimeError(msg) => {
                write!(f, "Database error: {}", msg)
            }
        }
    }
}

impl From<std::time::SystemTimeError> for DatabaseError {
    fn from(err: std::time::SystemTimeError) -> Self {
        DatabaseError::TimeError(err.to_string())
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::QueryError(err.to_string())
    }
}

impl Database {
    pub fn new() -> Result<Self, DatabaseError> {
        let conn = Connection::open(DATABASE_PATH)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "OFF")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ui_settings (
                id                        INTEGER PRIMARY KEY CHECK (id = 1),
                language                  TEXT NOT NULL DEFAULT 'EN',
                carbon_intensity_g_per_kwh REAL NOT NULL DEFAULT 475.0
            )",
        )?;

        let tables = match conn.prepare("SELECT tables FROM hardware_info ORDER BY id DESC LIMIT 1") {
            Err(_) => None,
            Ok(mut stmt) => match stmt.query_row([], |row| row.get::<_, String>(0)).optional() {
                Ok(Some(materials)) => Some(materials.split(',').map(|s| s.trim().to_string()).collect()),
                _ => None,
            },
        };
        Ok(Database { conn, tables })
    }

    pub fn create_tables_if_not_exists(&mut self, table_names: &[&str]) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS timestamp (
                  id            INTEGER PRIMARY KEY,
                  timestamp     INTEGER NOT NULL,
                  period_type   INTEGER NOT NULL
                  )",
            [],
        )?;

        tx.execute(
            "CREATE TABLE IF NOT EXISTS hardware_info (
                    id                 INTEGER PRIMARY KEY,
                    tables             TEXT,
                    hardware_data      TEXT
            )",
            [],
        )?;

        tx.execute(
            "CREATE TABLE IF NOT EXISTS component_all_time_data (
                    id              INTEGER PRIMARY KEY,
                    component_name  TEXT UNIQUE NOT NULL,
                    total_energy_wh REAL NOT NULL DEFAULT 0.0
            )",
            [],
        )?;

        let mut current_tables = self.tables.clone().unwrap_or_default();
        let mut has_changed = false;
        for &name in table_names {
            if !current_tables.contains(&name.to_string()) {
                if let Some(create_sql) = dispatch_entry!(name, create_table_sql()) {
                    tx.execute_batch(&create_sql)?;
                    current_tables.push(name.to_string());
                    has_changed = true;
                }
            }
        }
        if has_changed {
            self.tables = Some(current_tables);
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_tables(&self) -> Vec<String> {
        self.tables.clone().unwrap_or_default()
    }

    pub fn insert_event(&mut self, event: &Event) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO timestamp (timestamp, period_type) VALUES (?1, ?2)",
            params![
                event.time().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64,
                1 as i64
            ],
        )?;
        let timestamp_id = tx.last_insert_rowid();
        for sensor_data in event.data() {
            Self::insert_sensor_data(&tx, &timestamp_id, sensor_data)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Insert hardware info if line with id=1 doesn't exist, otherwise update it
    pub fn insert_hardware_info(&mut self, data: &GeneralData) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        let existing: Option<String> = tx
            .query_row("SELECT hardware_data FROM hardware_info WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        if let Some(_) = existing {
            tx.execute(
                "UPDATE hardware_info SET tables = ?1, hardware_data = ?2 WHERE id = 1",
                params![data.tables, data.hardware_info_serialized],
            )?;
        } else {
            tx.execute(
                "INSERT INTO hardware_info (id, tables, hardware_data) VALUES (1, ?1, ?2)",
                params![data.tables, data.hardware_info_serialized],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_hardware_info(&self) -> Result<HardwareInfo, DatabaseError> {
        let query = "SELECT hardware_data FROM hardware_info WHERE id = 1";
        let mut stmt = self.conn.prepare(query)?;
        let result = stmt.query_row([], |row| {
            let hardware_info_serialized: String = row.get(0)?;
            let hardware_info: HardwareInfo = serde_json::from_str(&hardware_info_serialized)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(hardware_info)
        })?;

        Ok(result)
    }

    pub fn update_component_all_time_data(
        &mut self,
        component_name: &str,
        energy_wh: f64,
    ) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO component_all_time_data (component_name, total_energy_wh) VALUES (?1, ?2) \
             ON CONFLICT(component_name) DO UPDATE SET total_energy_wh = total_energy_wh + ?2",
            params![component_name, energy_wh],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn load_ui_settings(&self) -> Result<Option<(String, f64)>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT language, carbon_intensity_g_per_kwh FROM ui_settings WHERE id = 1")?;
        let result = stmt
            .query_row([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)))
            .optional()?;
        Ok(result)
    }

    pub fn save_ui_settings(&mut self, language: &str, carbon_g_per_kwh: f64) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO ui_settings (id, language, carbon_intensity_g_per_kwh) VALUES (1, ?1, ?2) \
             ON CONFLICT(id) DO UPDATE SET language = ?1, carbon_intensity_g_per_kwh = ?2",
            params![language, carbon_g_per_kwh],
        )?;
        Ok(())
    }

    fn insert_sensor_data(tx: &Transaction, timestamp_id: &i64, sensor_data: &SensorData) -> Result<(), DatabaseError> {
        match sensor_data {
            SensorData::CPU(data) => Self::insert_entry(tx, timestamp_id, data),
            SensorData::GPU(data) => Self::insert_entry(tx, timestamp_id, data),
            SensorData::Ram(data) => Self::insert_entry(tx, timestamp_id, data),
            SensorData::Disk(data) => Self::insert_entry(tx, timestamp_id, data),
            SensorData::Network(data) => Self::insert_entry(tx, timestamp_id, data),
            SensorData::Total(data) => Self::insert_entry(tx, timestamp_id, data),
            SensorData::Process(processes) => {
                for process in processes {
                    Self::insert_entry(tx, timestamp_id, process)?;
                }
                Ok(())
            }
        }
    }

    fn insert_entry<T: DatabaseEntry>(tx: &Transaction, timestamp_id: &i64, entry: &T) -> Result<(), DatabaseError> {
        let sql = T::insert_sql();
        let params = entry.insert_params(timestamp_id);
        tx.execute(&sql, params.as_slice())?;
        Ok(())
    }

    pub fn select_data_in_time_range(
        &mut self,
        table_name: &str,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<(SystemTime, SensorData)>, DatabaseError> {
        let start_time_millis = to_epoch_millis(start_time)?;
        let end_time_millis = to_epoch_millis(end_time)?;

        let sensor_data_list = self.select_table_between_millis(table_name, start_time_millis, end_time_millis)?;
        Ok(to_system_time_records(sensor_data_list))
    }

    pub fn select_all_data_in_time_range(
        &mut self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<(SystemTime, SensorData)>, DatabaseError> {
        let start_time_millis = to_epoch_millis(start_time)?;
        let end_time_millis = to_epoch_millis(end_time)?;

        let mut records = Vec::<(i64, SensorData)>::new();
        if let Some(tables) = &self.tables {
            for table_name in tables {
                let mut table_records =
                    self.select_table_between_millis(table_name, start_time_millis, end_time_millis)?;
                records.append(&mut table_records);
            }
        }
        Ok(to_system_time_records(records))
    }

    pub fn select_last_n_seconds_average(
        &mut self,
        n: i64,
        table_name: &str,
        window_seconds: i64,
    ) -> Result<Vec<(SystemTime, SensorData)>, DatabaseError> {
        if n <= 0 || window_seconds <= 0 {
            return Ok(Vec::new());
        }

        if table_name == ProcessData::table_name_static() {
            return Ok(Vec::new());
        }

        let now_ms = to_epoch_millis(SystemTime::now())?;
        let window_ms = window_seconds * 1000;
        let bucket_count = ((n + window_seconds) / window_seconds).max(1);

        let end_window_start = align_to_window_start(now_ms, window_ms);
        let start_window_start = end_window_start - (bucket_count) * window_ms;
        let query_end_exclusive = end_window_start + window_ms;

        let sensor_data_list = if table_name == TotalData::table_name_static() {
            self.select_windowed_total_data(start_window_start, query_end_exclusive, window_seconds)?
        } else {
            self.select_windowed_table_data(table_name, start_window_start, query_end_exclusive, window_seconds)?
        };

        Ok(to_system_time_records(sensor_data_list))
    }

    pub fn select_last_n_records(&mut self, n: i64) -> Result<Vec<(SystemTime, SensorData)>, DatabaseError> {
        let mut records = Vec::<(SystemTime, SensorData)>::new();
        let mut stmt = self
            .conn
            .prepare("SELECT id, timestamp FROM timestamp ORDER BY id DESC LIMIT ?1")?;
        let timestamps: Vec<(i64, SystemTime)> = stmt
            .query_map(params![n], |row| {
                let id: i64 = row.get(0)?;
                let timestamp_millis: i64 = row.get(1)?;
                let timestamp = SystemTime::UNIX_EPOCH + time::Duration::from_millis(timestamp_millis as u64);
                Ok((id, timestamp))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        if timestamps.is_empty() {
            return Ok(records);
        }

        let mut id_vec = Vec::new();
        let mut timestamps_map = HashMap::new();

        for timestamp in timestamps.iter() {
            id_vec.push(timestamp.0.to_string());
            timestamps_map.insert(timestamp.0, timestamp.1);
        }
        let id_list = id_vec.join(",");

        if let Some(tables) = &self.tables {
            for table_name in tables {
                if table_name == ProcessData::table_name_static() {
                    continue;
                }
                let query = format!(
                    "SELECT timestamp_id, * FROM {} WHERE timestamp_id IN ({})",
                    table_name, id_list
                );
                let sensor_data_list = self.execute_sensor_query(table_name, &query, [])?;
                for (ts_id, sensor_data) in sensor_data_list {
                    if let Some(ts) = timestamps_map.get(&ts_id) {
                        records.push((*ts, sensor_data));
                    }
                }
            }
        }
        Ok(records)
    }

    pub fn get_all_time_data(&mut self) -> Result<AllTimeData, DatabaseError> {
        let mut components = HashMap::new();
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT component_name, total_energy_wh FROM component_all_time_data")
        {
            if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))) {
                for row in rows.flatten() {
                    components.insert(row.0, row.1);
                }
            }
        }
        Ok(AllTimeData { components })
    }

    pub fn execute_sensor_query<P>(
        &self,
        table_name: &str,
        query: &str,
        params: P,
    ) -> rusqlite::Result<Vec<(i64, SensorData)>>
    where
        P: rusqlite::Params,
    {
        if table_name == CPUData::table_name_static() {
            self.query_sensor_table::<CPUData, P>(query, params)
        } else if table_name == GPUData::table_name_static() {
            self.query_sensor_table::<GPUData, P>(query, params)
        } else if table_name == RamData::table_name_static() {
            self.query_sensor_table::<RamData, P>(query, params)
        } else if table_name == DiskData::table_name_static() {
            self.query_sensor_table::<DiskData, P>(query, params)
        } else if table_name == NetworkData::table_name_static() {
            self.query_sensor_table::<NetworkData, P>(query, params)
        } else if table_name == TotalData::table_name_static() {
            self.query_sensor_table::<TotalData, P>(query, params)
        } else if table_name == AllTimeData::table_name_static() {
            self.query_sensor_table::<TotalData, P>(query, params)
        } else if table_name == ProcessData::table_name_static() {
            self.query_sensor_table::<ProcessData, P>(query, params)
        } else {
            Ok(Vec::new())
        }
    }

    fn query_sensor_table<T, P>(&self, query: &str, params: P) -> rusqlite::Result<Vec<(i64, SensorData)>>
    where
        T: DatabaseEntry + Into<SensorData>,
        P: rusqlite::Params,
    {
        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map(params, |row| {
            let ts_id_or_millis: i64 = row.get(0)?;
            let data = T::from_row(row)?;
            Ok((ts_id_or_millis, data.into()))
        })?;

        rows.collect()
    }

    fn select_table_between_millis(
        &self,
        table_name: &str,
        start_time_millis: i64,
        end_time_millis: i64,
    ) -> Result<Vec<(i64, SensorData)>, DatabaseError> {
        let query = format!(
            "SELECT t.timestamp, d.* FROM timestamp t JOIN {} d ON t.id = d.timestamp_id \
             WHERE t.timestamp >= ?1 AND t.timestamp <= ?2 ORDER BY t.timestamp ASC",
            table_name
        );

        Ok(self.execute_sensor_query(table_name, &query, params![start_time_millis, end_time_millis])?)
    }

    fn select_windowed_table_data(
        &self,
        table_name: &str,
        start_window_start: i64,
        end_exclusive: i64,
        window_seconds: i64,
    ) -> Result<Vec<(i64, SensorData)>, DatabaseError> {
        let avg_cols = get_windowed_average_columns(table_name, "d.", window_seconds)?;
        let query = format!(
            "SELECT
                (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start,
                {}
             FROM timestamp t
             JOIN {} d ON t.id = d.timestamp_id
             WHERE t.timestamp >= ?1 AND t.timestamp < ?3
             GROUP BY window_start
             ORDER BY window_start ASC",
            avg_cols, table_name
        );

        let rows = self.execute_sensor_query(
            table_name,
            &query,
            params![start_window_start, window_seconds, end_exclusive],
        )?;

        let mut by_window = HashMap::new();
        for (window_start, data) in rows {
            by_window.insert(window_start, data);
        }

        let mut filled = Vec::new();
        let mut current = start_window_start;
        while current < end_exclusive {
            let data = by_window
                .remove(&current)
                .or_else(|| zero_sensor_data(table_name))
                .ok_or_else(|| {
                    DatabaseError::QueryError(format!("Unsupported table for windowed average: {}", table_name))
                })?;
            filled.push((current, data));
            current += window_seconds * 1000;
        }

        Ok(filled)
    }

    fn select_windowed_total_data(
        &self,
        start_window_start: i64,
        end_exclusive: i64,
        window_seconds: i64,
    ) -> Result<Vec<(i64, SensorData)>, DatabaseError> {
        let second_query = "SELECT
                (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start,
                SUM(COALESCE(d.total_power_watts, 0.0)) / ?2 AS total_power_watts,
                'second' AS period_type
             FROM timestamp t
             JOIN total_data d ON t.id = d.timestamp_id
             WHERE d.period_type = 'second'
               AND t.timestamp >= ?1
               AND t.timestamp < ?3
             GROUP BY window_start
             ORDER BY window_start ASC";

        let hour_query = "SELECT
                (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start,
                AVG(COALESCE(d.total_power_watts, 0.0)) AS total_power_watts,
                'hour' AS period_type
             FROM timestamp t
             JOIN total_data d ON t.id = d.timestamp_id
             WHERE d.period_type = 'hour'
               AND t.timestamp >= ?1
               AND t.timestamp < ?3
             GROUP BY window_start
             ORDER BY window_start ASC";

        let second_rows = self.execute_sensor_query(
            TotalData::table_name_static(),
            second_query,
            params![start_window_start, window_seconds, end_exclusive],
        )?;

        let hour_rows = self.execute_sensor_query(
            TotalData::table_name_static(),
            hour_query,
            params![start_window_start, window_seconds, end_exclusive],
        )?;

        let mut second_by_window = HashMap::new();
        for (window_start, data) in second_rows {
            second_by_window.insert(window_start, data);
        }

        let mut hour_by_window = HashMap::new();
        for (window_start, data) in hour_rows {
            hour_by_window.insert(window_start, data);
        }

        let mut filled = Vec::new();
        let mut current = start_window_start;
        while current < end_exclusive {
            let data = if let Some(second_data) = second_by_window.remove(&current) {
                second_data
            } else if let Some(hour_data) = hour_by_window.remove(&current) {
                hour_data
            } else {
                SensorData::Total(TotalData {
                    total_power_watts: 0.0,
                    period_type: if window_seconds >= 3600 {
                        "hour".to_string()
                    } else {
                        "second".to_string()
                    },
                })
            };

            filled.push((current, data));
            current += window_seconds * 1000;
        }

        Ok(filled)
    }

    pub fn select_top_processes_average(
        &self,
        n_seconds: i64,
        top_n: usize,
        energy_mode: bool,
    ) -> Result<Vec<(SystemTime, SensorData)>, DatabaseError> {
        if n_seconds <= 0 {
            return Ok(vec![(SystemTime::now(), SensorData::Process(Vec::new()))]);
        }

        let now_ms = to_epoch_millis(SystemTime::now())?;
        let start = now_ms - n_seconds * 1000;

        let query = if energy_mode {
            // Energy mode: combine second-level and hour-level data.
            // Second rows: each row = 1 s → power/3600 = Wh contribution.
            // Hour rows:   each row = 1 h of averaged power → value IS Wh.
            // CPU/GPU/RAM/disk are "unfolded" so the final AVG = time-weighted avg
            // (missing time = 0).
            "SELECT
                ?2 AS timestamp,
                combined.app_name AS app_name,
                MAX(combined.process_exe_path) AS process_exe_path,
                SUM(combined.energy_wh) AS process_power_watts,
                SUM(combined.cpu_contrib) / ?4 AS process_cpu_usage,
                SUM(combined.gpu_contrib) / ?4 AS process_gpu_usage,
                SUM(combined.mem_contrib) / ?4 AS process_mem_usage,
                SUM(combined.read_contrib) / ?4 AS read_bytes_per_sec,
                SUM(combined.write_contrib) / ?4 AS written_bytes_per_sec,
                MAX(combined.subprocess_count) AS subprocess_count
             FROM (
                 SELECT
                     p.app_name,
                     p.process_exe_path,
                     COALESCE(p.process_power_watts, 0.0) / 3600.0 AS energy_wh,
                     COALESCE(p.process_cpu_usage, 0.0) AS cpu_contrib,
                     COALESCE(p.process_gpu_usage, 0.0) AS gpu_contrib,
                     COALESCE(p.process_mem_usage, 0.0) AS mem_contrib,
                     COALESCE(p.read_bytes_per_sec, 0.0) AS read_contrib,
                     COALESCE(p.written_bytes_per_sec, 0.0) AS write_contrib,
                     p.subprocess_count
                 FROM timestamp t
                 JOIN process_data p ON t.id = p.timestamp_id
                 WHERE t.period_type = 1
                   AND t.timestamp >= ?1 AND t.timestamp < ?2
                 UNION ALL
                 SELECT
                     p.app_name,
                     p.process_exe_path,
                     COALESCE(p.process_power_watts, 0.0) AS energy_wh,
                     COALESCE(p.process_cpu_usage, 0.0) * 3600.0 AS cpu_contrib,
                     COALESCE(p.process_gpu_usage, 0.0) * 3600.0 AS gpu_contrib,
                     COALESCE(p.process_mem_usage, 0.0) * 3600.0 AS mem_contrib,
                     COALESCE(p.read_bytes_per_sec, 0.0) * 3600.0 AS read_contrib,
                     COALESCE(p.written_bytes_per_sec, 0.0) * 3600.0 AS write_contrib,
                     p.subprocess_count
                 FROM timestamp t
                 JOIN process_data p ON t.id = p.timestamp_id
                 WHERE t.period_type = 3600
                   AND t.timestamp >= ?1 AND t.timestamp < ?2
             ) combined
             GROUP BY combined.app_name
             ORDER BY process_power_watts DESC
             LIMIT ?3"
                .to_string()
        } else {
            // Average mode: only second-level data, simple average.
            "SELECT
                ?2 AS timestamp,
                p.app_name AS app_name,
                MAX(p.process_exe_path) AS process_exe_path,
                SUM(COALESCE(p.process_power_watts, 0.0)) / ?4 AS process_power_watts,
                SUM(COALESCE(p.process_cpu_usage, 0.0)) / ?4 AS process_cpu_usage,
                SUM(COALESCE(p.process_gpu_usage, 0.0)) / ?4 AS process_gpu_usage,
                SUM(COALESCE(p.process_mem_usage, 0.0)) / ?4 AS process_mem_usage,
                SUM(COALESCE(p.read_bytes_per_sec, 0.0)) / ?4 AS read_bytes_per_sec,
                SUM(COALESCE(p.written_bytes_per_sec, 0.0)) / ?4 AS written_bytes_per_sec,
                MAX(p.subprocess_count) AS subprocess_count
             FROM timestamp t
             JOIN process_data p ON t.id = p.timestamp_id
             WHERE t.period_type = 1
               AND t.timestamp >= ?1 AND t.timestamp < ?2
             GROUP BY p.app_name
             ORDER BY process_power_watts DESC
             LIMIT ?3"
                .to_string()
        };

        let rows = self.execute_sensor_query(
            ProcessData::table_name_static(),
            &query,
            params![start, now_ms, top_n as i64, n_seconds as f64],
        )?;

        let mut processes = Vec::new();
        for (_, data) in rows {
            if let SensorData::Process(mut proc_data) = data {
                processes.append(&mut proc_data);
            }
        }
        Ok(vec![(from_epoch_millis(now_ms), SensorData::Process(processes))])
    }
}

fn get_windowed_average_columns(table_name: &str, prefix: &str, window_seconds: i64) -> Result<String, DatabaseError> {
    let columns = dispatch_entry!(table_name, columns_static())
        .ok_or_else(|| DatabaseError::QueryError(format!("Unknown table for average columns: {}", table_name)))?;

    let aggregated = columns
        .iter()
        .map(|(name, _)| {
            format!(
                "SUM(COALESCE({}{}, 0.0)) / {} AS {}",
                prefix, name, window_seconds, name
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    Ok(aggregated)
}

fn zero_sensor_data(table_name: &str) -> Option<SensorData> {
    dispatch_entry!(table_name, zero())
}

fn to_epoch_millis(ts: SystemTime) -> Result<i64, DatabaseError> {
    Ok(ts.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64)
}

fn from_epoch_millis(ts_millis: i64) -> SystemTime {
    SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts_millis as u64)
}

fn to_system_time_records(records: Vec<(i64, SensorData)>) -> Vec<(SystemTime, SensorData)> {
    records
        .into_iter()
        .map(|(ts_millis, data)| (from_epoch_millis(ts_millis), data))
        .collect()
}

fn align_to_window_start(timestamp_ms: i64, window_ms: i64) -> i64 {
    timestamp_ms - timestamp_ms.rem_euclid(window_ms)
}
