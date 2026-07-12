pub mod entries;
mod migration;
pub mod purge;

use core::time;
use std::{collections::HashMap, time::SystemTime};

pub use entries::DatabaseEntry;
pub use purge::averaging_and_purging_data;
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::{
    AllTimeData,
    types::{
        CPUData, ComputedSensorData, DiskData, EnergyUj, Event, GPUData, GeneralData, HardwareInfo, NetworkData,
        ProcessData, RamData, TotalData,
    },
};

pub static DATABASE_PATH: &str = "power_monitoring.db";
pub const DATABASE_TARGET_VERSION: i32 = 2;
pub const LIVE_SAMPLING_PERIOD_SECONDS: i64 = 1;
pub const HOURLY_SAMPLING_PERIOD_SECONDS: i64 = 3600;

macro_rules! dispatch_entry {
    // Static method dispatch based on table name
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

    // Instance method dispatch based on table name and generic type
    ($table_name:expr, $instance:expr, $method:ident, $generic_p:ty, ( $($arg:expr),* )) => {{
        if $table_name == CPUData::table_name_static() { Some($instance.$method::<CPUData, $generic_p>($($arg),*)) }
        else if $table_name == GPUData::table_name_static() { Some($instance.$method::<GPUData, $generic_p>($($arg),*)) }
        else if $table_name == RamData::table_name_static() { Some($instance.$method::<RamData, $generic_p>($($arg),*)) }
        else if $table_name == DiskData::table_name_static() { Some($instance.$method::<DiskData, $generic_p>($($arg),*)) }
        else if $table_name == NetworkData::table_name_static() { Some($instance.$method::<NetworkData, $generic_p>($($arg),*)) }
        else if $table_name == TotalData::table_name_static() || $table_name == AllTimeData::table_name_static() {
            Some($instance.$method::<TotalData, $generic_p>($($arg),*))
        }
        else if $table_name == ProcessData::table_name_static() { Some($instance.$method::<ProcessData, $generic_p>($($arg),*)) }
        else { None }
    }};
}

/// Returns `true` if the table name is in the known list of sensor tables.
pub fn is_valid_table_name(name: &str) -> bool {
    dispatch_entry!(name, table_name_static()).is_some()
}

/// Returns the display name for a given table name (e.g. "cpu_data" -> "CPU").
pub fn generic_name_for_table(table_name: &str) -> Option<&'static str> {
    dispatch_entry!(table_name, generic_name())
}

/// SQLite database handle with tracked table list.
pub struct Database {
    pub(crate) conn: Connection,
    tables: Option<Vec<String>>,
}

/// All persisted UI preference fields.
#[derive(Debug, Clone)]
pub struct UiSettings {
    pub language: String,
    pub carbon_intensity: String,
    pub kwh_cost: String,
    pub theme: String,
}

