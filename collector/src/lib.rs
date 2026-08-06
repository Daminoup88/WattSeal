pub mod database;
pub mod sensors;

use std::{
    cell::RefCell,
    net::SocketAddr,
    rc::Rc,
    thread,
    time::{Duration, Instant, SystemTime},
};

pub use common::clog;
#[cfg(not(debug_assertions))]
use common::logging::start_log_session;
use common::{
    DatabaseEntry, ProcessData, TotalData, database::purge::averaging_and_purging_data, types::PublishableSensor,
};
use database::Database;
use mqtt::{
    MQTTPublisher,
    topics::{hardware_info_topic, sensor_type_to_topic},
};
use sensors::{
    DiskSensor, NetworkSensor, RamSensor, SensorType, create_event_from_sensors, get_hardware_info,
    gpu::{GPUVendor, get_gpu_list},
};
use sysinfo::System;

use crate::sensors::to_computed_event;

/// Possible units to choose as output
#[derive(Debug, Default, Clone, Copy)]
pub enum ConsumptionUnit {
    WattHour,
    #[default]
    UJoul,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum MqttDataMode {
    #[default]
    Computed,
    Raw,
}

/// MQTT information to interact with a MQTT client
pub struct MQTTInfo {
    id: String,
    publisher: MQTTPublisher<rumqttc::Client>,
    unit: Option<ConsumptionUnit>,
    data_mode: MqttDataMode,
}

/// Background sensor-collection application.
pub struct CollectorApp {
    database: Option<Database>,
    mqtt_info: Option<MQTTInfo>,
    sensors: Vec<SensorType>,
    system: Rc<RefCell<System>>,
    last_update: Instant,
    last_purge: Instant,
    #[cfg(debug_assertions)]
    iteration: u64,
}

impl MQTTInfo {
    pub fn new(id: &str, addr: &SocketAddr, unit: Option<ConsumptionUnit>, is_raw_mode: bool) -> Self {
        let publisher = MQTTPublisher::new_from_addr(addr);
        MQTTInfo {
            id: id.to_string(),
            publisher,
            unit,
            data_mode: is_raw_mode.then_some(MqttDataMode::Raw).unwrap_or_default(),
        }
    }
}

impl CollectorApp {
    /// Creates a new collector with a database connection.
    pub fn new(enable_save_db: bool, mqtt_info: Option<MQTTInfo>) -> Result<Self, String> {
        let database;
        if enable_save_db {
            database = Some(Database::new().map_err(|e| format!("Failed to create database: {e}"))?);
        } else {
            database = None
        }
        let s = System::new_all();
        Ok(CollectorApp {
            database,
            mqtt_info,
            sensors: Vec::new(),
            system: Rc::new(RefCell::new(s)),
            last_update: Instant::now(),
            last_purge: Instant::now(),
            #[cfg(debug_assertions)]
            iteration: 0,
        })
    }

    fn purge_and_average(&mut self) {
        thread::spawn(|| {
            if let Ok(mut db) = Database::new() {
                let _ = averaging_and_purging_data(&mut db, 24, 24);
            }
        });
        self.last_purge = Instant::now();
    }

    /// Detects hardware sensors, creates database tables, and saves hardware info.
    pub fn initialize(&mut self) -> Result<(), String> {
        #[cfg(not(debug_assertions))]
        start_log_session();

        crate::clog!("\n========== INITIALIZING SYSTEM ==========\n");

        // CPU sensor
        crate::clog!("Initializing sensors...");
        match sensors::cpu::get_cpu_power_sensor(self.system.clone(), 0) {
            Ok(sensor) => {
                if let SensorType::CPU(cpu_sensor) = &sensor {
                    let (os_label, mode_label) = cpu_sensor.power_mode_labels();
                    crate::clog!("CPU power mode: {os_label}/{mode_label}");
                }
                crate::clog!("✓ CPU Power Sensor initialized successfully");
                self.sensors.push(sensor);
            }
            Err(e) => crate::clog!("✗ Failed to initialize CPU Power Sensor: {:?}", e),
        }

        // GPU sensors
        let gpu_list = get_gpu_list();
        crate::clog!("\nDetected GPUs: {gpu_list:#?}");
        if gpu_list.is_empty() {
            crate::clog!("⚠ No supported GPU adapters detected.");
        }

        let (mut nvidia_index, mut amd_index, mut intel_index) = (0u32, 0u32, 0u32);
        for gpu_name in &gpu_list {
            let vendor = GPUVendor::from_str(gpu_name);
            let vendor_index = match vendor {
                GPUVendor::Nvidia => {
                    let idx = nvidia_index;
                    nvidia_index += 1;
                    idx
                }
                GPUVendor::Amd => {
                    let idx = amd_index;
                    amd_index += 1;
                    idx
                }
                GPUVendor::Intel => {
                    let idx = intel_index;
                    intel_index += 1;
                    idx
                }
                GPUVendor::Other => 0,
            };

            match sensors::gpu::get_gpu_energy_sensor(gpu_name, vendor_index) {
                Ok(sensor) => {
                    crate::clog!(
                        "✓ GPU sensor initialized: '{}' (vendor={:?}, vendor_index={})",
                        gpu_name,
                        vendor,
                        vendor_index
                    );
                    self.sensors.push(sensor);
                }
                Err(e) => {
                    crate::clog!(
                        "✗ Failed to initialize GPU sensor for '{}' (vendor={:?}, vendor_index={}): {:?}",
                        gpu_name,
                        vendor,
                        vendor_index,
                        e
                    );
                }
            }
        }

        // RAM, Disk, Network sensors
        self.sensors.push(SensorType::RAM(RamSensor::new(self.system.clone())));
        self.sensors.push(SensorType::Disk(DiskSensor::new()));
        self.sensors.push(SensorType::Network(NetworkSensor::new()));

        let mut table_names: Vec<String> = self.sensors.iter().map(|s| s.table_name().into()).collect();
        table_names.push(ProcessData::table_name_static().into());
        table_names.push(TotalData::table_name_static().into());

        if let Some(database) = &mut self.database {
            // Database
            crate::clog!("\n========== SETTING UP DATABASE ==========");
            database
                .create_tables_if_not_exists(&table_names)
                .map_err(|e| format!("Failed to create database tables: {e}"))?;
            crate::clog!("✓ Database initialized");
        }

        // Hardware info
        crate::clog!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        let info = get_hardware_info(&self.sensors, &table_names);

        if let Some(database) = &mut self.database {
            match database.insert_hardware_info(&info) {
                Ok(_) => crate::clog!("✓ Hardware info saved"),
                Err(e) => crate::clog!("✗ Failed to save hardware info: {e}"),
            }
        }
        if let Some(mqtt_info) = &self.mqtt_info {
            let topic = hardware_info_topic(&mqtt_info.id);
            match mqtt_info.publisher.publish(&topic, &info.hardware_info) {
                Ok(_) => crate::clog!("✓ Hardware info published on broker"),
                Err(e) => crate::clog!("✗ Failed to publish hardware info: {e}"),
            }
        }
        crate::clog!("Initialization complete");
        Ok(())
    }

