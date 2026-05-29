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
    DatabaseEntry, EnergyWH, PowerWatt, ProcessDataDB, SensorData, TotalDataDB,
    database::{purge::averaging_and_purging_data, types::GeneralDataDB},
};
use database::{Database, consumption_event_to_eventdb};
use mqtt::{
    MQTTPublisher,
    topics::{hardware_info_topic, sensor_data_to_topic},
};
use sensors::{
    DiskSensor, NetworkSensor, RamSensor, SensorType, create_event_from_sensors, get_hardware_info,
    get_process_gpu_usage,
    gpu::{GPUVendor, get_gpu_list},
};
use sysinfo::System;

/// Possible units to choose as output
#[derive(Debug, Clone, Copy)]
pub enum ConsumptionUnit {
    Watt,
    WattHour,
    UJoul,
}

/// MQTT information to interact with a MQTT client
pub struct MQTTInfos {
    id: String,
    publisher: MQTTPublisher<rumqttc::Client>,
    unit: Option<ConsumptionUnit>,
}

/// Background sensor-collection application.
pub struct CollectorApp {
    database: Option<Database>,
    mqtt_infos: Option<MQTTInfos>,
    sensors: Vec<SensorType>,
    system: Rc<RefCell<System>>,
    last_update: Instant,
    last_purge: Instant,
    #[cfg(debug_assertions)]
    iteration: u64,
}

impl MQTTInfos {
    pub fn new(id: &str, addr: &SocketAddr, unit: Option<ConsumptionUnit>) -> Self {
        let publisher = MQTTPublisher::new_from_addr(addr);
        MQTTInfos {
            id: id.to_string(),
            publisher,
            unit,
        }
    }
}

impl CollectorApp {
    /// Creates a new collector with a database connection.
    pub fn new(enable_save_db: bool, mqtt_infos: Option<MQTTInfos>) -> Result<Self, String> {
        let database;
        if enable_save_db {
            database = Some(Database::new().map_err(|e| format!("Failed to create database: {e}"))?);
        } else {
            database = None
        }
        let s = System::new_all();
        Ok(CollectorApp {
            database,
            mqtt_infos,
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

            match sensors::gpu::get_gpu_consumption_sensor(gpu_name, vendor_index) {
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

        if let Some(database) = &mut self.database {
            // Database
            crate::clog!("\n========== SETTING UP DATABASE ==========");
            let mut table_names: Vec<&str> = self.sensors.iter().map(|s| s.table_name()).collect();
            table_names.push(ProcessDataDB::table_name_static());
            table_names.push(TotalDataDB::table_name_static());
            database
                .create_tables_if_not_exists(&table_names)
                .map_err(|e| format!("Failed to create database tables: {e}"))?;
            crate::clog!("✓ Database initialized");
        }

        // Hardware info
        crate::clog!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        let info = get_hardware_info(&self.sensors);

        if let Some(database) = &mut self.database {
            let infodb = GeneralDataDB::from(info.clone());
            match database.insert_hardware_info(&infodb) {
                Ok(_) => crate::clog!("✓ Hardware info saved"),
                Err(e) => crate::clog!("✗ Failed to save hardware info: {e}"),
            }
        }
        if let Some(mqtt_infos) = &self.mqtt_infos {
            let topic = hardware_info_topic(&mqtt_infos.id);
            match mqtt_infos.publisher.publish(&topic, &info.hardware_info.serialized()) {
                Ok(_) => crate::clog!("✓ Hardware info published on broker"),
                Err(e) => crate::clog!("✗ Failed to publish hardware info: {e}"),
            }
        }
        crate::clog!("Initialization complete");
        Ok(())
    }

    /// Runs the collection loop, sampling sensors every second.
    pub fn run(&mut self) {
        // Purge/averaging runs in a separate thread so collection starts immediately.
        self.purge_and_average();

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

            let event = create_event_from_sensors(&self.sensors, since_last_update);

            let proc_gpu_usage = get_process_gpu_usage(&self.sensors);

            if let Some(database) = &mut self.database {
                let eventdb = consumption_event_to_eventdb(&event, self.system.clone(), proc_gpu_usage);

                #[cfg(debug_assertions)]
                {
                    let start = Instant::now();
                    let result = database.insert_event_and_update_energy(&eventdb, since_last_update.as_secs_f64());

                    let duration = start.elapsed();
                    match result {
                        Ok(_) => println!("✓ Event data saved to database (took {:.2?})", duration),
                        Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
                    }
                }

                #[cfg(not(debug_assertions))]
                let _ = database.insert_event_and_update_energy(&eventdb, since_last_update.as_secs_f64());
            }

            if let Some(mqtt_infos) = &self.mqtt_infos {
                for sensor_data in event.data() {
                    let topic = sensor_data_to_topic(&mqtt_infos.id, &sensor_data);

                    let _result = mqtt_infos.unit.map_or_else(
                        || mqtt_infos.publisher.publish(&topic, sensor_data),
                        |u| match u {
                            ConsumptionUnit::UJoul => mqtt_infos.publisher.publish(&topic, &sensor_data),
                            ConsumptionUnit::WattHour => {
                                let sensor_data_wh: SensorData<EnergyWH> = sensor_data.to_wh();
                                mqtt_infos.publisher.publish(&topic, &sensor_data_wh)
                            }
                            ConsumptionUnit::Watt => {
                                let sensor_data_w: SensorData<PowerWatt> = sensor_data.to_watts(since_last_update);
                                mqtt_infos.publisher.publish(&topic, &sensor_data_w)
                            }
                        },
                    );
                    #[cfg(debug_assertions)]
                    match _result {
                        Ok(_) => println!("✓ Sensor data published on topic {}", topic),
                        Err(e) => eprintln!("✗ Failed to publish sensor data on topic {}: {:?}", topic, e),
                    }
                }
            }

            #[cfg(debug_assertions)]
            for sensor_data in event.data() {
                println!("{sensor_data}");
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