/// Error types for database operations.
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
    /// Opens the database, applies collector-owned migrations, and reads metadata.
    pub fn new() -> Result<Self, DatabaseError> {
        let mut conn = Self::open_connection()?;
        let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if current_version == 0 {
            // Differentiate between a fresh DB and a version 0 DB
            let timestamp_table_exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='timestamp')",
                [],
                |row| row.get(0),
            )?;

            if timestamp_table_exists {
                // Legacy Version 0
                migration::run_migrations(&mut conn)?;
            } else {
                // Fresh database
                Self::create_base_tables(&conn)?;
                conn.pragma_update(None, "user_version", DATABASE_TARGET_VERSION)?;
            }
        } else if current_version < DATABASE_TARGET_VERSION {
            // Legacy versioned database
            migration::run_migrations(&mut conn)?;
        }

        Self::from_connection(conn)
    }

    fn create_base_tables(conn: &Connection) -> Result<(), DatabaseError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS timestamp (
                sampling_period INTEGER NOT NULL,
                timestamp       INTEGER NOT NULL,
                PRIMARY KEY (sampling_period, timestamp)
            ) WITHOUT ROWID;

            CREATE TABLE IF NOT EXISTS devices (
                id   INTEGER PRIMARY KEY,
                kind TEXT NOT NULL,
                name TEXT NOT NULL,
                UNIQUE(kind, name)
            );

            CREATE TABLE IF NOT EXISTS apps (
                id       INTEGER PRIMARY KEY,
                identity TEXT NOT NULL UNIQUE,
                name     TEXT NOT NULL,
                exe_path TEXT
            );

            CREATE TABLE IF NOT EXISTS hardware_info (
                id            INTEGER PRIMARY KEY,
                tables        TEXT,
                hardware_data TEXT
            );

            CREATE TABLE IF NOT EXISTS component_all_time_data (
                id              INTEGER PRIMARY KEY,
                component_name  TEXT UNIQUE NOT NULL,
                total_energy_uj INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        Ok(())
    }

    /// Opens the database for the UI without running migrations.
    pub fn open_without_migrations() -> Result<Self, DatabaseError> {
        Self::from_connection(Self::open_connection()?)
    }

    fn open_connection() -> Result<Connection, DatabaseError> {
        let conn = Connection::open(DATABASE_PATH)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", "5000")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")?;
        Self::create_settings_table_if_not_exists(&conn)?;
        Ok(conn)
    }

    fn create_settings_table_if_not_exists(conn: &Connection) -> Result<(), DatabaseError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ui_settings (
                id               INTEGER PRIMARY KEY CHECK (id = 1),
                language         TEXT NOT NULL DEFAULT 'EN',
                carbon_intensity TEXT NOT NULL DEFAULT 'World average',
                kwh_cost         TEXT NOT NULL DEFAULT 'World average',
                theme            TEXT NOT NULL DEFAULT 'Hunting'
            )",
        )?;
        Ok(())
    }

    fn from_connection(conn: Connection) -> Result<Self, DatabaseError> {
        let tables = match conn.prepare("SELECT tables FROM hardware_info ORDER BY id DESC LIMIT 1") {
            Err(e) => {
                crate::clog!("✗ Failed to read hardware_info tables: {:?}", e);
                None
            }
            Ok(mut stmt) => match stmt.query_row([], |row| row.get::<_, String>(0)).optional() {
                Ok(Some(materials)) => Some(
                    materials
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|name| is_valid_table_name(name))
                        .collect(),
                ),
                _ => None,
            },
        };
        Ok(Database { conn, tables })
    }

    pub fn user_version(&self) -> Result<i32, DatabaseError> {
        Ok(self.conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    pub fn is_schema_current(&self) -> Result<bool, DatabaseError> {
        Ok(self.user_version()? >= DATABASE_TARGET_VERSION)
    }

    /// Creates sensor tables that don't already exist.
    pub fn create_tables_if_not_exists(&mut self, table_names: &Vec<String>) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;

        let mut current_tables = self.tables.clone().unwrap_or_default();
        let mut has_changed = false;
        for name in table_names {
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

    /// Returns the list of active sensor table names.
    pub fn get_tables(&self) -> Vec<String> {
        self.tables.clone().unwrap_or_default()
    }

    /// Insert an event and update component energy totals in a single transaction.
    pub fn insert_event_and_update_energy(
        &mut self,
        event: &Event<ComputedSensorData>,
        sampling_period: u32,
    ) -> Result<(), DatabaseError> {
        let timestamp_ms = event.time().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;
        let sampling_period_i64 = sampling_period as i64;

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO timestamp (sampling_period, timestamp) VALUES (?1, ?2)",
            params![sampling_period_i64, timestamp_ms],
        )?;
        for sensor_data in event.data() {
            Self::insert_sensor_data(&tx, sampling_period_i64, timestamp_ms, sensor_data)?;
        }

        // Batch energy updates in the same transaction
        for sensor_data in event.data() {
            if let Some(energy) = sensor_data.total_energy() {
                tx.execute(
                    "INSERT INTO component_all_time_data (component_name, total_energy_uj) VALUES (?1, ?2) \
                     ON CONFLICT(component_name) DO UPDATE SET total_energy_uj = total_energy_uj + ?2",
                    params![sensor_data.table_name(), energy],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Inserts a sensor event with all its readings.
    pub fn insert_event(&mut self, event: &Event<ComputedSensorData>) -> Result<(), DatabaseError> {
        let timestamp_ms = event.time().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;
        let sampling_period_i64: i64 = 1;

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO timestamp (sampling_period, timestamp) VALUES (?1, ?2)",
            params![sampling_period_i64, timestamp_ms],
        )?;
        for sensor_data in event.data() {
            Self::insert_sensor_data(&tx, sampling_period_i64, timestamp_ms, sensor_data)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Insert hardware info if line with id=1 doesn't exist, otherwise update it
    pub fn insert_hardware_info(&mut self, data: &GeneralData) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;

        let existing: Option<i64> = tx
            .query_row("SELECT id FROM hardware_info WHERE id = 1", [], |row| row.get(0))
            .optional()?;

        if existing.is_some() {
            tx.execute(
                "UPDATE hardware_info SET tables = ?1, hardware_data = ?2 WHERE id = 1",
                params![data.tables, data.hardware_info.serialized()],
            )?;
        } else {
            tx.execute(
                "INSERT INTO hardware_info (id, tables, hardware_data) VALUES (1, ?1, ?2)",
                params![data.tables, data.hardware_info.serialized()],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Loads the stored hardware info from the database.
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

    /// Adds energy to a component's cumulative total.
    pub fn update_component_all_time_data(
        &mut self,
        component_name: &str,
        energy: EnergyUj,
    ) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO component_all_time_data (component_name, total_energy_uj) VALUES (?1, ?2) \
             ON CONFLICT(component_name) DO UPDATE SET total_energy_uj = total_energy_uj + ?2",
            params![component_name, energy],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Loads all persisted UI settings.
    pub fn load_ui_settings(&self) -> Result<Option<UiSettings>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT language, carbon_intensity, kwh_cost, theme \
             FROM ui_settings WHERE id = 1",
        )?;
        let result = stmt
            .query_row([], |row| {
                Ok(UiSettings {
                    language: row.get(0)?,
                    carbon_intensity: row.get(1)?,
                    kwh_cost: row.get(2)?,
                    theme: row.get(3)?,
                })
            })
            .optional()?;
        Ok(result)
    }

    /// Persists all UI settings.
    pub fn save_ui_settings(&mut self, settings: &UiSettings) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO ui_settings (id, language, carbon_intensity, kwh_cost, theme) \
             VALUES (1, ?1, ?2, ?3, ?4) \
             ON CONFLICT(id) DO UPDATE SET \
               language = ?1, carbon_intensity = ?2, kwh_cost = ?3, theme = ?4",
            params![
                settings.language,
                settings.carbon_intensity,
                settings.kwh_cost,
                settings.theme
            ],
        )?;
        Ok(())
    }

    fn insert_sensor_data(
        tx: &Transaction,
        sampling_period: i64,
        timestamp_ms: i64,
        sensor_data: &ComputedSensorData,
    ) -> Result<(), DatabaseError> {
        match sensor_data {
            ComputedSensorData::GPU(gpu) => {
                let device_id = get_or_create_device_id(tx, "gpu", gpu.name.as_deref().unwrap_or("unknown"))?;
                tx.execute(
                    "INSERT OR IGNORE INTO gpu_data \
                     (sampling_period, timestamp, device_id, total_energy_uj, usage_percent, vram_usage_percent) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        sampling_period,
                        timestamp_ms,
                        device_id,
                        gpu.total_energy,
                        gpu.usage_percent,
                        gpu.vram_usage_percent
                    ],
                )?;
                Ok(())
            }
            ComputedSensorData::Process(processes) => {
                for process in processes {
                    let identity = process
                        .measured
                        .process_exe_path
                        .as_deref()
                        .unwrap_or(&process.measured.app_name);
                    let app_id = get_or_create_app_id(
                        tx,
                        identity,
                        &process.measured.app_name,
                        process.measured.process_exe_path.as_deref(),
                    )?;
                    tx.execute(
                        "INSERT OR IGNORE INTO process_data \
                         (sampling_period, timestamp, app_id, process_energy_uj, \
                          process_cpu_usage, process_gpu_usage, process_mem_usage, \
                          read_bytes, written_bytes, subprocess_count) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![
                            sampling_period,
                            timestamp_ms,
                            app_id,
                            process.process_energy,
                            process.measured.process_cpu_usage,
                            process.measured.process_gpu_usage,
                            process.measured.process_mem_usage,
                            process.measured.read_bytes,
                            process.measured.written_bytes,
                            process.subprocess_count
                        ],
                    )?;
                }
                Ok(())
            }
            ComputedSensorData::CPU(data) => Self::insert_entry(tx, sampling_period, timestamp_ms, data),
            ComputedSensorData::Ram(data) => Self::insert_entry(tx, sampling_period, timestamp_ms, data),
            ComputedSensorData::Disk(data) => Self::insert_entry(tx, sampling_period, timestamp_ms, data),
            ComputedSensorData::Network(data) => Self::insert_entry(tx, sampling_period, timestamp_ms, data),
            ComputedSensorData::Total(data) => Self::insert_entry(tx, sampling_period, timestamp_ms, data),
        }
    }

    fn insert_entry<T: DatabaseEntry>(
        tx: &Transaction,
        sampling_period: i64,
        timestamp_ms: i64,
        entry: &T,
    ) -> Result<(), DatabaseError> {
        let sql = T::insert_sql();
        let params = entry.insert_params(&sampling_period, &timestamp_ms);
        tx.execute(&sql, params.as_slice())?;
        Ok(())
    }

    /// Queries sensor data between two timestamps.
    pub fn select_data_in_time_range(
        &mut self,
        table_name: &str,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<(SystemTime, ComputedSensorData)>, DatabaseError> {
        let start_time_millis = to_epoch_millis(start_time)?;
        let end_time_millis = to_epoch_millis(end_time)?;

        let sensor_data_list = self.select_table_between_millis(table_name, start_time_millis, end_time_millis)?;
        Ok(to_system_time_records(sensor_data_list))
    }

    /// Queries all sensor tables between two timestamps.
    pub fn select_all_data_in_time_range(
        &mut self,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<(SystemTime, ComputedSensorData)>, DatabaseError> {
        let start_time_millis = to_epoch_millis(start_time)?;
        let end_time_millis = to_epoch_millis(end_time)?;

        let mut records = Vec::<(i64, ComputedSensorData)>::new();
        if let Some(tables) = &self.tables {
            for table_name in tables {
                let mut table_records =
                    self.select_table_between_millis(table_name, start_time_millis, end_time_millis)?;
                records.append(&mut table_records);
            }
        }
        Ok(to_system_time_records(records))
    }

    /// Returns windowed averages over the last N seconds.
    pub fn select_last_n_seconds_average(
        &mut self,
        n: i64,
        table_name: &str,
        window_seconds: i64,
    ) -> Result<Vec<(SystemTime, ComputedSensorData)>, DatabaseError> {
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

    /// Returns the most recent N timestamped records.
    pub fn select_last_n_records(&mut self, n: i64) -> Result<Vec<(SystemTime, ComputedSensorData)>, DatabaseError> {
        let mut records = Vec::<(SystemTime, ComputedSensorData)>::new();

        // Fetch last N (sampling_period, timestamp) pairs ordered by timestamp DESC
        let mut stmt = self
            .conn
            .prepare("SELECT sampling_period, timestamp FROM timestamp ORDER BY timestamp DESC LIMIT ?1")?;
        let ts_pairs: Vec<(i64, i64)> = stmt
            .query_map(params![n], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        if ts_pairs.is_empty() {
            return Ok(records);
        }

        // Build a map from timestamp_ms -> SystemTime
        let mut ts_map: HashMap<i64, SystemTime> = HashMap::new();
        for &(_, ts_ms) in &ts_pairs {
            ts_map
                .entry(ts_ms)
                .or_insert_with(|| SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts_ms as u64));
        }

        // Build the IN-clause for timestamp values
        let ts_list: Vec<String> = ts_pairs.iter().map(|(_, ts)| ts.to_string()).collect();
        let ts_in = ts_list.join(",");

        if let Some(tables) = &self.tables {
            for table_name in tables {
                if table_name == ProcessData::table_name_static() {
                    continue;
                }
                if !is_valid_table_name(table_name) {
                    continue;
                }

                // Aggregate multiple GPU rows per timestamp into one
                let query = if table_name == GPUData::table_name_static() {
                    format!(
                        "SELECT d.timestamp, \
                         SUM(d.total_energy_uj) AS total_energy_uj, \
                         AVG(d.usage_percent) AS usage_percent, \
                         AVG(d.vram_usage_percent) AS vram_usage_percent \
                         FROM gpu_data d \
                         WHERE d.timestamp IN ({ts_in}) \
                         GROUP BY d.timestamp",
                    )
                } else {
                    format!(
                        "SELECT d.timestamp, d.* FROM {table} d WHERE d.timestamp IN ({ts_in})",
                        table = table_name,
                    )
                };

                let sensor_data_list = self.execute_sensor_query(table_name, &query, [])?;
                for (ts_ms, sensor_data) in sensor_data_list {
                    if let Some(&sys_time) = ts_map.get(&ts_ms) {
                        records.push((sys_time, sensor_data));
                    }
                }
            }
        }
        Ok(records)
    }

    /// Loads cumulative energy totals for all components.
    pub fn get_all_time_data(&mut self) -> Result<AllTimeData, DatabaseError> {
        let mut components = HashMap::new();
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT component_name, total_energy_uj FROM component_all_time_data")
        {
            if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, EnergyUj>(1)?))) {
                for row in rows.flatten() {
                    components.insert(row.0, row.1);
                }
            }
        }
        Ok(AllTimeData { components })
    }

    /// Dispatches a raw SQL query to the correct typed table reader.
    pub fn execute_sensor_query<P>(
        &self,
        table_name: &str,
        query: &str,
        params: P,
    ) -> rusqlite::Result<Vec<(i64, ComputedSensorData)>>
    where
        P: rusqlite::Params,
    {
        dispatch_entry!(table_name, self, query_sensor_table, P, (query, params)).unwrap_or_else(|| Ok(Vec::new()))
    }

    fn query_sensor_table<T, P>(&self, query: &str, params: P) -> rusqlite::Result<Vec<(i64, ComputedSensorData)>>
    where
        T: DatabaseEntry + Into<ComputedSensorData>,
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
    ) -> Result<Vec<(i64, ComputedSensorData)>, DatabaseError> {
        if !is_valid_table_name(table_name) {
            return Err(DatabaseError::QueryError(format!(
                "Rejected table name: {}",
                table_name
            )));
        }
        // Aggregate multiple GPU device rows into a single reading per timestamp
        let query = if table_name == GPUData::table_name_static() {
            "SELECT t.timestamp, \
                 SUM(d.total_energy_uj) AS total_energy_uj, \
                 AVG(d.usage_percent) AS usage_percent, \
                 AVG(d.vram_usage_percent) AS vram_usage_percent \
             FROM timestamp t \
             JOIN gpu_data d ON t.sampling_period = d.sampling_period AND t.timestamp = d.timestamp \
             WHERE t.timestamp >= ?1 AND t.timestamp <= ?2 \
             GROUP BY t.timestamp \
             ORDER BY t.timestamp ASC"
                .to_string()
        } else {
            format!(
                "SELECT t.timestamp, d.* \
                 FROM timestamp t \
                 JOIN {table} d ON t.sampling_period = d.sampling_period AND t.timestamp = d.timestamp \
                 WHERE t.timestamp >= ?1 AND t.timestamp <= ?2 \
                 ORDER BY t.timestamp ASC",
                table = table_name
            )
        };

        Ok(self.execute_sensor_query(table_name, &query, params![start_time_millis, end_time_millis])?)
    }

    fn select_windowed_table_data(
        &self,
        table_name: &str,
        start_window_start: i64,
        end_exclusive: i64,
        window_seconds: i64,
    ) -> Result<Vec<(i64, ComputedSensorData)>, DatabaseError> {
        if !is_valid_table_name(table_name) {
            return Err(DatabaseError::QueryError(format!(
                "Rejected table name: {}",
                table_name
            )));
        }
        let live_cols = get_windowed_columns(table_name, "d.", window_seconds, WindowedSource::Live)?;
        let hourly_cols = get_windowed_columns(table_name, "d.", window_seconds, WindowedSource::Hourly)?;
        let query = |cols: &str| {
            if table_name == "gpu_data" {
                format!(
                    "SELECT \
                        (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start, \
                        {} \
                     FROM timestamp t \
                     JOIN ( \
                        SELECT sampling_period, timestamp, \
                               SUM(total_energy_uj) AS total_energy_uj, \
                               SUM(usage_percent) AS usage_percent, \
                               SUM(vram_usage_percent) AS vram_usage_percent \
                        FROM gpu_data \
                        GROUP BY sampling_period, timestamp \
                     ) d ON t.sampling_period = d.sampling_period AND t.timestamp = d.timestamp \
                     WHERE t.timestamp >= ?1 AND t.timestamp < ?3 \
                       AND t.sampling_period = ?4 \
                     GROUP BY window_start \
                     ORDER BY window_start ASC",
                    cols
                )
            } else {
                format!(
                    "SELECT \
                        (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start, \
                        {} \
                     FROM timestamp t \
                     JOIN {table} d ON t.sampling_period = d.sampling_period AND t.timestamp = d.timestamp \
                     WHERE t.timestamp >= ?1 AND t.timestamp < ?3 \
                       AND t.sampling_period = ?4 \
                     GROUP BY window_start \
                     ORDER BY window_start ASC",
                    cols,
                    table = table_name
                )
            }
        };

        let live_rows = self.execute_sensor_query(
            table_name,
            &query(&live_cols),
            params![
                start_window_start,
                window_seconds,
                end_exclusive,
                LIVE_SAMPLING_PERIOD_SECONDS
            ],
        )?;
        let hourly_rows = self.execute_sensor_query(
            table_name,
            &query(&hourly_cols),
            params![
                start_window_start,
                window_seconds,
                end_exclusive,
                HOURLY_SAMPLING_PERIOD_SECONDS
            ],
        )?;

        let mut live_by_window = HashMap::new();
        for (window_start, data) in live_rows {
            live_by_window.insert(window_start, data);
        }

        let mut hourly_by_window = HashMap::new();
        for (window_start, data) in hourly_rows {
            hourly_by_window.insert(window_start, data);
        }

        let mut filled = Vec::new();
        let mut current = start_window_start;
        while current < end_exclusive {
            let data = live_by_window
                .remove(&current)
                .or_else(|| hourly_by_window.remove(&current))
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
    ) -> Result<Vec<(i64, ComputedSensorData)>, DatabaseError> {
        let second_query = "SELECT \
                (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start, \
                CAST(SUM(COALESCE(d.total_energy_uj, 0)) / ?2 AS INTEGER) AS total_energy_uj \
             FROM timestamp t \
             JOIN total_data d ON t.sampling_period = d.sampling_period AND t.timestamp = d.timestamp \
             WHERE t.sampling_period = ?4 \
               AND t.timestamp >= ?1 \
               AND t.timestamp < ?3 \
             GROUP BY window_start \
             ORDER BY window_start ASC";

        let hour_query = "SELECT \
                (t.timestamp / (?2 * 1000)) * (?2 * 1000) AS window_start, \
                CAST(AVG(COALESCE(d.total_energy_uj, 0)) / ?4 AS INTEGER) AS total_energy_uj \
             FROM timestamp t \
             JOIN total_data d ON t.sampling_period = d.sampling_period AND t.timestamp = d.timestamp \
             WHERE t.sampling_period = ?4 \
               AND t.timestamp >= ?1 \
               AND t.timestamp < ?3 \
             GROUP BY window_start \
             ORDER BY window_start ASC";

        let second_rows = self.execute_sensor_query(
            TotalData::table_name_static(),
            second_query,
            params![
                start_window_start,
                window_seconds,
                end_exclusive,
                LIVE_SAMPLING_PERIOD_SECONDS
            ],
        )?;

        let hour_rows = self.execute_sensor_query(
            TotalData::table_name_static(),
            hour_query,
            params![
                start_window_start,
                window_seconds,
                end_exclusive,
                HOURLY_SAMPLING_PERIOD_SECONDS
            ],
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
                ComputedSensorData::Total(TotalData {
                    total_energy: EnergyUj::from_u64(0),
                })
            };

            filled.push((current, data));
            current += window_seconds * 1000;
        }

        Ok(filled)
    }

    /// Returns top-N processes averaged over the given time range.
    pub fn select_top_processes_average(
        &self,
        n_seconds: i64,
        top_n: usize,
    ) -> Result<Vec<(SystemTime, ComputedSensorData)>, DatabaseError> {
        if n_seconds == 0 {
            return Ok(vec![(SystemTime::now(), ComputedSensorData::Process(Vec::new()))]);
        }

        let now_ms = to_epoch_millis(SystemTime::now())?;
        let start = now_ms - n_seconds * 1000;

        // JOIN apps to recover app_name and exe_path
        let query = "\
                SELECT \
                    ?2 AS timestamp, \
                    a.name AS app_name, \
                    a.exe_path AS process_exe_path, \
                    CAST(SUM(COALESCE(p.process_energy_uj, 0)) AS INTEGER) AS process_energy_uj, \
                    MIN(SUM(COALESCE(p.process_cpu_usage, 0.0) * t.sampling_period) / SUM(t.sampling_period), 100.0) AS process_cpu_usage, \
                    MIN(SUM(COALESCE(p.process_gpu_usage, 0.0) * t.sampling_period) / SUM(t.sampling_period), 100.0) AS process_gpu_usage, \
                    MIN(SUM(COALESCE(p.process_mem_usage, 0.0) * t.sampling_period) / SUM(t.sampling_period), 100.0) AS process_mem_usage, \
                    CAST(SUM(COALESCE(p.read_bytes, 0)) * 1.0 / SUM(t.sampling_period) AS INTEGER) AS read_bytes, \
                    CAST(SUM(COALESCE(p.written_bytes, 0)) * 1.0 / SUM(t.sampling_period) AS INTEGER) AS written_bytes, \
                    CAST(MAX(p.subprocess_count) AS INTEGER) AS subprocess_count \
                FROM timestamp t \
                JOIN process_data p ON t.sampling_period = p.sampling_period AND t.timestamp = p.timestamp \
                JOIN apps a ON p.app_id = a.id \
                WHERE t.timestamp >= ?1 AND t.timestamp < ?2 \
                GROUP BY p.app_id \
                ORDER BY process_energy_uj DESC \
                LIMIT ?3";

        let rows = self.execute_sensor_query(
            ProcessData::table_name_static(),
            query,
            rusqlite::params![start, now_ms, top_n as i64],
        )?;

        let mut processes = Vec::new();
        for (_, data) in rows {
            if let ComputedSensorData::Process(mut proc_data) = data {
                processes.append(&mut proc_data);
            }
        }
        Ok(vec![(
            from_epoch_millis(now_ms as i64),
            ComputedSensorData::Process(processes),
        )])
    }
}

// ---------------------------------------------------------------------------
// Upsert helpers for devices and apps normalization tables
// ---------------------------------------------------------------------------

/// Returns the id of the device row matching (kind, name), inserting if absent.
pub fn get_or_create_device_id(conn: &Connection, kind: &str, name: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "INSERT INTO devices (kind, name) VALUES (?1, ?2)
         ON CONFLICT (kind, name) DO UPDATE SET kind = excluded.kind
         RETURNING id",
        params![kind, name],
        |row| row.get(0),
    )
}

