use std::time::SystemTime;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Result, ToSql, Transaction, params};

use crate::types::{CPUData, GPUData, SensorData};

pub static DATABASE_PATH: &str = "power_monitoring.db";

pub struct Database {
    conn: Connection,
    tables: Option<Vec<String>>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(DATABASE_PATH)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "OFF")?;

        let tables = {
            let mut stmt = conn.prepare("SELECT detected_materials FROM hardware_info ORDER BY id DESC LIMIT 1")?;
            match stmt.query_row([], |row| row.get::<_, String>(0)).optional() {
                Ok(Some(materials)) => Some(materials.split(',').map(|s| s.trim().to_string()).collect()),
                _ => None,
            }
        };
        Ok(Database { conn, tables })
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

        tx.execute(
            "CREATE TABLE IF NOT EXISTS hardware_info (
                    id                 INTEGER PRIMARY KEY,
                    timestamp_id       INTEGER REFERENCES timestamp(id),
                    detected_materials TEXT,
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

    pub fn insert_hardware_info(tx: &Transaction, timestamp: DateTime<Utc>, detected_materials: &str) -> Result<()> {
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

    pub fn select_last_n_events(&mut self, n: i64) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent, timestamp FROM timestamp JOIN cpu_data ON timestamp.id = cpu_data.timestamp_id ORDER BY timestamp.id DESC LIMIT ?1",
        )?;
        let data_rows = stmt.query_map(params![n], |row| {
            Ok((
                CPUData {
                    total_power_watts: row.get(0)?,
                    pp0_power_watts: row.get(1)?,
                    pp1_power_watts: row.get(2)?,
                    dram_power_watts: row.get(3)?,
                    usage_percent: row.get(4)?,
                },
                row.get::<_, String>(5)?,
            ))
        })?;
        for data_row in data_rows {
            let (cpu_data, timestamp_str) = data_row?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc)
                .into();
            let sensor_data = vec![SensorData::CPU(cpu_data)];
            events.push(Event::new(timestamp, sensor_data));
        }
        Ok(events)
    }
}

pub trait DatabaseTable {
    fn table_name(&self) -> &'static str;
    fn columns(&self) -> &'static [&'static str];
}

pub trait DatabaseEntry {
    fn table_name_static() -> &'static str;
    fn insert_sql(&self) -> String;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
    fn select_last_n_sql(&self, n: i64) -> String;
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

    fn select_last_n_sql(&self, n: i64) -> String {
        let name = match self {
            SensorData::CPU(_) => CPUData::table_name_static(),
            SensorData::GPU(_) => GPUData::table_name_static(),
            _ => "",
        };

        format!(
            "SELECT * FROM timestamp JOIN {} ON timestamp.id = {}.timestamp_id ORDER BY timestamp.id DESC LIMIT {}",
            name, name, n
        )
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

    fn select_last_n_sql(&self, _n: i64) -> String {
        "".to_string()
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

    fn select_last_n_sql(&self, _n: i64) -> String {
        "".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    time: SystemTime,
    data: Vec<SensorData>,
}

impl Event {
    pub fn new(time: SystemTime, data: Vec<SensorData>) -> Self {
        Event { time, data }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn data(&self) -> &Vec<SensorData> {
        &self.data
    }
}
