mod tables;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Params, Result, ToSql, params};
pub use tables::*;

use crate::core::types::{BatteryData, CPUData, Event, GPUData, PeripheralsData, ScreenData, SensorData};

pub static DATABASE_PATH: &str = "power_monitoring.db";

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(DATABASE_PATH)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        Ok(Database { conn })
    }

    pub fn create_tables_if_not_exists(&mut self, tables: &Vec<impl DatabaseTable>) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS timestamp (
                  id            INTEGER PRIMARY KEY,
                  timestamp     TEXT NOT NULL
                  )",
            [],
        )?;

        for table in tables {
            let create_table_sql = format!(
                "CREATE TABLE IF NOT EXISTS {} ({});",
                table.table_name(),
                table.columns().join(", ")
            );
            tx.execute(&create_table_sql, [])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_event(&mut self, event: &Event) -> Result<()> {
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
}

pub trait DatabaseEntry {
    fn insert_sql(&self) -> String;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
}

impl DatabaseEntry for SensorData {
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
}

impl DatabaseEntry for CPUData {
    fn insert_sql(&self) -> String {
        "INSERT INTO cpu_data (timestamp_id, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)".to_string()
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
}

impl DatabaseEntry for GPUData {
    fn insert_sql(&self) -> String {
        "INSERT INTO gpu_data (timestamp_id, total_power_watts, usage_percent, vram_usage_percent) VALUES (?1, ?2, ?3, ?4)".to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.usage_percent,
            &self.vram_usage_percent,
        ]
    }
}