    /// Runs the collection loop, sampling sensors every second.
    pub fn run(&mut self) {
        let start = Instant::now();
        // Purge/averaging runs in a separate thread so collection starts immediately.
        self.purge_and_average();
        // Wait at least 1 second before first collection to avoid skewed readings
        let duration = start.elapsed();
        if duration < Duration::from_secs(1) {
            thread::sleep(Duration::from_secs(1) - duration);
        }

        #[cfg(debug_assertions)]
        println!(
            "\n========== POWER CONSUMPTION MONITORING ==========\nLogging to database every second. Press Ctrl+C to stop.\n"
        );

        loop {
            if self.last_purge.elapsed() > Duration::from_secs(24 * 3600) {
                self.purge_and_average();
            }
            let start_time = Instant::now();
            let since_last_update = self.last_update.elapsed();
            self.last_update = start_time;

            #[cfg(debug_assertions)]
            println!("\n--- Iteration {} ---", self.iteration);

            let event = create_event_from_sensors(&self.sensors, self.system.clone(), since_last_update);

            let needs_computed = self.database.is_some()
                || self
                    .mqtt_info
                    .as_ref()
                    .is_some_and(|m| m.data_mode == MqttDataMode::Computed);

            let computed_event = needs_computed.then(|| to_computed_event(&event));

            if let Some(database) = &mut self.database {
                if let Some(computed_event) = &computed_event {
                    #[cfg(debug_assertions)]
                    {
                        let start = Instant::now();

                        let result = database.insert_event_and_update_energy(&computed_event, since_last_update);
                        let duration = start.elapsed();
                        match result {
                            Ok(_) => println!("✓ Event data saved to database (took {:.2?})", duration),
                            Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
                        }
                    }

                    #[cfg(not(debug_assertions))]
                    let _ = database.insert_event_and_update_energy(&computed_event, since_last_update);
                }
            }

            if let Some(mqtt_info) = &self.mqtt_info {
                match mqtt_info.data_mode {
                    MqttDataMode::Raw => {
                        for sensor_data in event.data() {
                            publish_sensor(mqtt_info, sensor_data);
                        }
                    }
                    MqttDataMode::Computed => {
                        if let Some(computed) = &computed_event {
                            for sensor_data in computed.data() {
                                publish_sensor(mqtt_info, sensor_data);
                            }
                        }
                    }
                }
            }

            #[cfg(debug_assertions)]
            {
                if let Some(computed) = &computed_event {
                    for sensor_data in computed.data() {
                        println!("{sensor_data}");
                    }
                } else {
                    for sensor_data in event.data() {
                        println!("{sensor_data}");
                    }
                }
            }

            #[cfg(debug_assertions)]
            {
                self.iteration += 1;
                if start_time.elapsed() > Duration::from_millis(1000) {
                    eprintln!("WARNING: Iteration {} took longer than 1 second.", self.iteration);
                }
            }

            // Adjust sleep duration to maintain 1 second interval
            let now_sub_ms = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis()
                % 1000;
            if now_sub_ms < 1000 {
                thread::sleep(Duration::from_millis(1000 - now_sub_ms as u64));
            }
        }
    }
}

fn publish_sensor<T: PublishableSensor>(mqtt_info: &MQTTInfo, sensor_data: &T) {
    let topic = sensor_type_to_topic(&mqtt_info.id, sensor_data.sensor_type());
    let _result = match mqtt_info.unit {
        Some(ConsumptionUnit::WattHour) => mqtt_info.publisher.publish(&topic, &sensor_data.to_wh()),
        _ => mqtt_info.publisher.publish(&topic, sensor_data),
    };
    #[cfg(debug_assertions)]
    match _result {
        Ok(_) => println!("✓ Sensor data published on topic {}", topic),
        Err(e) => eprintln!("✗ Failed to publish sensor data on topic {}: {:?}", topic, e),
    }
}
