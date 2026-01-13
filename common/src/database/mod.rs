mod entries;

use core::time;
use std::{collections::HashMap, time::SystemTime};

pub use entries::DatabaseEntry;
use rusqlite::{Connection, OptionalExtension, Row, ToSql, Transaction, params};

use crate::types::{CPUData, Event, GPUData, SensorData};

pub static DATABASE_PATH: &str = "power_monitoring.db";

pub struct Database {
    conn: Connection,
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
        match err {
            _ => DatabaseError::QueryError(err.to_string()),
        }
    }
}

pub trait DatabaseTable {
    fn table_name(&self) -> &'static str;
    fn columns(&self) -> Vec<String>;
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

    pub fn create_tables_if_not_exists(&mut self, tables: &Vec<impl DatabaseTable>) -> Result<(), DatabaseError> {
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
                    timestamp_id       INTEGER REFERENCES timestamp(id),
                    detected_materials TEXT
            )",
            [],
        )?;

        let mut table_names = Vec::new();

        for table in tables {
            let name = table.table_name();
            let create_table_sql = format!("CREATE TABLE IF NOT EXISTS {} ({});", name, table.columns().join(", "));
            tx.execute(&create_table_sql, [])?;
            table_names.push(name.to_string());
        }
        Self::insert_hardware_info(&tx, SystemTime::now(), &table_names.join(","))?;
        self.tables = Some(table_names);
        tx.commit()?;
        Ok(())
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
            let insert_sql = sensor_data.insert_sql();
            let params = sensor_data.insert_params(&timestamp_id);
            tx.execute(&insert_sql, params.as_slice())?;
        }
        tx.commit()?;
        Ok(())
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
                let sensor_data_list = self.fetch_sensor_data(table_name, &id_list)?;
                for (ts_id, sensor_data) in sensor_data_list {
                    if let Some(ts) = timestamps_map.get(&ts_id) {
                        records.push((*ts, sensor_data));
                    }
                }
            }
        }

        Ok(records)
    }

    fn fetch_sensor_data(&self, table_name: &str, timestamp_ids: &str) -> rusqlite::Result<Vec<(i64, SensorData)>> {
        if table_name == CPUData::table_name_static() {
            self.query_table::<CPUData>(table_name, timestamp_ids)
        } else if table_name == GPUData::table_name_static() {
            self.query_table::<GPUData>(table_name, timestamp_ids)
        } else {
            Ok(Vec::new())
        }
    }

    fn query_table<T>(&self, table_name: &str, timestamp_ids: &str) -> rusqlite::Result<Vec<(i64, SensorData)>>
    where
        T: DatabaseEntry + Into<SensorData>,
    {
        let cols = T::columns_static()
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "SELECT timestamp_id, {} FROM {} WHERE timestamp_id IN ({})",
            cols, table_name, timestamp_ids
        );
        let mut stmt = self.conn.prepare(&query)?;

        let rows = stmt.query_map([], |row| {
            let ts_id: i64 = row.get(0)?;
            let data = T::from_row(row)?;
            Ok((ts_id, data.into()))
        })?;

        rows.collect()
    }
}
