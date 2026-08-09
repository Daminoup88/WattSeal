use rusqlite::Connection;

use super::DATABASE_TARGET_VERSION;
use crate::DatabaseError;

pub fn run_migrations(conn: &mut Connection) -> Result<(), DatabaseError> {
    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if version >= DATABASE_TARGET_VERSION {
        return Ok(());
    }

    let tx = conn.transaction()?;

    if version < 2 {
        crate::clog!(
            "Migrating database from version {} to {}...",
            version,
            DATABASE_TARGET_VERSION
        );
        migrate_v1_to_v2(&tx)?;
    }

    tx.pragma_update(None, "user_version", DATABASE_TARGET_VERSION)?;
    tx.commit()?;
    conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")?;
    conn.execute_batch("VACUUM;")?;
    crate::clog!("Migration to version {} complete.", DATABASE_TARGET_VERSION);

    Ok(())
}

fn migrate_v1_to_v2(tx: &rusqlite::Transaction) -> Result<(), DatabaseError> {
    let mut default_gpu = "unknown".to_string();
    if let Ok(serialized) = tx.query_row("SELECT hardware_data FROM hardware_info WHERE id = 1", [], |row| {
        row.get::<_, String>(0)
    }) {
        if let Ok(info) = serde_json::from_str::<crate::HardwareInfo>(&serialized) {
            if let Some(first_gpu) = info.gpus.first() {
                default_gpu = first_gpu.clone();
            }
        }
    }

    let sql = r#"
        -- Create centralisation tables first
        CREATE TABLE IF NOT EXISTS devices (
            id   INTEGER PRIMARY KEY,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            UNIQUE(kind, name)
        );

        CREATE TABLE IF NOT EXISTS apps (
            id       INTEGER PRIMARY KEY,
            identity TEXT NOT NULL UNIQUE,
            name     TEXT NOT NULL,
            exe_path TEXT
        );

        -- 1. Rename old timestamp table so we can read period_type from it during migration
        ALTER TABLE timestamp RENAME TO timestamp_old;

        -- 2. Migrate cpu_data table (Watts -> Microjoules, remove pp1 and dram)
        ALTER TABLE cpu_data RENAME TO cpu_data_old;
        CREATE TABLE cpu_data (
            timestamp       INTEGER NOT NULL,
            duration_ms     INTEGER NOT NULL,
            total_energy_uj INTEGER,
            pp0_energy_uj   INTEGER,
            usage_percent   REAL,
            PRIMARY KEY (timestamp, duration_ms)
        ) WITHOUT ROWID;
        INSERT OR IGNORE INTO cpu_data (timestamp, duration_ms, total_energy_uj, pp0_energy_uj, usage_percent)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               CAST(c.total_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               CAST(c.pp0_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               c.usage_percent 
        FROM cpu_data_old c JOIN timestamp_old t ON c.timestamp_id = t.id;
        -- Note: cpu_data_old is dropped later, after DRAM data is merged into ram_data


        -- 3. Populate devices and migrate gpu_data table
        ALTER TABLE gpu_data RENAME TO gpu_data_old;

        INSERT OR IGNORE INTO devices (kind, name) VALUES ('gpu', 'unknown');

        CREATE TABLE gpu_data (
            timestamp          INTEGER NOT NULL,
            duration_ms        INTEGER NOT NULL,
            device_id          INTEGER NOT NULL REFERENCES devices(id),
            total_energy_uj    INTEGER,
            usage_percent      REAL,
            vram_usage_percent REAL,
            PRIMARY KEY (timestamp, duration_ms, device_id)
        ) WITHOUT ROWID;

        INSERT OR IGNORE INTO gpu_data (timestamp, duration_ms, device_id, total_energy_uj, usage_percent, vram_usage_percent)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               d.id,
               CAST(g.total_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER),
               g.usage_percent,
               g.vram_usage_percent
        FROM gpu_data_old g
        JOIN timestamp_old t ON g.timestamp_id = t.id
        JOIN devices d ON d.kind = 'gpu' AND d.name = 'unknown';
        DROP TABLE gpu_data_old;

        -- 4. Migrate ram_data table
        ALTER TABLE ram_data RENAME TO ram_data_old;
        CREATE TABLE ram_data (
            timestamp       INTEGER NOT NULL,
            duration_ms     INTEGER NOT NULL,
            total_energy_uj INTEGER,
            usage_percent   REAL,
            PRIMARY KEY (timestamp, duration_ms)
        ) WITHOUT ROWID;
        INSERT OR IGNORE INTO ram_data (timestamp, duration_ms, total_energy_uj, usage_percent)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               CAST(r.total_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               r.usage_percent 
        FROM ram_data_old r JOIN timestamp_old t ON r.timestamp_id = t.id;
        -- Add historical DRAM energy (from cpu_data_old) into ram_data for timestamps that exist
        UPDATE ram_data
        SET total_energy_uj = COALESCE(ram_data.total_energy_uj, 0) + dram.dram_uj
        FROM (
            SELECT t.timestamp AS ts,
                   CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END AS dur,
                   CAST(c.dram_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER) AS dram_uj
            FROM cpu_data_old c
            JOIN timestamp_old t ON c.timestamp_id = t.id
            WHERE c.dram_power_watts IS NOT NULL AND c.dram_power_watts > 0
        ) AS dram
        WHERE ram_data.timestamp = dram.ts AND ram_data.duration_ms = dram.dur;
        DROP TABLE cpu_data_old;
        DROP TABLE ram_data_old;

        -- 5. Migrate disk_data table
        ALTER TABLE disk_data RENAME TO disk_data_old;
        CREATE TABLE disk_data (
            timestamp       INTEGER NOT NULL,
            duration_ms     INTEGER NOT NULL,
            total_energy_uj INTEGER,
            read_bytes      INTEGER,
            written_bytes   INTEGER,
            PRIMARY KEY (timestamp, duration_ms)
        ) WITHOUT ROWID;
        INSERT OR IGNORE INTO disk_data (timestamp, duration_ms, total_energy_uj, read_bytes, written_bytes)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               CAST(d.total_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               CAST(d.read_usage_mb_s * 1048576 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               CAST(d.write_usage_mb_s * 1048576 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER) 
        FROM disk_data_old d JOIN timestamp_old t ON d.timestamp_id = t.id;
        DROP TABLE disk_data_old;

        -- 6. Migrate network_data table
        ALTER TABLE network_data RENAME TO network_data_old;
        CREATE TABLE network_data (
            timestamp        INTEGER NOT NULL,
            duration_ms      INTEGER NOT NULL,
            total_energy_uj  INTEGER,
            downloaded_bytes INTEGER,
            uploaded_bytes   INTEGER,
            PRIMARY KEY (timestamp, duration_ms)
        ) WITHOUT ROWID;
        INSERT OR IGNORE INTO network_data (timestamp, duration_ms, total_energy_uj, downloaded_bytes, uploaded_bytes)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               CAST(n.total_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               CAST(n.download_speed_mb_s * 1048576 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               CAST(n.upload_speed_mb_s * 1048576 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER) 
        FROM network_data_old n JOIN timestamp_old t ON n.timestamp_id = t.id;
        DROP TABLE network_data_old;

        -- 7. Migrate total_data table
        ALTER TABLE total_data RENAME TO total_data_old;
        CREATE TABLE total_data (
            timestamp       INTEGER NOT NULL,
            duration_ms     INTEGER NOT NULL,
            total_energy_uj INTEGER,
            PRIMARY KEY (timestamp, duration_ms)
        ) WITHOUT ROWID;
        INSERT OR IGNORE INTO total_data (timestamp, duration_ms, total_energy_uj)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               CAST(o.total_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER) 
        FROM total_data_old o JOIN timestamp_old t ON o.timestamp_id = t.id;
        DROP TABLE total_data_old;

        -- 8. Populate apps and migrate process_data table
        ALTER TABLE process_data RENAME TO process_data_old;

        INSERT OR IGNORE INTO apps (identity, name, exe_path)
        SELECT DISTINCT COALESCE(process_exe_path, app_name), app_name, process_exe_path FROM process_data_old;

        CREATE TABLE process_data (
            timestamp         INTEGER NOT NULL,
            duration_ms       INTEGER NOT NULL,
            app_id            INTEGER NOT NULL REFERENCES apps(id),
            process_energy_uj INTEGER,
            process_cpu_usage REAL,
            process_gpu_usage REAL,
            process_mem_usage REAL,
            read_bytes        INTEGER,
            written_bytes     INTEGER,
            subprocess_count  INTEGER,
            PRIMARY KEY (timestamp, duration_ms, app_id)
        ) WITHOUT ROWID;

        INSERT OR IGNORE INTO process_data (timestamp, duration_ms, app_id, process_energy_uj, process_cpu_usage, process_gpu_usage, process_mem_usage, read_bytes, written_bytes, subprocess_count)
        SELECT t.timestamp,
               CASE WHEN t.period_type > 0 THEN t.period_type * 1000 ELSE 1000 END,
               a.id,
               CAST(p.process_power_watts * 1000000 * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               p.process_cpu_usage, p.process_gpu_usage, p.process_mem_usage, 
               CAST(p.read_bytes_per_sec * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               CAST(p.written_bytes_per_sec * CASE WHEN t.period_type > 0 THEN t.period_type ELSE 1 END AS INTEGER), 
               p.subprocess_count 
        FROM process_data_old p
        JOIN timestamp_old t ON p.timestamp_id = t.id
        JOIN apps a ON a.identity = COALESCE(p.process_exe_path, p.app_name);
        DROP TABLE process_data_old;

        -- 9. Migrate component_all_time_data table
        ALTER TABLE component_all_time_data RENAME TO component_all_time_data_old;
        CREATE TABLE component_all_time_data (
            id              INTEGER PRIMARY KEY,
            component_name  TEXT UNIQUE NOT NULL,
            total_energy_uj INTEGER NOT NULL DEFAULT 0
        );
        INSERT INTO component_all_time_data (id, component_name, total_energy_uj)
        SELECT id, component_name, CAST(total_energy_wh * 3600000000 AS INTEGER) FROM component_all_time_data_old;
        DROP TABLE component_all_time_data_old;

        -- 10. Drop the now-obsolete timestamp table and any other fully deprecated tables
        DROP TABLE timestamp_old;
        DROP TABLE IF EXISTS timestamp;
        DROP TABLE IF EXISTS all_time_data;
    "#.replace("'unknown'", &format!("'{}'", default_gpu.replace('\'', "''")));

    tx.execute_batch(&sql)?;

    Ok(())
}
