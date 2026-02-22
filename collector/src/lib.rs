#![allow(dead_code, unused_imports)]

pub mod database;
pub mod sensors;

use core::time;
use std::{
    cell::RefCell,
    rc::Rc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use common::database::purge::averaging_and_purging_data;
use database::Database;
use sensors::{AllTimeData, SensorType, create_event_from_sensors, get_hardware_info, gpu::get_gpu_list};
use sysinfo::System;

use crate::sensors::{DiskSensor, NetworkSensor, RamSensor};

pub struct CollectorApp {
    database: Database,
    sensors: Vec<SensorType>,
    all_time_data: AllTimeData,
    iteration: u64,
    system: Rc<RefCell<System>>,
}

impl CollectorApp {
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Failed to create database: {}", e))?;
        let s = System::new_all();
        Ok(CollectorApp {
            database,
            sensors: Vec::new(),
            all_time_data: AllTimeData::new(),
            iteration: 0,
            system: Rc::new(RefCell::new(s)),
        })
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        check_permissions()?;

        println!("\n========== INITIALIZING SYSTEM ==========\n");

        // Initialize CPU sensor
        println!("\nInitializing sensors...");
        let sensor_cpu = sensors::cpu::get_cpu_power_sensor(self.system.clone(), 0);
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

        //Initialize RAM, Disk, Network sensors
        self.sensors.push(SensorType::RAM(RamSensor::new(self.system.clone())));
        self.sensors.push(SensorType::Disk(DiskSensor::new()));
        self.sensors.push(SensorType::Network(NetworkSensor::new()));

        // Add total power sensor
        self.sensors.push(SensorType::Total);
        // Add process sensor
        self.sensors.push(SensorType::Process);
        // Add all-time data sensor
        self.sensors.push(SensorType::AllTime);

        println!("\n========== SETTING UP DATABASE ==========");
        // Initialize database
        let mut database = Database::new().map_err(|e| format!("Failed to open database: {}", e))?;
        let table_names: Vec<&str> = self.sensors.iter().map(|s| s.table_name()).collect();

        database
            .create_tables_if_not_exists(&table_names)
            .map_err(|e| format!("Failed to create database tables: {}", e))?;
        println!("✓ Database initialized");

        println!("\n========== GETTING ALL TIME DATA ==========");
        if let Ok(all_time) = database.get_all_time_data() {
            self.all_time_data = all_time;
            println!("✓ All-time data loaded from database");
        } else {
            println!("✗ No existing all-time data found, starting fresh");
        }

        Ok(())
    }

    pub fn run(&mut self) {
        println!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        self.get_hardware_info();

        println!("\n========== PURGING & AVERAGING OLD DATA ==========");
        // averaging data every hour and purge the database until the last X_hours
        averaging_and_purging_data(&mut self.database, 24, 24)
            .map_err(|e| format!("Failed averaging/purging data: {}", e))
            .ok();

        println!("\n========== POWER CONSUMPTION MONITORING ==========");
        println!("Logging data to database every second. Press Ctrl+C to stop.\n");

        loop {
            let start_time = Instant::now();
            println!("\n--- Iteration {} ---", self.iteration);

            let event = create_event_from_sensors(&self.sensors, self.system.clone(), &mut self.all_time_data);

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

            self.iteration += 1;

            // ADJUST SLEEP DURATION TO MAINTAIN 1 SECOND INTERVALS
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_millis(1000) {
                let now = SystemTime::now();
                let time_before_next_second = now
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_millis()
                    % 1000;
                thread::sleep(Duration::from_millis(1000 - time_before_next_second as u64));
            } else {
                println!(
                    "WARNING: Iteration {} took longer than 1 second. Consider optimizing.",
                    self.iteration
                );
            }
        }
    }

    pub fn get_hardware_info(&mut self) {
        let info = get_hardware_info(&self.sensors);

        match self.database.insert_hardware_info(&info) {
            Ok(_) => println!("✓ Hardware info saved to database"),
            Err(e) => eprintln!("✗ Failed to save hardware info to database: {:?}", e),
        }

        println!("{:#?}", info);
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
                std::env::current_exe().unwrap_or_else(|_| "<program>".into()).display()
            ))
        } else {
            Ok(())
        }
    }
}
