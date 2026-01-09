use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row, ToSql, Transaction, params};

use crate::types::{CPUData, Event, GPUData, SensorData};

pub static DATABASE_PATH: &str = "power_monitoring.db";

pub struct Database {
    conn: Connection,
    tables: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum DatabaseError {
    ConversionError(String),
    QueryError(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ConversionError(msg) => write!(f, "Conversion Error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Query Error: {}", msg),
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            _ => DatabaseError::QueryError(err.to_string()),
        }
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

    pub fn create_tables_if_not_exists(&mut self, tables: &Vec<impl DatabaseTable>) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS timestamp (
                  id            INTEGER PRIMARY KEY,
                  timestamp     TEXT NOT NULL
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
        Self::insert_hardware_info(&tx, Utc::now(), &table_names.join(","))?;
        self.tables = Some(table_names);
        tx.commit()?;
        Ok(())
    }

    pub fn insert_hardware_info(
        tx: &Transaction,
        timestamp: DateTime<Utc>,
        detected_materials: &str,
    ) -> Result<(), DatabaseError> {
        tx.execute(
            "INSERT INTO timestamp (timestamp) VALUES (?1)",
            params![timestamp.to_rfc3339()],
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
            params![DateTime::<Utc>::from(event.time()).to_rfc3339()],
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
                let ts_str: String = row.get(1)?;
                let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
                    })?
                    .with_timezone(&Utc)
                    .into();
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

pub trait DatabaseTable {
    fn table_name(&self) -> &'static str;
    fn columns(&self) -> Vec<String>;
}

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
