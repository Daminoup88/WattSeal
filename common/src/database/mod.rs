pub mod entries;
pub mod purge;

use core::time;
use std::{collections::HashMap, time::SystemTime};

pub use entries::DatabaseEntry;
pub use purge::averaging_and_purging_data;
use rusqlite::{Connection, OptionalExtension, Row, ToSql, Transaction, params};

use crate::types::{CPUData, DiskData, Event, GPUData, NetworkData, ProcessData, RamData, SensorData, TotalData};

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

        let tables = match conn.prepare("SELECT detected_materials FROM hardware_info ORDER BY id DESC LIMIT 1") {
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
                  timestamp     INTEGER NOT NULL
                  )",
            [],
        )?;

        tx.execute(
            "CREATE TABLE IF NOT EXISTS hardware_info (
                    id                 INTEGER PRIMARY KEY,
                    timestamp_id       INTEGER REFERENCES timestamp(id) ON DELETE CASCADE,
                    detected_materials TEXT
            )",
            [],
        )?;

        let mut current_tables = self.tables.clone().unwrap_or_default();
        let mut has_changed = false;
        for &name in table_names {
            if !current_tables.contains(&name.to_string()) {
                if let Some(create_sql) = dispatch_entry!(name, create_table_sql()) {
                    tx.execute(&create_sql, [])?;
                    current_tables.push(name.to_string());
                    has_changed = true;
                }
            }
        }
        if !has_changed {
            return Ok(());
        }
        Self::insert_hardware_info(&tx, SystemTime::now(), &current_tables.join(","))?;
        self.tables = Some(current_tables);
        tx.commit()?;
        Ok(())
    }

    pub fn get_tables(&self) -> Vec<String> {
        self.tables.clone().unwrap_or_default()
    }

    pub fn insert_hardware_info(
        tx: &Transaction,
        timestamp: SystemTime,
        detected_materials: &str,
    ) -> Result<(), DatabaseError> {
        tx.execute(
            "INSERT INTO timestamp (timestamp) VALUES (?1)",
            params![timestamp.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64],
        )?;
        let timestamp_id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO hardware_info (timestamp_id, detected_materials) VALUES (?1, ?2)",
            params![timestamp_id, detected_materials],
        )?;
        Ok(())
    }

    pub fn insert_event(&mut self, event: &Event) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO timestamp (timestamp) VALUES (?1)",
            params![event.time().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64],
        )?;
        let timestamp_id = tx.last_insert_rowid();
        for sensor_data in event.data() {
            Self::insert_sensor_data(&tx, &timestamp_id, sensor_data)?;
        }
        tx.commit()?;
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
        let start_time_millis = start_time.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;
        let end_time_millis = end_time.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;

        let query = format!(
            "SELECT t.timestamp, d.* FROM timestamp t JOIN {} d ON t.id = d.timestamp_id \
             WHERE t.timestamp >= {} AND t.timestamp <= {} ORDER BY t.timestamp ASC",
            table_name, start_time_millis, end_time_millis
        );

        let sensor_data_list = self.execute_sensor_query(table_name, &query, [])?;
        let mut records = Vec::<(SystemTime, SensorData)>::new();
        for (ts_millis, sensor_data) in sensor_data_list {
            let timestamp = SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts_millis as u64);
            records.push((timestamp, sensor_data));
        }
        Ok(records)
    }

    pub fn select_last_n_seconds_average(
        &mut self,
        n: i64,
        table_name: &str,
        window_seconds: i64,
    ) -> Result<Vec<(SystemTime, SensorData)>, DatabaseError> {
        let avg_cols = get_avg_columns(table_name, "d.");
        let query = format!(
            "SELECT t.timestamp, {} FROM timestamp t \
             JOIN {} d ON t.id = d.timestamp_id \
             WHERE t.timestamp >= ?1 \
             GROUP BY (t.timestamp / (?2 * 1000)) \
             ORDER BY t.timestamp ASC",
            avg_cols, table_name
        );
        let start_time_millis = (SystemTime::now() - time::Duration::from_secs(n as u64))
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as i64;
        let sensor_data_list =
            self.execute_sensor_query(table_name, &query, params![start_time_millis, window_seconds])?;
        let mut records = Vec::<(SystemTime, SensorData)>::new();
        for (ts_millis, sensor_data) in sensor_data_list {
            let timestamp = SystemTime::UNIX_EPOCH + time::Duration::from_millis(ts_millis as u64);
            records.push((timestamp, sensor_data));
        }
        Ok(records)
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
}

fn get_avg_columns(table_name: &str, prefix: &str) -> String {
    dispatch_entry!(table_name, avg_columns_sql(prefix)).unwrap_or_default()
}
