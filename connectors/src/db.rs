use std::{
    thread,
    time::{Duration, Instant},
};

use common::{
    MetricSample, clog,
    database::{Database, purge::averaging_and_purging_data},
    types::HardwareInfo,
};

/// Connector that writes collected sensor metrics into the local SQLite database.
pub struct DatabaseConnector {
    database: Database,
    last_purge: Instant,
}

impl DatabaseConnector {
    /// Initializes the database connection, creates default tables, saves hardware info,
    /// and triggers initial data averaging and purging in the background.
    pub fn new(hardware: &HardwareInfo) -> Result<Self, String> {
        clog!("\n========== SETTING UP DATABASE ==========");
        let mut db = Database::new().map_err(|e| format!("Failed to create database: {e}"))?;
        db.create_tables_if_not_exists(hardware)
            .map_err(|e| format!("Failed to create database tables: {e}"))?;
        clog!("✓ Database tables initialized");

        match db.insert_hardware_info(hardware) {
            Ok(_) => clog!("✓ Hardware info saved to database"),
            Err(e) => clog!("✗ Failed to save hardware info to database: {e}"),
        }

        let mut connector = Self {
            database: db,
            last_purge: Instant::now(),
        };
        connector.purge_and_average();
        Ok(connector)
    }

    /// Spawns a background thread to aggregate raw data into hourly averages and purge expired entries.
    pub fn purge_and_average(&mut self) {
        thread::spawn(|| {
            if let Ok(mut db) = Database::new() {
                let _ = averaging_and_purging_data(&mut db, 24, 24);
            }
        });
        self.last_purge = Instant::now();
    }

    /// Inserts a newly collected metric sample into the database.
    pub fn handle_sample(&mut self, sample: &MetricSample) -> Result<(), String> {
        if self.last_purge.elapsed() > Duration::from_secs(24 * 3600) {
            self.purge_and_average();
        }

        #[cfg(debug_assertions)]
        let start = Instant::now();

        let result = self.database.insert_sample_and_update_energy(sample);

        #[cfg(debug_assertions)]
        {
            let duration = start.elapsed();
            match &result {
                Ok(_) => println!("✓ Event data saved to database (took {:.2?})", duration),
                Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
            }
        }

        result.map_err(|e| format!("Failed to save event data to database: {e}"))
    }
}