/// Returns the id of the app row matching identity, inserting or updating if absent.
pub fn get_or_create_app_id(
    conn: &Connection,
    identity: &str,
    name: &str,
    exe_path: Option<&str>,
) -> rusqlite::Result<i64> {
    conn.query_row(
        "INSERT INTO apps (identity, name, exe_path) VALUES (?1, ?2, ?3)
         ON CONFLICT (identity) DO UPDATE SET name = excluded.name
         RETURNING id",
        params![identity, name, exe_path],
        |row| row.get(0),
    )
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum WindowedSource {
    Live,
    Hourly,
}

fn get_windowed_columns(
    table_name: &str,
    prefix: &str,
    window_seconds: i64,
    source: WindowedSource,
) -> Result<String, DatabaseError> {
    let columns = dispatch_entry!(table_name, columns_static())
        .ok_or_else(|| DatabaseError::QueryError(format!("Unknown table for average columns: {}", table_name)))?;

    let aggregated = columns
        .iter()
        .filter(|(name, _)| *name != "device_id" && *name != "app_id")
        .map(|(name, sql_type)| windowed_column_expr(prefix, name, sql_type, window_seconds, source))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(aggregated)
}

fn windowed_column_expr(
    prefix: &str,
    name: &str,
    sql_type: &str,
    window_seconds: i64,
    source: WindowedSource,
) -> String {
    let qualified = format!("{prefix}{name}");
    // Aggregation semantics are inferred from the column-name suffix:
    //   _percent → simple average (already a ratio, no time weighting needed)
    //   _uj      → energy; sum live records and normalize per second
    //   _bytes   → byte rate; live records sum/normalize, hourly records are
    //              already per-second averages so a plain AVG is correct
    //   anything else → plain average
    let expr = if name.ends_with("_percent") {
        format!("AVG({qualified})")
    } else if name.ends_with("_uj") {
        format!("SUM(COALESCE({qualified}, 0)) / {window_seconds}")
    } else if name.ends_with("_bytes") {
        match source {
            WindowedSource::Live => format!("SUM(COALESCE({qualified}, 0)) / {window_seconds}"),
            WindowedSource::Hourly => format!("AVG(COALESCE({qualified}, 0))"),
        }
    } else {
        format!("AVG({qualified})")
    };
    if sql_type.contains("INTEGER") {
        format!("CAST({expr} AS INTEGER) AS {name}")
    } else {
        format!("{expr} AS {name}")
    }
}

fn zero_sensor_data(table_name: &str) -> Option<ComputedSensorData> {
    dispatch_entry!(table_name, zero())
}

fn to_epoch_millis(ts: SystemTime) -> Result<i64, DatabaseError> {
    Ok(ts.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64)
}

fn from_epoch_millis(ts_millis: i64) -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ts_millis as u64)
}

fn to_system_time_records(records: Vec<(i64, ComputedSensorData)>) -> Vec<(SystemTime, ComputedSensorData)> {
    records
        .into_iter()
        .map(|(ts_millis, data)| (from_epoch_millis(ts_millis), data))
        .collect()
}

fn align_to_window_start(timestamp_ms: i64, window_ms: i64) -> i64 {
    timestamp_ms - timestamp_ms.rem_euclid(window_ms)
}
