use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};

use crate::core::types::{BatteryData, CPUData, Event, GPUData, PeripheralsData, ScreenData};

// ===== QUERY HELPER FUNCTIONS =====

/// Retrieve the last N CPU data entries
pub fn get_last_cpu_entries(conn: &Connection, limit: usize) -> Result<Vec<Event<CPUData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent 
         FROM cpu_data 
         ORDER BY id DESC 
         LIMIT ?1",
    )?;

    let cpu_iter = stmt.query_map([limit], |row| {
        let timestamp_str: String = row.get(0)?;
        let _timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidQuery)?
            .with_timezone(&Utc);

        Ok(Event::new(CPUData {
            total_power_watts: row.get(1)?,
            pp0_power_watts: row.get(2)?,
            pp1_power_watts: row.get(3)?,
            dram_power_watts: row.get(4)?,
            usage_percent: row.get(5)?,
        }))
    })?;

    cpu_iter.collect()
}

/// Retrieve the last N GPU data entries
pub fn get_last_gpu_entries(conn: &Connection, limit: usize) -> Result<Vec<Event<GPUData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, total_power_watts, usage_percent, vram_usage_percent 
         FROM gpu_data 
         ORDER BY id DESC 
         LIMIT ?1",
    )?;

    let gpu_iter = stmt.query_map([limit], |row| {
        let timestamp_str: String = row.get(0)?;
        let _timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidQuery)?
            .with_timezone(&Utc);

        Ok(Event::new(GPUData {
            total_power_watts: row.get(1)?,
            usage_percent: row.get(2)?,
            vram_usage_percent: row.get(3)?,
        }))
    })?;

    gpu_iter.collect()
}

/// Retrieve CPU data within a time range
pub fn get_cpu_data_in_range(
    conn: &Connection,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<Event<CPUData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent 
         FROM cpu_data 
         WHERE timestamp BETWEEN ?1 AND ?2 
         ORDER BY timestamp ASC",
    )?;

    let cpu_iter = stmt.query_map([start_time.to_rfc3339(), end_time.to_rfc3339()], |row| {
        let timestamp_str: String = row.get(0)?;
        let _timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidQuery)?
            .with_timezone(&Utc);

        Ok(Event::new(CPUData {
            total_power_watts: row.get(1)?,
            pp0_power_watts: row.get(2)?,
            pp1_power_watts: row.get(3)?,
            dram_power_watts: row.get(4)?,
            usage_percent: row.get(5)?,
        }))
    })?;

    cpu_iter.collect()
}

/// Retrieve GPU data within a time range
pub fn get_gpu_data_in_range(
    conn: &Connection,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<Event<GPUData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, total_power_watts, usage_percent, vram_usage_percent 
         FROM gpu_data 
         WHERE timestamp BETWEEN ?1 AND ?2 
         ORDER BY timestamp ASC",
    )?;

    let gpu_iter = stmt.query_map([start_time.to_rfc3339(), end_time.to_rfc3339()], |row| {
        let timestamp_str: String = row.get(0)?;
        let _timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidQuery)?
            .with_timezone(&Utc);

        Ok(Event::new(GPUData {
            total_power_watts: row.get(1)?,
            usage_percent: row.get(2)?,
            vram_usage_percent: row.get(3)?,
        }))
    })?;

    gpu_iter.collect()
}

/// Calculate average CPU power consumption
pub fn get_average_cpu_power(conn: &Connection) -> Result<f64> {
    let mut stmt = conn.prepare("SELECT AVG(total_power_watts) FROM cpu_data WHERE total_power_watts IS NOT NULL")?;

    stmt.query_row([], |row| row.get(0))
}

/// Calculate average GPU power consumption
pub fn get_average_gpu_power(conn: &Connection) -> Result<f64> {
    let mut stmt = conn.prepare("SELECT AVG(total_power_watts) FROM gpu_data WHERE total_power_watts IS NOT NULL")?;

    stmt.query_row([], |row| row.get(0))
}

/// Get total number of CPU data entries
pub fn get_cpu_data_count(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM cpu_data")?;
    stmt.query_row([], |row| row.get(0))
}

/// Get total number of GPU data entries
pub fn get_gpu_data_count(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM gpu_data")?;
    stmt.query_row([], |row| row.get(0))
}

/// Delete old CPU data entries (keep only last N entries)
pub fn cleanup_old_cpu_data(conn: &Connection, keep_last: usize) -> Result<usize> {
    let count: i64 = get_cpu_data_count(conn)?;

    if count as usize > keep_last {
        let delete_count = count as usize - keep_last;
        let affected = conn.execute(
            "DELETE FROM cpu_data WHERE id IN (
                SELECT id FROM cpu_data ORDER BY id ASC LIMIT ?1
            )",
            [delete_count],
        )?;
        Ok(affected)
    } else {
        Ok(0)
    }
}

