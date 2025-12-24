#![allow(dead_code, unused_imports)]

pub mod database;
pub mod sensors;

use std::{thread, time::Duration};

use database::Database;
use hardware_query::HardwareInfo;
use sensors::{SensorType, create_event_from_sensors};

pub struct CollectorApp {
    database: Database,
    sensors: Vec<SensorType>,
}

impl CollectorApp {
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Failed to create database: {}", e))?;
        Ok(CollectorApp {
            database,
            sensors: Vec::new(),
        })
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        check_permissions()?;

        println!("\n========== INITIALIZING SYSTEM ==========\n");
        // Initialize hardware information
        let hw_info = match HardwareInfo::query() {
            Ok(info) => info,
            Err(e) => return Err(format!("Failed to query hardware information: {}", e)),
        };
        println!("✓ Hardware information loaded");
        println!("{:#?}", hw_info);

        // Initialize CPU sensor
        println!("\nInitializing sensors...");
        let sensor_cpu = sensors::cpu::get_cpu_power_sensor(0);
        match sensor_cpu {
            Ok(sensor) => {
                println!("✓ CPU Power Sensor initialized successfully");
                self.sensors.push(sensor);
            }
            Err(e) => {
                eprintln!("✗ Failed to initialize CPU Power Sensor: {:?}", e);
            }
        }

        // Initialize GPU sensors
        for (i, gpu) in hw_info.gpus().iter().enumerate() {
            let gpu_name = format!("{} {}", gpu.vendor(), gpu.model_name());
            let sensor_gpu = sensors::gpu::get_gpu_power_sensor(&gpu_name, i as u32);
            match sensor_gpu {
                Ok(sensor) => {
                    println!("✓ GPU Sensor {} initialized for: {}", i, gpu_name);
                    self.sensors.push(sensor);
                }
                Err(e) => {
                    println!("✗ Failed to initialize GPU sensor for {}: {:?}", gpu_name, e);
                }
            }
        }

        println!("\n========== SETTING UP DATABASE ==========");
        // Initialize database
        let mut database = Database::new().map_err(|e| format!("Failed to open database: {}", e))?;
        database
            .create_tables_if_not_exists(&self.sensors)
            .map_err(|e| format!("Failed to create database tables: {}", e))?;
        println!("✓ Database initialized");
        Ok(())
    }

    pub fn run(&mut self) {
        println!("\n========== POWER CONSUMPTION MONITORING ==========");
        println!("Logging data to database every second. Press Ctrl+C to stop.\n");

        let mut iteration = 0;
        loop {
            thread::sleep(Duration::from_millis(1000));
            iteration += 1;

            println!("\n--- Iteration {} ---", iteration);

            let event = create_event_from_sensors(&self.sensors);
            match self.database.insert_event(&event) {
                Ok(_) => println!("✓ Event data saved to database"),
                Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
            }
        }
    }
}

fn check_permissions() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if !is_admin::is_admin() {
            Err(
                "This program requires Administrator privileges on Windows. Please run this program as Administrator."
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    {
        if !is_root() {
            Err(format!(
                "This program requires root privileges on Linux. Please run with: sudo {}",
                std::env::current_exe().unwrap().display()
            ))
        } else {
            Ok(())
        }
    }
}
