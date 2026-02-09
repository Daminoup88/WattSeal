#![allow(dead_code, unused_imports)]

pub mod database;
pub mod process;
pub mod sensors;

use core::time;
use std::{
    thread,
    time::{Duration, Instant, SystemTime},
};

use adlx::gpu;
use common::database::purge::averaging_and_purging_data;
use database::Database;
use display_info::DisplayInfo;
use process::{estimate_app_power_consumption, groups::group_processes_by_app};
use sensors::{SensorType, create_event_from_sensors, gpu::get_gpu_list};

pub struct CollectorApp {
    database: Database,
    sensors: Vec<SensorType>,
    iteration: u64,
}

impl CollectorApp {
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Failed to create database: {}", e))?;
        Ok(CollectorApp {
            database,
            sensors: Vec::new(),
            iteration: 0,
        })
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        check_permissions()?;

        // Initialize hardware information
        // let time = Instant::now();
        // let hw_info = match HardwareInfo::query() {
        //     Ok(info) => info,
        //     Err(e) => return Err(format!("Failed to query hardware information: {}", e)),
        // };
        // println!("Time taken to query hardware info: {:?}", time.elapsed());
        // println!("✓ Hardware information loaded");
        // println!("{:#?}", hw_info);

        println!("\n========== INITIALIZING SYSTEM ==========\n");

        // Initialize display information
        let display_infos = DisplayInfo::all().unwrap();
        for display_info in display_infos {
            println!("display_info {display_info:?}");
        }

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
        let gpu_list = get_gpu_list();
        print!("\nDetected GPUs:\n");
        println!("{:#?}", gpu_list);
        for (i, gpu_name) in gpu_list.iter().enumerate() {
            print!("  GPU {}: {}\n", i, gpu_name);
            let sensor_gpu = sensors::gpu::get_gpu_power_sensor(gpu_name, i as u32);
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

        // Add total power sensor
        self.sensors.push(SensorType::Total);
        // Add process sensor
        self.sensors.push(SensorType::Process);

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
        println!("\n========== PURGING & AVERAGING OLD DATA ==========");
        // averaging data every hour and purge the database until the last X_hours
        averaging_and_purging_data(&mut self.database, 24, 24)
            .map_err(|e| format!("Failed averaging/purging data: {}", e))
            .unwrap();

        println!("\n========== POWER CONSUMPTION MONITORING ==========");
        println!("Logging data to database every second. Press Ctrl+C to stop.\n");

        loop {
            let start_time = Instant::now();

            self.iteration += 1;
            println!("\n--- Iteration {} ---", self.iteration);

            let event = create_event_from_sensors(&self.sensors);

            match self.database.insert_event(&event) {
                Ok(_) => println!("✓ Event data saved to database"),
                Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
            }
            for sensor_data in event.data().iter() {
                // PRINT HARDWARE DATA
                if !(sensor_data.sensor_type() == "Processes") {
                    println!("{}", sensor_data);
                }
                // PRINT PROCESS DATA
                else {
                    println!("{}", sensor_data);
                }
            }

            // ADJUST SLEEP DURATION TO MAINTAIN 1 SECOND INTERVALS
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_millis(1000) {
                let now = SystemTime::now();
                let time_before_next_second = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() % 1000;
                thread::sleep(Duration::from_millis(1000 - time_before_next_second as u64));
            } else {
                println!(
                    "WARNING: Iteration {} took longer than 1 second. Consider optimizing.",
                    self.iteration
                );
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
