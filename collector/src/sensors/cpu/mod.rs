use std::{cell::RefCell, collections::HashSet, rc::Rc};

use common::types::{CpuInfo, InitialInfo};
use sysinfo::System;

use super::{Sensor, SensorError, SensorType};
use crate::database::{CPUData, SensorData};

mod estimation;
#[cfg(target_os = "linux")]
mod linux_cpu;
#[cfg(target_os = "windows")]
mod windows_cpu;

use estimation::EstimationCPUSensor;
#[cfg(target_os = "linux")]
use linux_cpu::LinuxCPUSensor;
#[cfg(target_os = "windows")]
use windows_cpu::WindowsCPUSensor;

pub enum CPUOS {
    #[cfg(target_os = "windows")]
    Windows(WindowsCPUSensor),
    #[cfg(target_os = "linux")]
    Linux(LinuxCPUSensor),
    Estimation(EstimationCPUSensor),
}

pub struct CPUSensor {
    sensor: CPUOS,
    system: Rc<RefCell<System>>,
}

impl Sensor for CPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        // Get CPU usage first (needed for estimation, always populated)
        let usage_percent = {
            let mut sys = self
                .system
                .try_borrow_mut()
                .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
            sys.refresh_cpu_usage();
            sys.global_cpu_usage() as f64
        };

        let mut power_data = match &self.sensor {
            #[cfg(target_os = "windows")]
            CPUOS::Windows(sensor) => sensor.read_full_data()?,
            #[cfg(target_os = "linux")]
            CPUOS::Linux(sensor) => sensor.read_full_data()?,
            CPUOS::Estimation(sensor) => SensorData::CPU(CPUData {
                total_power_watts: Some(sensor.estimate(usage_percent)),
                pp0_power_watts: None,
                pp1_power_watts: None,
                dram_power_watts: None,
                usage_percent: None,
            }),
        };

        if let SensorData::CPU(cpu_data) = power_data {
            power_data = SensorData::CPU(CPUData {
                usage_percent: Some(usage_percent),
                ..cpu_data
            });
        }
        Ok(power_data)
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        let sys = self
            .system
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
        let logical_cores = sys.cpus().len() as u16;
        let physical_cores = System::physical_core_count().unwrap_or(0) as u16;
        let cpu_name = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());
        let cpu_vendor = sys
            .cpus()
            .first()
            .map(|cpu| cpu.vendor_id().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let cpu_frequency = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
        let architecture = System::cpu_arch();

        Ok(InitialInfo::CPU(CpuInfo {
            name: cpu_name,
            vendor: cpu_vendor,
            base_frequency_mhz: cpu_frequency,
            logical_cores,
            physical_cores,
            architecture,
        }))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        let sys = self
            .system
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
        let cpu_name = sys.cpus().first().map(|cpu| cpu.brand().to_string());
        Ok(format!(
            "Cpu: {}",
            cpu_name.unwrap_or_else(|| "Unknown CPU".to_string())
        ))
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum CPUVendor {
    Intel,
    Amd,
    Other,
}

impl CPUVendor {
    pub fn from_str(vendor_str: &str) -> CPUVendor {
        let vendor_lower = vendor_str.to_lowercase();
        if vendor_lower.contains("intel") {
            CPUVendor::Intel
        } else if vendor_lower.contains("amd") {
            CPUVendor::Amd
        } else {
            CPUVendor::Other
        }
    }
}

pub fn get_cpu_list(system: Rc<RefCell<System>>) -> Result<Vec<String>, String> {
    let s = system
        .try_borrow_mut()
        .map_err(|e| format!("Failed to borrow system: {}", e))?;
    Ok(s.cpus()
        .iter()
        .map(|cpu| cpu.brand().to_string())
        .collect::<HashSet<String>>()
        .into_iter()
        .collect())
}

pub fn get_cpu_power_sensor(
    system: Rc<RefCell<System>>,
    index: usize,
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] is_admin: bool,
) -> Result<SensorType, SensorError> {
    let s = system
        .try_borrow_mut()
        .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
    let cpu = s.cpus().get(index).ok_or(SensorError::NotSupported)?;
    let cpu_name = cpu.brand().to_string();
    #[cfg(target_os = "windows")]
    let vendor_id = cpu.vendor_id().to_string();
    drop(s);

    // Try platform-specific sensor first, fall back to TDP estimation
    #[cfg(target_os = "windows")]
    if is_admin {
        match WindowsCPUSensor::new(&vendor_id) {
            Ok(sensor) => {
                println!("✓ MSR sensor initialized successfully for vendor: {}", vendor_id);
                return Ok(SensorType::CPU(CPUSensor {
                    sensor: CPUOS::Windows(sensor),
                    system: system.clone(),
                }));
            }
            Err(e) => eprintln!("\u{26a0} MSR sensor unavailable ({:?}), falling back to estimation", e),
        }
    } else {
        eprintln!("\u{26a0} Not running as Administrator, skipping MSR sensor (WinRing0 requires admin)");
    }

    #[cfg(target_os = "linux")]
    match LinuxCPUSensor::new() {
        Ok(sensor) => {
            return Ok(SensorType::CPU(CPUSensor {
                sensor: CPUOS::Linux(sensor),
                system: system.clone(),
            }));
        }
        Err(e) => eprintln!("\u{26a0} RAPL sensor unavailable ({:?}), falling back to estimation", e),
    }
    println!("\u{26a0} No direct CPU power sensor available, using estimation based on TDP and usage");

    let tdp = estimation::lookup_tdp(&cpu_name);
    println!("\u{26a0} Using TDP estimation ({:.0} W) for {}", tdp, cpu_name);
    Ok(SensorType::CPU(CPUSensor {
        sensor: CPUOS::Estimation(EstimationCPUSensor::new(tdp)),
        system: system.clone(),
    }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cpu_vendor_from_str() {
        let intel = CPUVendor::from_str("GenuineIntel");
        assert!(matches!(intel, CPUVendor::Intel));

        let amd = CPUVendor::from_str("AuthenticAMD");
        assert!(matches!(amd, CPUVendor::Amd));

        let other = CPUVendor::from_str("SomeOtherVendor");
        assert!(matches!(other, CPUVendor::Other));
    }
}
