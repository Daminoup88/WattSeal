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

    // ===== CPU DATA OPERATIONS =====
    pub fn insert_cpu_data(&self, event: &Event<CPUData>) -> Result<()> {
        let data = event.data();
        let mut stmt = self.conn.prepare(
            "INSERT INTO cpu_data (timestamp, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )?;
        stmt.execute(params![
            DateTime::<Utc>::from(event.time()).to_rfc3339(),
            data.total_power_watts,
            data.pp0_power_watts,
            data.pp1_power_watts,
            data.dram_power_watts,
            data.usage_percent,
        ])?;
        Ok(())
    }

    // ===== GPU DATA OPERATIONS =====
    pub fn insert_gpu_data(&self, event: &Event<GPUData>) -> Result<()> {
        let data = event.data();
        let mut stmt = self.conn.prepare(
            "INSERT INTO gpu_data (timestamp, total_power_watts, usage_percent, vram_usage_percent) 
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        stmt.execute(params![
            DateTime::<Utc>::from(event.time()).to_rfc3339(),
            data.total_power_watts,
            data.usage_percent,
            data.vram_usage_percent,
        ])?;
        Ok(())
    }

    // ===== SCREEN DATA OPERATIONS =====
    pub fn insert_screen_data(&self, event: &Event<ScreenData>) -> Result<()> {
        let data = event.data();
        self.conn.execute(
            "INSERT INTO screen_data (timestamp, resolution_width, resolution_height, refresh_rate_hz, technology, luminosity_nits) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                DateTime::<Utc>::from(event.time()).to_rfc3339(),
                data.resolution.0,
                data.resolution.1,
                data.refresh_rate_hz,
                data.technology,
                data.luminosity_nits,
            ],
        )?;
        Ok(())
    }

    // ===== BATTERY DATA OPERATIONS =====
    pub fn insert_battery_data(&self, event: &Event<BatteryData>) -> Result<()> {
        let data = event.data();
        self.conn.execute(
            "INSERT INTO battery_data (timestamp, manufacturer, model, serial_number, design_capacity_mwh, full_charge_capacity_mwh, cycle_count) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                DateTime::<Utc>::from(event.time()).to_rfc3339(),
                data.manufacturer,
                data.model,
                data.serial_number,
                data.design_capacity_mwh,
                data.full_charge_capacity_mwh,
                data.cycle_count,
            ],
        )?;
        Ok(())
    }

    // ===== PERIPHERALS DATA OPERATIONS =====
    pub fn insert_peripherals_data(&self, event: &Event<PeripheralsData>) -> Result<()> {
        let data = event.data();
        self.conn.execute(
            "INSERT INTO peripherals_data (timestamp, device_name, device_type, manufacturer, is_connected) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                DateTime::<Utc>::from(event.time()).to_rfc3339(),
                data.device_name,
                data.device_type,
                data.manufacturer,
                data.is_connected as i32,
            ],
        )?;
        Ok(())
    }

    // ===== BATCH INSERT OPERATIONS =====
    /// Insert multiple CPU data events in a single transaction for better performance
    pub fn insert_cpu_data_batch(&self, events: &[Event<CPUData>]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO cpu_data (timestamp, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for event in events {
                let data = event.data();
                stmt.execute(params![
                    DateTime::<Utc>::from(event.time()).to_rfc3339(),
                    data.total_power_watts,
                    data.pp0_power_watts,
                    data.pp1_power_watts,
                    data.dram_power_watts,
                    data.usage_percent,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Insert multiple GPU data events in a single transaction for better performance
    pub fn insert_gpu_data_batch(&self, events: &[Event<GPUData>]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO gpu_data (timestamp, total_power_watts, usage_percent, vram_usage_percent) 
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for event in events {
                let data = event.data();
                stmt.execute(params![
                    DateTime::<Utc>::from(event.time()).to_rfc3339(),
                    data.total_power_watts,
                    data.usage_percent,
                    data.vram_usage_percent,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    // ===== QUERY OPERATIONS (wrappers for table functions) =====

    /// Get the last N CPU data entries
    pub fn get_last_cpu_entries(&self, limit: usize) -> Result<Vec<Event<CPUData>>> {
        tables::get_last_cpu_entries(&self.conn, limit)
    }

    /// Get the last N GPU data entries
    pub fn get_last_gpu_entries(&self, limit: usize) -> Result<Vec<Event<GPUData>>> {
        tables::get_last_gpu_entries(&self.conn, limit)
    }

    /// Get CPU data within a time range
    pub fn get_cpu_data_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Event<CPUData>>> {
        tables::get_cpu_data_in_range(&self.conn, start, end)
    }

    /// Get GPU data within a time range
    pub fn get_gpu_data_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Event<GPUData>>> {
        tables::get_gpu_data_in_range(&self.conn, start, end)
    }

    /// Get average CPU power consumption
    pub fn get_average_cpu_power(&self) -> Result<f64> {
        tables::get_average_cpu_power(&self.conn)
    }

    /// Get average GPU power consumption
    pub fn get_average_gpu_power(&self) -> Result<f64> {
        tables::get_average_gpu_power(&self.conn)
    }

    /// Cleanup old CPU data (keep only last N entries)
    pub fn cleanup_old_cpu_data(&self, keep_last: usize) -> Result<usize> {
        tables::cleanup_old_cpu_data(&self.conn, keep_last)
    }

    /// Cleanup old GPU data (keep only last N entries)
    pub fn cleanup_old_gpu_data(&self, keep_last: usize) -> Result<usize> {
        tables::cleanup_old_gpu_data(&self.conn, keep_last)
    }

    /// Delete data older than a specific date
    pub fn delete_data_before_date(&self, before_date: DateTime<Utc>) -> Result<(usize, usize)> {
        tables::delete_data_before_date(&self.conn, before_date)
    }

    // ===== SCREEN DATA OPERATIONS =====

    /// Get the last N screen data entries
    pub fn get_last_screen_entries(&self, limit: usize) -> Result<Vec<Event<ScreenData>>> {
        tables::get_last_screen_entries(&self.conn, limit)
    }

    /// Get total count of screen data entries
    pub fn get_screen_data_count(&self) -> Result<i64> {
        tables::get_screen_data_count(&self.conn)
    }

    // ===== BATTERY DATA OPERATIONS =====

    /// Get the last N battery data entries
    pub fn get_last_battery_entries(&self, limit: usize) -> Result<Vec<Event<BatteryData>>> {
        tables::get_last_battery_entries(&self.conn, limit)
    }

    /// Get total count of battery data entries
    pub fn get_battery_data_count(&self) -> Result<i64> {
        tables::get_battery_data_count(&self.conn)
    }

    /// Get battery health percentage
    pub fn get_latest_battery_health(&self) -> Result<f64> {
        tables::get_latest_battery_health(&self.conn)
    }

    // ===== PERIPHERALS DATA OPERATIONS =====

    /// Get the last N peripherals data entries
    pub fn get_last_peripherals_entries(&self, limit: usize) -> Result<Vec<Event<PeripheralsData>>> {
        tables::get_last_peripherals_entries(&self.conn, limit)
    }

    /// Get total count of peripherals data entries
    pub fn get_peripherals_data_count(&self) -> Result<i64> {
        tables::get_peripherals_data_count(&self.conn)
    }

    /// Get all currently connected peripherals
    pub fn get_connected_peripherals(&self) -> Result<Vec<Event<PeripheralsData>>> {
        tables::get_connected_peripherals(&self.conn)
    }
}
