use core::time;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};

use crate::{
    database::Database,
    types::{Event, SensorData, TotalData},
};

pub fn averaging_and_purging_data(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    averaging_data(database, duration_in_hours)
        .map_err(|e| format!("Failed to average data: {}", e))
        .unwrap();
    purge_old_events(/*database, duration_in_hours*/)
        .map_err(|e| format!("Failed to purge data: {}", e))
        .unwrap();
    Ok(())
}

// Insert records of TotalData with average values every hour until the duration specified (ex: 24h ago)
fn averaging_data(database: &mut Database, duration_in_hours: i64) -> Result<(), String> {
    let cutoff_end_timestamp = get_timestamp_oclock() - duration_in_hours * 3600 * 1000;

    let first_timestamp: Option<i64> = database
        .conn
        .prepare(
            "SELECT MIN(t.timestamp) FROM timestamp t \
             JOIN total_data d ON t.id = d.timestamp_id \
             WHERE d.period_type = 'second'",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?
        .query_row([], |row| row.get(0))
        .optional()
        .map_err(|e| format!("Failed to execute query: {}", e))?
        .flatten();

    // If there's no data to average, return early
    let Some(first_timestamp) = first_timestamp else {
        return Ok(());
    };

    let mut start_ts = first_timestamp;
    let mut end_ts = next_oclock(start_ts);

    while end_ts <= cutoff_end_timestamp {
        let avg_power: Option<f64> = {
            let mut stmt = database
                .conn
                .prepare(
                    "SELECT AVG(d.total_power_watts) FROM timestamp t \
                     JOIN total_data d ON t.id = d.timestamp_id \
                     WHERE t.timestamp >= ?1 AND t.timestamp < ?2 \
                     AND d.period_type = 'second'",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;
            stmt.query_row(params![start_ts, end_ts], |row| row.get(0))
                .map_err(|e| format!("Failed to execute query: {}", e))?
        };

        if let Some(avg_power) = avg_power {
            println!(
                "Averaging data from {} to {}",
                get_datetime_from_ts(start_ts),
                get_datetime_from_ts(end_ts)
            );

            let total_data = TotalData {
                total_power_watts: avg_power,
                period_type: "hour".to_string(),
            };

            let mut event_time = UNIX_EPOCH + time::Duration::from_millis(start_ts as u64);
            if start_ts == first_timestamp {
                println!("First timestamp\n");
                // If first timestamp -> set event time to the time oclock before.
                event_time = UNIX_EPOCH + time::Duration::from_millis(next_oclock(start_ts - 3600 * 1000) as u64);
            }

            let event = Event::new(event_time, vec![SensorData::Total(total_data)]);

            database
                .insert_event(&event)
                .map_err(|e| format!("Failed to insert averaged event: {}", e))?;
        }

        start_ts = end_ts;
        end_ts += 3600 * 1000;
    }

    Ok(())
}

// Delete in Cascade every events of the DB until the duration specified (ex: 24h ago)
// Except if total_data period_type is "hour"
fn purge_old_events(/*database: &mut Database, duration_in_hours: i64*/) -> Result<(), String> {
    // let cutoff_timestamp = get_timestamp_oclock() - duration_in_hours * 3600 * 1000;

    // database
    //     .conn
    //     .execute(
    //         "DELETE FROM event
    //          WHERE timestamp_id IN (
    //              SELECT t.id FROM timestamp t
    //              JOIN total_data d ON t.id = d.timestamp_id
    //              WHERE t.timestamp < ?1 AND d.period_type != 'hour'
    //          )",
    //         params![cutoff_timestamp],
    //     )
    //     .map_err(|e| format!("Failed to delete events: {}", e))?;

    // // Clean up orphaned timestamps that are no longer referenced by any event
    // database
    //     .conn
    //     .execute(
    //         "DELETE FROM timestamp
    //          WHERE id NOT IN (SELECT DISTINCT timestamp_id FROM event)",
    //         [],
    //     )
    //     .map_err(|e| format!("Failed to clean up orphaned timestamps: {}", e))?;

    Ok(())
}

fn get_timestamp_oclock() -> i64 {
    let timestamp_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let ms_after_oclock = timestamp_now % (3600 * 1000);

    return timestamp_now - ms_after_oclock;
}

fn next_oclock(timestamp_millis: i64) -> i64 {
    let hour_ms = 3600 * 1000;
    let ms_after_oclock = timestamp_millis % hour_ms;
    if ms_after_oclock == 0 {
        timestamp_millis + hour_ms
    } else {
        timestamp_millis - ms_after_oclock + hour_ms
    }
}

fn get_datetime_from_ts(timestamp_millis: i64) -> String {
    let datetime = UNIX_EPOCH + time::Duration::from_millis(timestamp_millis as u64);
    let datetime: chrono::DateTime<chrono::Utc> = datetime.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
