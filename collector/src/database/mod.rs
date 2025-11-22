mod tables;

use std::time::Instant;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, params};

use crate::core::types::{CPUData, Event};

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        Ok(Database { conn })
    }

    pub fn create_tables_if_not_exists(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cpu_data (
                  id                    INTEGER PRIMARY KEY,
                  timestamp             TEXT NOT NULL,
                  total_power_watts     REAL NOT NULL,
                  pp0_power_watts       REAL,
                  pp1_power_watts       REAL,
                  dram_power_watts      REAL,
                  usage_percent         REAL NOT NULL
                  )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_cpu_data(&self, event: &Event<CPUData>) -> Result<()> {
        let data = event.data();
        let mut stmt = self
            .conn
            .prepare("INSERT INTO cpu_data (timestamp, total_power_watts, usage_percent) VALUES (?1, ?2, ?3)")?;
        stmt.execute((
            DateTime::<Utc>::from(event.time()),
            data.total_power_watts,
            data.usage_percent,
        ))?;
        Ok(())
    }
}
