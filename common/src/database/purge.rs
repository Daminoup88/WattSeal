#[cfg(debug_assertions)]
use std::time::Instant;
use std::time::{Duration, SystemTime};

use rusqlite::{OptionalExtension, params};

use crate::{
    database::{Database, HOUR_MS, is_valid_table_name},
    types::{Byte, EnergyUj},
};

/// Aggregates old data into hourly buckets and purges raw records.
pub fn averaging_and_purging_data(
    database: &mut Database,
    average_until_time: i64,
    purge_until_time: i64,
) -> Result<(), String> {
    #[cfg(debug_assertions)]
    let start = Instant::now();
    log_step(
        "Averaging total data",
        averaging_total_data(database, average_until_time),
    );
    #[cfg(debug_assertions)]
    println!("Averaging total data took {} millis", start.elapsed().as_millis());
    #[cfg(debug_assertions)]
    let start = Instant::now();
    log_step(
        "Averaging process data",
        summing_process_data(database, average_until_time),
    );
    #[cfg(debug_assertions)]
    println!("Averaging process data took {} millis", start.elapsed().as_millis());
    #[cfg(debug_assertions)]
    let start = Instant::now();
    log_step("Purging old events", purge_old_events(database, purge_until_time));
    database
        .conn
        .pragma_update(None, "incremental_vacuum", 200)
        .inspect_err(|e| {
            crate::clog!("✗ Vacuum failed: {}", e);
        })
        .ok();
    #[cfg(debug_assertions)]
    println!("Purging old events took {} millis", start.elapsed().as_millis());
    Ok(())
}

fn log_step(label: &str, result: Result<(), String>) {
    if let Err(e) = result {
        crate::clog!("✗ {label} failed: {e}");
    }
}

