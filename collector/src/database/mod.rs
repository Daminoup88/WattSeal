mod tables;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, params};
pub use tables::*;

use crate::core::types::{BatteryData, CPUData, Event, GPUData, PeripheralsData, ScreenData};

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
        // CPU data table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cpu_data (
                  id                    INTEGER PRIMARY KEY,
                  timestamp             TEXT NOT NULL,
                  total_power_watts     REAL,
                  pp0_power_watts       REAL,
                  pp1_power_watts       REAL,
                  dram_power_watts      REAL,
                  usage_percent         REAL NOT NULL
                  )",
            [],
        )?;

        // GPU data table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS gpu_data (
                  id                    INTEGER PRIMARY KEY,
                  timestamp             TEXT NOT NULL,
                  total_power_watts     REAL,
                  usage_percent         REAL,
                  vram_usage_percent    REAL
                  )",
            [],
        )?;

        // Screen data table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS screen_data (
                  id                    INTEGER PRIMARY KEY,
                  timestamp             TEXT NOT NULL,
                  resolution_width      INTEGER NOT NULL,
                  resolution_height     INTEGER NOT NULL,
                  refresh_rate_hz       INTEGER NOT NULL,
                  technology            TEXT NOT NULL,
                  luminosity_nits       INTEGER NOT NULL
                  )",
            [],
        )?;

        // Battery data table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS battery_data (
                  id                        INTEGER PRIMARY KEY,
                  timestamp                 TEXT NOT NULL,
                  manufacturer              TEXT NOT NULL,
                  model                     TEXT NOT NULL,
                  serial_number             TEXT NOT NULL,
                  design_capacity_mwh       INTEGER NOT NULL,
                  full_charge_capacity_mwh  INTEGER NOT NULL,
                  cycle_count               INTEGER NOT NULL
                  )",
            [],
        )?;

        // Peripherals data table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS peripherals_data (
                  id                    INTEGER PRIMARY KEY,
                  timestamp             TEXT NOT NULL,
                  device_name           TEXT NOT NULL,
                  device_type           TEXT NOT NULL,
                  manufacturer          TEXT NOT NULL,
                  is_connected          INTEGER NOT NULL
                  )",
            [],
        )?;

        Ok(())
    }
}