/// Delete old GPU data entries (keep only last N entries)
pub fn cleanup_old_gpu_data(conn: &Connection, keep_last: usize) -> Result<usize> {
    let count: i64 = get_gpu_data_count(conn)?;

    if count as usize > keep_last {
        let delete_count = count as usize - keep_last;
        let affected = conn.execute(
            "DELETE FROM gpu_data WHERE id IN (
                SELECT id FROM gpu_data ORDER BY id ASC LIMIT ?1
            )",
            [delete_count],
        )?;
        Ok(affected)
    } else {
        Ok(0)
    }
}

/// Delete data older than a specific date
pub fn delete_data_before_date(conn: &Connection, before_date: DateTime<Utc>) -> Result<(usize, usize)> {
    let cpu_deleted = conn.execute("DELETE FROM cpu_data WHERE timestamp < ?1", [before_date.to_rfc3339()])?;

    let gpu_deleted = conn.execute("DELETE FROM gpu_data WHERE timestamp < ?1", [before_date.to_rfc3339()])?;

    Ok((cpu_deleted, gpu_deleted))
}
// ===== SCREEN DATA QUERY FUNCTIONS =====

/// Get the last N screen data entries
pub fn get_last_screen_entries(conn: &Connection, limit: usize) -> Result<Vec<Event<ScreenData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, resolution_width, resolution_height, refresh_rate_hz, technology, luminosity_nits 
         FROM screen_data 
         ORDER BY id DESC 
         LIMIT ?1",
    )?;

    let screen_iter = stmt.query_map([limit], |row| {
        Ok(Event::new(ScreenData {
            resolution: (row.get(1)?, row.get(2)?),
            refresh_rate_hz: row.get(3)?,
            technology: row.get(4)?,
            luminosity_nits: row.get(5)?,
        }))
    })?;

    screen_iter.collect()
}

/// Get total count of screen data entries
pub fn get_screen_data_count(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM screen_data")?;
    stmt.query_row([], |row| row.get(0))
}

// ===== BATTERY DATA QUERY FUNCTIONS =====

/// Get the last N battery data entries
pub fn get_last_battery_entries(conn: &Connection, limit: usize) -> Result<Vec<Event<BatteryData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, manufacturer, model, serial_number, design_capacity_mwh, full_charge_capacity_mwh, cycle_count 
         FROM battery_data 
         ORDER BY id DESC 
         LIMIT ?1"
    )?;

    let battery_iter = stmt.query_map([limit], |row| {
        Ok(Event::new(BatteryData {
            manufacturer: row.get(1)?,
            model: row.get(2)?,
            serial_number: row.get(3)?,
            design_capacity_mwh: row.get(4)?,
            full_charge_capacity_mwh: row.get(5)?,
            cycle_count: row.get(6)?,
        }))
    })?;

    battery_iter.collect()
}

/// Get total count of battery data entries
pub fn get_battery_data_count(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM battery_data")?;
    stmt.query_row([], |row| row.get(0))
}

/// Get battery health percentage (full_charge_capacity / design_capacity * 100)
pub fn get_latest_battery_health(conn: &Connection) -> Result<f64> {
    let mut stmt = conn.prepare(
        "SELECT full_charge_capacity_mwh, design_capacity_mwh 
         FROM battery_data 
         ORDER BY id DESC 
         LIMIT 1",
    )?;

    stmt.query_row([], |row| {
        let full_charge: u32 = row.get(0)?;
        let design: u32 = row.get(1)?;
        Ok((full_charge as f64 / design as f64) * 100.0)
    })
}

// ===== PERIPHERALS DATA QUERY FUNCTIONS =====

/// Get the last N peripherals data entries
pub fn get_last_peripherals_entries(conn: &Connection, limit: usize) -> Result<Vec<Event<PeripheralsData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, device_name, device_type, manufacturer, is_connected 
         FROM peripherals_data 
         ORDER BY id DESC 
         LIMIT ?1",
    )?;

    let peripherals_iter = stmt.query_map([limit], |row| {
        Ok(Event::new(PeripheralsData {
            device_name: row.get(1)?,
            device_type: row.get(2)?,
            manufacturer: row.get(3)?,
            is_connected: row.get::<_, i32>(4)? != 0,
        }))
    })?;

    peripherals_iter.collect()
}

/// Get total count of peripherals data entries
pub fn get_peripherals_data_count(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM peripherals_data")?;
    stmt.query_row([], |row| row.get(0))
}

/// Get all currently connected peripherals
pub fn get_connected_peripherals(conn: &Connection) -> Result<Vec<Event<PeripheralsData>>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, device_name, device_type, manufacturer, is_connected 
         FROM peripherals_data 
         WHERE is_connected = 1 
         ORDER BY id DESC",
    )?;

    let peripherals_iter = stmt.query_map([], |row| {
        Ok(Event::new(PeripheralsData {
            device_name: row.get(1)?,
            device_type: row.get(2)?,
            manufacturer: row.get(3)?,
            is_connected: true,
        }))
    })?;

    peripherals_iter.collect()
}
