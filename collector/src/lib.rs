pub mod database;
pub mod sensors;

use std::{
    cell::RefCell,
    net::SocketAddr,
    rc::Rc,
    thread,
    time::{Duration, Instant, SystemTime},
};

#[cfg(not(debug_assertions))]
use common::logging::start_log_session;
use common::{clog, database::purge::averaging_and_purging_data};
use database::Database;
use mqtt::{
    MQTTPublisher,
    topics::{hardware_info_topic, sensor_data_to_topic},
};
use sensors::{SensorType, create_event_from_sensors, get_hardware_info, gpu::get_gpu_list};
use sysinfo::System;

use crate::sensors::{DiskSensor, NetworkSensor, RamSensor};

/// MQTT information to interact with a MQTT client
pub struct MQTTInfos {
    id: String,
    publisher: MQTTPublisher<rumqttc::Client>,
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
    pub fn new(id: &str, addr: &SocketAddr) -> Self {
        let publisher = MQTTPublisher::new_from_addr(addr);
        MQTTInfos {
            id: id.to_string(),
            publisher,
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
        let is_admin = is_admin();

        #[cfg(not(debug_assertions))]
        start_log_session();

        clog!("\n========== INITIALIZING SYSTEM ==========\n");

        // CPU sensor
        clog!("Initializing sensors...");
        match sensors::cpu::get_cpu_power_sensor(self.system.clone(), 0, is_admin) {
            Ok(sensor) => {
                clog!("✓ CPU Power Sensor initialized successfully");
                self.sensors.push(sensor);
            }
            Err(e) => clog!("✗ Failed to initialize CPU Power Sensor: {:?}", e),
        }

        // GPU sensors
        let gpu_list = get_gpu_list();
        clog!("\nDetected GPUs: {gpu_list:#?}");
        for (i, gpu_name) in gpu_list.iter().enumerate() {
            match sensors::gpu::get_gpu_power_sensor(gpu_name, i as u32) {
                Ok(sensor) => {
                    clog!("✓ GPU Sensor {i} initialized: {gpu_name}");
                    self.sensors.push(sensor);
                }
                Err(e) => clog!("✗ Failed to initialize GPU sensor for {gpu_name}: {:?}", e),
            }
        }

        // RAM, Disk, Network sensors
        self.sensors.push(SensorType::RAM(RamSensor::new(self.system.clone())));
        self.sensors.push(SensorType::Disk(DiskSensor::new()));
        self.sensors.push(SensorType::Network(NetworkSensor::new()));
        self.sensors.push(SensorType::Total);
        self.sensors.push(SensorType::Process);

        if let Some(database) = &mut self.database {
            // Database
            clog!("\n========== SETTING UP DATABASE ==========");
            let table_names: Vec<&str> = self.sensors.iter().map(|s| s.table_name()).collect();
            database
                .create_tables_if_not_exists(&table_names)
                .map_err(|e| format!("Failed to create database tables: {e}"))?;
            clog!("✓ Database initialized");
        }

        // Hardware info
        clog!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        let info = get_hardware_info(&self.sensors);

        if let Some(database) = &mut self.database {
            match database.insert_hardware_info(&info) {
                Ok(_) => clog!("✓ Hardware info saved"),
                Err(e) => clog!("✗ Failed to save hardware info: {e}"),
            }
        }
        if let Some(mqtt_infos) = &self.mqtt_infos {
            let topic = hardware_info_topic(&mqtt_infos.id);
            match mqtt_infos.publisher.publish(&topic, &info.hardware_info_serialized) {
                Ok(_) => clog!("✓ Hardware info published on broker"),
                Err(e) => clog!("✗ Failed to publish hardware info: {e}"),
            }
        }
        clog!("Initialization complete");
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
            let since_last_update_secs = self.last_update.elapsed().as_secs_f64();
            self.last_update = start_time;

            #[cfg(debug_assertions)]
            println!("\n--- Iteration {} ---", self.iteration);

            let event = create_event_from_sensors(&self.sensors, self.system.clone());

            if let Some(database) = &mut self.database {
                #[cfg(debug_assertions)]
                {
                    let start = Instant::now();
                    let result = database.insert_event_and_update_energy(&event, since_last_update_secs);
                    let duration = start.elapsed();
                    match result {
                        Ok(_) => println!("✓ Event data saved to database (took {:.2?})", duration),
                        Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
                    }
                }

                #[cfg(not(debug_assertions))]
                let _ = database.insert_event_and_update_energy(&event, since_last_update_secs);
            }

            if let Some(mqtt_infos) = &self.mqtt_infos {
                for sensor_data in event.data() {
                    let topic = sensor_data_to_topic(&mqtt_infos.id, &sensor_data);

                    #[cfg(debug_assertions)]
                    match mqtt_infos.publisher.publish(&topic, sensor_data) {
                        Ok(_) => println!("✓ Sensor data published on topic {}", topic),
                        Err(e) => eprintln!("✗ Failed to publish sensor data on topic {}: {:?}", topic, e),
                    }

                    #[cfg(not(debug_assertions))]
                    let _ = mqtt_infos.publisher.publish(&topic, sensor_data);
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

/// Returns whether the current process has elevated/RAPL privileges.
fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        let admin = is_admin::is_admin();
        #[cfg(debug_assertions)]
        if !admin {
            eprintln!("\u{26a0} Running without Administrator privileges. CPU power readings will use estimation.");
        }
        admin
    }

    #[cfg(target_os = "linux")]
    {
        let rapl_accessible = std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/energy_uj").is_ok();
        #[cfg(debug_assertions)]
        if !rapl_accessible {
            eprintln!("\u{26a0} RAPL not accessible. CPU power readings will use estimation.");
            eprintln!("  Tip: run as root or grant read access to /sys/class/powercap/");
        }
        rapl_accessible
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        #[cfg(debug_assertions)]
        eprintln!("\u{26a0} No privileged power-reading support on this platform. Using estimation.");
        false
    }
}
