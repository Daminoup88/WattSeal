use core::time;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};

use crate::database::Database;

const HOUR_MS: i64 = 3600 * 1000;

pub fn averaging_and_purging_data(
    database: &mut Database,
    average_until_time: i64,
    purge_until_time: i64,
) -> Result<(), String> {
    let start = Instant::now();
    averaging_data(database, average_until_time)
        .map_err(|e| format!("Failed to average data: {}", e))
        .ok();
    println!("Time to average data: {:.2?}", Instant::now().duration_since(start));

    let start = Instant::now();
    purge_old_events(database, purge_until_time)
        .map_err(|e| format!("Failed to purge data: {}", e))
        .ok();
    println!("Time to purge data {:.2?}", Instant::now().duration_since(start));

    Ok(())
}

// Insert records of TotalData with average values every hour until the duration specified (ex: 24h ago)
fn averaging_data(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    let cutoff_end_timestamp = get_timestamp_oclock() - duration_in_hours * HOUR_MS;

    let first_timestamp: Option<i64> = database
        .conn
        .prepare(
            "SELECT MIN(timestamp) FROM timestamp \
             WHERE period_type = 1 \
               AND timestamp < ?1",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?
        .query_row([cutoff_end_timestamp], |row| row.get(0))
        .optional()
        .map_err(|e| format!("Failed to execute query: {}", e))?
        .flatten();

    let first_timestamp = match first_timestamp {
        Some(ts) => ts,
        None => {
            println!("No data to average");
            return Ok(());
        }
    };

    println!(
        "First timestamp to average: {} ({}), cutoff end timestamp: {} ({})",
        first_timestamp,
        get_datetime_from_ts(first_timestamp),
        cutoff_end_timestamp,
        get_datetime_from_ts(cutoff_end_timestamp)
    );

    let first_bucket_end = next_oclock(first_timestamp);

    let mut stmt = database
        .conn
        .prepare(
            "SELECT
                CASE
                    WHEN t.timestamp < ?3 THEN ?1
                    ELSE (t.timestamp / ?4) * ?4
                END AS bucket_start,
                AVG(d.total_power_watts) AS avg_power,
                COUNT(d.total_power_watts) AS value_count
             FROM timestamp t
             JOIN total_data d ON t.id = d.timestamp_id
             WHERE t.timestamp >= ?1
               AND t.timestamp < ?2
               AND d.period_type = 'second'
             GROUP BY bucket_start
             ORDER BY bucket_start",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(
            params![first_timestamp, cutoff_end_timestamp, first_bucket_end, HOUR_MS],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, i64>(2)?)),
        )
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut aggregated = Vec::<(i64, f64, i64)>::new();
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

    let mut insert_ts_stmt = tx
        .prepare("INSERT INTO timestamp (timestamp, period_type) VALUES (?1, ?2)")
        .map_err(|e| format!("Failed to prepare timestamp insert: {}", e))?;

    let mut insert_total_stmt = tx
        .prepare("INSERT INTO total_data (timestamp_id, total_power_watts, period_type) VALUES (?1, ?2, 'hour')")
        .map_err(|e| format!("Failed to prepare total_data insert: {}", e))?;

    for (bucket_start, avg_power, value_count) in aggregated {
        let mut power = avg_power;
        if value_count > 0 {
            power = avg_power * (value_count as f64) / 3600.0;
            // We assume missing values = 0W
        }

        let event_timestamp = if bucket_start == first_timestamp {
            bucket_start - (bucket_start % HOUR_MS)
        } else {
            bucket_start
        };

        insert_ts_stmt
            .execute(params![event_timestamp, 3600 as i64])
            .map_err(|e| format!("Failed to insert timestamp: {}", e))?;
        let timestamp_id = tx.last_insert_rowid();

        insert_total_stmt
            .execute(params![timestamp_id, power])
            .map_err(|e| format!("Failed to insert averaged event: {}", e))?;
    }

    drop(insert_total_stmt);
    drop(insert_ts_stmt);

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(())
}

// Delete in Cascade every events of the DB until the duration specified (ex: 24h ago)
// Except if total_data period_type is "hour"
fn purge_old_events(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    let cutoff_timestamp = get_timestamp_oclock() - duration_in_hours * HOUR_MS;

    database
        .conn
        .execute(
            "DELETE FROM timestamp \
             WHERE timestamp < ?1 \
               AND period_type = 1",
            params![cutoff_timestamp],
        )
        .map_err(|e| format!("Failed to delete old events: {}", e))?;

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

fn get_datetime_from_ts(timestamp_millis: i64) -> String {
    let datetime = UNIX_EPOCH + time::Duration::from_millis(timestamp_millis as u64);
    let datetime: chrono::DateTime<chrono::Utc> = datetime.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
