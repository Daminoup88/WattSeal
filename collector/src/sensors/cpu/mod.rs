use std::{cell::RefCell, collections::HashSet, rc::Rc};

use common::types::{CpuInfo, InitialInfo};
use sysinfo::System;
use windows_cpu::WindowsCPUSensor;

use super::{Sensor, SensorError, SensorType};
use crate::database::{CPUData, SensorData};

#[cfg(target_os = "windows")]
mod windows_cpu;

pub enum CPUOS {
    Windows(WindowsCPUSensor),
}

pub struct CPUSensor {
    sensor: CPUOS,
    system: Rc<RefCell<System>>,
}

impl Sensor for CPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let mut power_data = match &self.sensor {
            CPUOS::Windows(sensor) => sensor.read_full_data(),
        }?;
        if let SensorData::CPU(cpu_data) = power_data {
            let mut sys = self
                .system
                .try_borrow_mut()
                .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
            sys.refresh_cpu_usage();
            let usage_percent = sys.global_cpu_usage() as f64;
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

        Ok(InitialInfo::CPU(CpuInfo {
            name: cpu_name,
            vendor: cpu_vendor,
            base_frequency_mhz: cpu_frequency,
            logical_cores,
            physical_cores,
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

pub fn get_cpu_power_sensor(system: Rc<RefCell<System>>, index: usize) -> Result<SensorType, SensorError> {
    let s = system
        .try_borrow_mut()
        .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
    let cpu = s.cpus().get(index);
    let vendor_id = match cpu {
        None => return Err(SensorError::NotSupported),
        Some(cpu_info) => cpu_info.vendor_id(),
    };

    #[cfg(target_os = "windows")]
    return Ok(SensorType::CPU(CPUSensor {
        sensor: CPUOS::Windows(WindowsCPUSensor::new(vendor_id)),
        system: system.clone(),
    }));

    #[cfg(not(target_os = "windows"))]
    return Err(SensorError::NotSupported);
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
