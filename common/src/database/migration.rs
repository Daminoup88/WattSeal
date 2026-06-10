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
    crate::clog!("Migration to version {} complete.", DATABASE_TARGET_VERSION);

    Ok(())
}

fn migrate_v1_to_v2(tx: &rusqlite::Transaction) -> Result<(), DatabaseError> {
    // Defer foreign key checks until the transaction commits so we can swap tables safely
    tx.execute_batch("PRAGMA defer_foreign_keys = ON;")?;

    tx.execute_batch(
        r#"
        -- 1. Prepare timestamp table migration (rename period_type to sampling_period)
        ALTER TABLE timestamp RENAME TO timestamp_old;
        CREATE TABLE timestamp (
            id              INTEGER PRIMARY KEY,
            timestamp       INTEGER NOT NULL,
            sampling_period INTEGER NOT NULL
        );
        INSERT INTO timestamp (id, timestamp, sampling_period)
        SELECT id, timestamp, CASE WHEN period_type > 0 THEN period_type ELSE 1 END FROM timestamp_old;

        -- 2. Migrate cpu_data table (Watts -> Microjoules)
        ALTER TABLE cpu_data RENAME TO cpu_data_old;
        CREATE TABLE cpu_data (
            id              INTEGER PRIMARY KEY,
            timestamp_id    INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            total_energy_uj INTEGER,
            pp0_energy_uj   INTEGER,
            pp1_energy_uj   INTEGER,
            dram_energy_uj  INTEGER,
            usage_percent   REAL
        );
        INSERT INTO cpu_data (id, timestamp_id, total_energy_uj, pp0_energy_uj, pp1_energy_uj, dram_energy_uj, usage_percent)
        SELECT c.id, c.timestamp_id, 
               CAST(c.total_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               CAST(c.pp0_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               CAST(c.pp1_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               CAST(c.dram_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               c.usage_percent 
        FROM cpu_data_old c JOIN timestamp t ON c.timestamp_id = t.id;
        DROP TABLE cpu_data_old;
        CREATE INDEX idx_cpu_data_timestamp_id ON cpu_data(timestamp_id);

        -- 3. Migrate gpu_data table
        ALTER TABLE gpu_data RENAME TO gpu_data_old;
        CREATE TABLE gpu_data (
            id                 INTEGER PRIMARY KEY,
            timestamp_id       INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            total_energy_uj    INTEGER,
            usage_percent      REAL,
            vram_usage_percent REAL
        );
        INSERT INTO gpu_data (id, timestamp_id, total_energy_uj, usage_percent, vram_usage_percent)
        SELECT g.id, g.timestamp_id, 
               CAST(g.total_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               g.usage_percent, 
               g.vram_usage_percent 
        FROM gpu_data_old g JOIN timestamp t ON g.timestamp_id = t.id;
        DROP TABLE gpu_data_old;
        CREATE INDEX idx_gpu_data_timestamp_id ON gpu_data(timestamp_id);

        -- 4. Migrate ram_data table
        ALTER TABLE ram_data RENAME TO ram_data_old;
        CREATE TABLE ram_data (
            id              INTEGER PRIMARY KEY,
            timestamp_id    INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            total_energy_uj INTEGER,
            usage_percent   REAL
        );
        INSERT INTO ram_data (id, timestamp_id, total_energy_uj, usage_percent)
        SELECT r.id, r.timestamp_id, 
               CAST(r.total_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               r.usage_percent 
        FROM ram_data_old r JOIN timestamp t ON r.timestamp_id = t.id;
        DROP TABLE ram_data_old;
        CREATE INDEX idx_ram_data_timestamp_id ON ram_data(timestamp_id);

        -- 5. Migrate disk_data table (convert MB/s rate to total cumulative bytes over the period)
        ALTER TABLE disk_data RENAME TO disk_data_old;
        CREATE TABLE disk_data (
            id              INTEGER PRIMARY KEY,
            timestamp_id    INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            total_energy_uj INTEGER,
            read_bytes      INTEGER,
            written_bytes   INTEGER
        );
        INSERT INTO disk_data (id, timestamp_id, total_energy_uj, read_bytes, written_bytes)
        SELECT d.id, d.timestamp_id, 
               CAST(d.total_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               CAST(d.read_usage_mb_s * 1048576 * t.sampling_period AS INTEGER), 
               CAST(d.write_usage_mb_s * 1048576 * t.sampling_period AS INTEGER) 
        FROM disk_data_old d JOIN timestamp t ON d.timestamp_id = t.id;
        DROP TABLE disk_data_old;
        CREATE INDEX idx_disk_data_timestamp_id ON disk_data(timestamp_id);

        -- 6. Migrate network_data table (convert MB/s rate to total cumulative bytes over the period)
        ALTER TABLE network_data RENAME TO network_data_old;
        CREATE TABLE network_data (
            id               INTEGER PRIMARY KEY,
            timestamp_id     INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            total_energy_uj  INTEGER,
            downloaded_bytes INTEGER,
            uploaded_bytes   INTEGER
        );
        INSERT INTO network_data (id, timestamp_id, total_energy_uj, downloaded_bytes, uploaded_bytes)
        SELECT n.id, n.timestamp_id, 
               CAST(n.total_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               CAST(n.download_speed_mb_s * 1048576 * t.sampling_period AS INTEGER), 
               CAST(n.upload_speed_mb_s * 1048576 * t.sampling_period AS INTEGER) 
        FROM network_data_old n JOIN timestamp t ON n.timestamp_id = t.id;
        DROP TABLE network_data_old;
        CREATE INDEX idx_network_data_timestamp_id ON network_data(timestamp_id);

        -- 7. Migrate total_data table (removes unused period_type TEXT column)
        ALTER TABLE total_data RENAME TO total_data_old;
        CREATE TABLE total_data (
            id              INTEGER PRIMARY KEY,
            timestamp_id    INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            total_energy_uj INTEGER
        );
        INSERT INTO total_data (id, timestamp_id, total_energy_uj)
        SELECT o.id, o.timestamp_id, 
               CAST(o.total_power_watts * 1000000 * t.sampling_period AS INTEGER) 
        FROM total_data_old o JOIN timestamp t ON o.timestamp_id = t.id;
        DROP TABLE total_data_old;
        CREATE INDEX idx_total_data_timestamp_id ON total_data(timestamp_id);

        -- 8. Migrate process_data table (convert bytes/sec rates to raw historical interval byte counts)
        ALTER TABLE process_data RENAME TO process_data_old;
        CREATE TABLE process_data (
            id                INTEGER PRIMARY KEY,
            timestamp_id      INTEGER NOT NULL REFERENCES timestamp(id) ON DELETE CASCADE,
            app_name          TEXT NOT NULL,
            process_exe_path  TEXT,
            process_energy_uj INTEGER,
            process_cpu_usage REAL,
            process_gpu_usage REAL,
            process_mem_usage REAL,
            read_bytes        INTEGER,
            written_bytes     INTEGER,
            subprocess_count  INTEGER
        );
        INSERT INTO process_data (id, timestamp_id, app_name, process_exe_path, process_energy_uj, process_cpu_usage, process_gpu_usage, process_mem_usage, read_bytes, written_bytes, subprocess_count)
        SELECT p.id, p.timestamp_id, p.app_name, p.process_exe_path, 
               CAST(p.process_power_watts * 1000000 * t.sampling_period AS INTEGER), 
               p.process_cpu_usage, p.process_gpu_usage, p.process_mem_usage, 
               CAST(p.read_bytes_per_sec * t.sampling_period AS INTEGER), 
               CAST(p.written_bytes_per_sec * t.sampling_period AS INTEGER), 
               p.subprocess_count 
        FROM process_data_old p JOIN timestamp t ON p.timestamp_id = t.id;
        DROP TABLE process_data_old;
        CREATE INDEX idx_process_data_timestamp_id ON process_data(timestamp_id);

        -- 9. All old child tables are dropped; it is now perfectly safe to drop timestamp_old!
        DROP TABLE timestamp_old;

        -- 10. Migrate component_all_time_data table (convert Wh to microjoules: 1 Wh = 3,600,000,000 uJ)
        ALTER TABLE component_all_time_data RENAME TO component_all_time_data_old;
        CREATE TABLE component_all_time_data (
            id              INTEGER PRIMARY KEY,
            component_name  TEXT UNIQUE NOT NULL,
            total_energy_uj INTEGER NOT NULL DEFAULT 0
        );
        INSERT INTO component_all_time_data (id, component_name, total_energy_uj)
        SELECT id, component_name, CAST(total_energy_wh * 3600000000 AS INTEGER) FROM component_all_time_data_old;
        DROP TABLE component_all_time_data_old;

        -- 11. Clean up fully deprecated tables
        DROP TABLE IF EXISTS all_time_data;
        "#,
    )?;

    Ok(())
}
