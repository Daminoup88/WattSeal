pub mod sensors;

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Instant, SystemTime},
};

#[cfg(not(debug_assertions))]
use common::logging::start_log_session;
pub use common::types::ConsumptionUnit;
use common::{MetricSample, clog, types::HardwareInfo};
use sensors::{
    DiskSensor, NetworkSensor, RamSensor, SensorType, compute_sensors, gather_hardware_info,
    gpu::{GPUVendor, get_gpu_adapters},
};
use sysinfo::System;

use crate::sensors::measure_sensors;

/// Hardware sensor metrics collector.
/// Samples CPU, GPU, RAM, Disk, Network and process energy/power.
pub struct Collector {
    sensors: Vec<SensorType>,
    system: Rc<RefCell<System>>,
    hardware_info: HardwareInfo,
    last_update: Instant,
    process_limit: usize,
}

impl Collector {
    /// Detects hardware sensors, loads drivers, and gathers hardware specifications.
    pub fn new() -> Result<Self, String> {
        #[cfg(not(debug_assertions))]
        start_log_session();

        clog!("\n========== INITIALIZING SYSTEM ==========\n");

        let s = System::new_all()
            .or_else(|_| System::new())
            .map_err(|e| format!("Failed to initialize sysinfo System: {e}"))?;
        let system = Rc::new(RefCell::new(s));
        let mut sensors = Vec::new();

        // CPU sensor
        clog!("Initializing sensors...");
        match sensors::cpu::get_cpu_power_sensor(system.clone(), 0) {
            Ok(sensor) => {
                if let SensorType::CPU(cpu_sensor) = &sensor {
                    let (os_label, mode_label) = cpu_sensor.power_mode_labels();
                    clog!("CPU power mode: {os_label}/{mode_label}");
                }
                clog!("✓ CPU Power Sensor initialized successfully");
                sensors.push(sensor);
            }
            Err(e) => clog!("✗ Failed to initialize CPU Power Sensor: {:?}", e),
        }

        // GPU sensors
        let adapters = get_gpu_adapters();
        let adapter_names: Vec<String> = adapters.iter().map(|a| a.name.clone()).collect();
        clog!("\nDetected GPUs: {adapter_names:#?}");
        if adapters.is_empty() {
            clog!("⚠ No supported GPU adapters detected.");
        }

        let (mut nvidia_index, mut amd_index, mut intel_index) = (0u32, 0u32, 0u32);
        for adapter in &adapters {
            let display_index = match adapter.vendor {
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

            let luid_str = adapter
                .luid
                .map(|l| format!("0x{:016x}", l))
                .unwrap_or_else(|| "none".to_string());
            clog!(
                "Initializing GPU sensor: '{}' (Vendor: {:?}, PCI Vendor ID: 0x{:04x}, Device ID: 0x{:04x}, LUID: {}, Index: {})",
                adapter.name,
                adapter.vendor,
                adapter.vendor_id,
                adapter.device_id,
                luid_str,
                display_index
            );

            match sensors::gpu::get_gpu_energy_sensor(adapter, display_index) {
                Ok(sensor) => {
                    clog!("✓ GPU sensor initialized: '{} ({})'", adapter.name, display_index);
                    sensors.push(sensor);
                }
                Err(e) => {
                    clog!(
                        "✗ Failed to initialize GPU sensor for '{}' (vendor={:?}, index={}): {:?}",
                        adapter.name,
                        adapter.vendor,
                        display_index,
                        e
                    );
                }
            }
        }

        // RAM, Disk, Network sensors
        sensors.push(SensorType::RAM(RamSensor::new(system.clone())));
        match DiskSensor::new() {
            Ok(disk_sensor) => {
                clog!("✓ Disk sensor initialized successfully");
                sensors.push(SensorType::Disk(disk_sensor));
            }
            Err(e) => {
                clog!("✗ Failed to initialize Disk sensor: {:?}", e);
            }
        }
        match NetworkSensor::new() {
            Ok(network_sensor) => {
                clog!("✓ Network sensor initialized successfully");
                sensors.push(SensorType::Network(network_sensor));
            }
            Err(e) => {
                clog!("✗ Failed to initialize Network sensor: {:?}", e);
            }
        }

        // Hardware info
        clog!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        let hardware_info = gather_hardware_info(&sensors);
        clog!("Initialization complete");

        Ok(Collector {
            sensors,
            system,
            hardware_info,
            last_update: Instant::now(),
            process_limit: 10,
        })
    }

    /// Returns a reference to the detected static hardware specifications.
    pub fn hardware_info(&self) -> &HardwareInfo {
        &self.hardware_info
    }

    /// Sets the maximum number of processes included in collected samples.
    pub fn set_process_limit(&mut self, process_limit: usize) {
        self.process_limit = process_limit;
    }

    /// Performs one sampling tick across all hardware sensors and returns the computed sample.
    pub fn sample(&mut self) -> MetricSample {
        let now = Instant::now();
        let since_last_update = self.last_update.elapsed();
        self.last_update = now;

        let time = SystemTime::now();
        let raw_data = measure_sensors(
            &self.sensors,
            self.system.clone(),
            since_last_update,
            self.process_limit,
        );
        let computed_data = compute_sensors(&raw_data, self.process_limit);

        MetricSample::new(time, since_last_update, computed_data)
    }
}