// Insert records of TotalData with average values every hour until the duration specified (ex: 24h ago)
fn averaging_total_data(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    let cutoff_end_timestamp = get_timestamp_oclock() - duration_in_hours * HOUR_MS;

    let first_timestamp: Option<i64> = database
        .conn
        .prepare(
            "SELECT MIN(timestamp) FROM total_data \
             WHERE duration_ms < ?2 \
               AND timestamp < ?1",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?
        .query_row(params![cutoff_end_timestamp, HOUR_MS], |row| row.get(0))
        .optional()
        .map_err(|e| format!("Failed to execute query: {}", e))?
        .flatten();

    let first_timestamp = match first_timestamp {
        Some(ts) => ts,
        None => return Ok(()),
    };

    let first_bucket_end = next_oclock(first_timestamp);

    let mut stmt = database
        .conn
        .prepare(
            "SELECT \
                CASE \
                    WHEN timestamp < ?3 THEN ?1 \
                    ELSE (timestamp / ?4) * ?4 \
                END AS bucket_start, \
                SUM(total_energy_uj) AS total_energy \
             FROM total_data \
             WHERE timestamp >= ?1 \
               AND timestamp < ?2 \
               AND duration_ms < ?5 \
             GROUP BY bucket_start \
             ORDER BY bucket_start",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(
            params![
                first_timestamp,
                cutoff_end_timestamp,
                first_bucket_end,
                HOUR_MS,
                HOUR_MS
            ],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, EnergyUj>(1)?)),
        )
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut aggregated = Vec::<(i64, EnergyUj)>::new();
    for row in rows {
        aggregated.push(row.map_err(|e| format!("Failed to read query row: {}", e))?);
    }
    drop(stmt);

    if aggregated.is_empty() {
        return Ok(());
    }

    let tx = database
        .conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    for (bucket_start, total_energy) in aggregated {
        let event_timestamp = if bucket_start == first_timestamp {
            bucket_start - (bucket_start % HOUR_MS)
        } else {
            bucket_start
        };

        tx.execute(
            "INSERT OR IGNORE INTO total_data (timestamp, duration_ms, total_energy_uj) VALUES (?1, ?2, ?3)",
            params![event_timestamp, HOUR_MS, total_energy],
        )
        .map_err(|e| format!("Failed to insert averaged event: {}", e))?;
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(())
}

/// Aggregate top-N process data into hourly buckets so that process history
/// survives the raw-data purge and can be queried over weeks / months.
fn summing_process_data(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    let cutoff_end_timestamp = get_timestamp_oclock() - duration_in_hours * HOUR_MS;

    // Check whether there are any un-summed process rows to aggregate
    let has_rows: bool = database
        .conn
        .prepare(
            "SELECT 1 FROM process_data \
             WHERE duration_ms < ?2 AND timestamp < ?1 \
             LIMIT 1",
        )
        .and_then(|mut s| s.query_row(params![cutoff_end_timestamp, HOUR_MS], |_| Ok(true)))
        .unwrap_or(false);

    if !has_rows {
        return Ok(());
    }

    // Top-10 processes per hourly bucket, summed
    let mut stmt = database
        .conn
        .prepare(
            "SELECT \
                (p.timestamp / ?2) * ?2 AS bucket_start, \
                p.app_id, \
                a.name AS app_name, \
                a.exe_path AS exe_path, \
                SUM(COALESCE(p.process_energy_uj, 0)) AS process_energy_uj, \
                SUM(COALESCE(p.process_cpu_usage, 0.0))   / MAX(1, COUNT(*)) AS process_cpu_usage, \
                SUM(COALESCE(p.process_gpu_usage, 0.0))   / MAX(1, COUNT(*)) AS process_gpu_usage, \
                SUM(COALESCE(p.process_mem_usage, 0.0))   / MAX(1, COUNT(*)) AS process_mem_usage, \
                SUM(COALESCE(p.read_bytes, 0))  / MAX(1, COUNT(*)) AS read_bytes, \
                SUM(COALESCE(p.written_bytes, 0))/ MAX(1, COUNT(*)) AS written_bytes, \
                MAX(p.subprocess_count) AS subprocess_count \
             FROM process_data p \
             JOIN apps a ON p.app_id = a.id \
             WHERE p.duration_ms < ?3 \
               AND p.timestamp < ?1 \
             GROUP BY bucket_start, p.app_id \
             ORDER BY bucket_start, process_energy_uj DESC",
        )
        .map_err(|e| format!("prepare process avg: {}", e))?;

    // Collect rows grouped by bucket
    let rows = stmt
        .query_map(params![cutoff_end_timestamp, HOUR_MS, HOUR_MS], |row| {
            Ok((
                row.get::<_, i64>(0)?,      // bucket_start
                row.get::<_, i64>(1)?,      // app_id
                row.get::<_, EnergyUj>(4)?, // energy
                row.get::<_, f64>(5)?,      // cpu
                row.get::<_, f64>(6)?,      // gpu
                row.get::<_, f64>(7)?,      // mem
                row.get::<_, Byte>(8)?,     // read
                row.get::<_, Byte>(9)?,     // write
                row.get::<_, u32>(10)?,     // subprocs
            ))
        })
        .map_err(|e| format!("query process avg: {}", e))?;

    // Keep only top-10 per bucket
    let mut buckets: Vec<(i64, Vec<(i64, EnergyUj, f64, f64, f64, Byte, Byte, u32)>)> = Vec::new();
    for row in rows {
        let (bucket, app_id, euj, cpu, gpu, mem, rd, wr, sub) = row.map_err(|e| format!("row: {}", e))?;
        if buckets.last().map_or(true, |(b, _)| *b != bucket) {
            buckets.push((bucket, Vec::new()));
        }
        let procs = &mut buckets.last_mut().unwrap().1;
        if procs.len() < 10 {
            procs.push((app_id, euj, cpu, gpu, mem, rd, wr, sub));
        }
    }
    drop(stmt);

    if buckets.is_empty() {
        return Ok(());
    }

    let tx = database.conn.transaction().map_err(|e| format!("tx: {}", e))?;

    for (bucket_start, procs) in &buckets {
        for (app_id, pw, cpu, gpu, mem, rd, wr, sub) in procs {
            tx.execute(
                "INSERT OR IGNORE INTO process_data \
                 (timestamp, duration_ms, app_id, process_energy_uj, \
                  process_cpu_usage, process_gpu_usage, process_mem_usage, \
                  read_bytes, written_bytes, subprocess_count) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![bucket_start, HOUR_MS, app_id, pw, cpu, gpu, mem, rd, wr, sub],
            )
            .map_err(|e| format!("insert proc: {}", e))?;
        }
    }

    tx.commit().map_err(|e| format!("commit: {}", e))?;

    Ok(())
}

// Delete directly from every active sensor table for events older than cutoff duration.
fn purge_old_events(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    let cutoff_timestamp = get_timestamp_oclock() - duration_in_hours * HOUR_MS;

    let tables = database.get_tables();
    let tx = database.conn.transaction().map_err(|e| format!("tx: {}", e))?;

    for table in tables {
        if !is_valid_table_name(&table) {
            continue;
        }
        let sql = format!("DELETE FROM {} WHERE timestamp < ?1 AND duration_ms < ?2", table);
        tx.execute(&sql, params![cutoff_timestamp, HOUR_MS])
            .map_err(|e| format!("Failed to delete old events from {}: {}", table, e))?;
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;
    Ok(())
}

fn get_timestamp_oclock() -> i64 {
    let timestamp_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as i64;
    let ms_after_oclock = timestamp_now % HOUR_MS;

    return timestamp_now - ms_after_oclock;
}

fn next_oclock(timestamp_millis: i64) -> i64 {
    let ms_after_oclock = timestamp_millis % HOUR_MS;
    if ms_after_oclock == 0 {
        timestamp_millis + HOUR_MS
    } else {
        timestamp_millis - ms_after_oclock + HOUR_MS
    }
}
