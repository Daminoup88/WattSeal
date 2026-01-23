use std::{cell::RefCell, time::Instant};

use driver::WinRing0Reader;
use sysinfo::System;

use super::{CPUVendor, Sensor, SensorError};
use crate::database::{CPUData, SensorData};

mod driver;

enum MeasurementSource {
    MSR(MSRReader),
    Estimation,
}

#[derive(Clone)]
struct CPUValues {
    pkg: Option<f64>,
    pp0: Option<f64>,
    pp1: Option<f64>,
    dram: Option<f64>,
}

impl Default for CPUValues {
    fn default() -> Self {
        CPUValues {
            pkg: None,
            pp0: None,
            pp1: None,
            dram: None,
        }
    }
}

#[derive(Clone)]
struct EnergyMeasurement {
    cpu_energy_values: CPUValues,
    instant: Instant,
}

impl Default for EnergyMeasurement {
    fn default() -> Self {
        EnergyMeasurement {
            cpu_energy_values: CPUValues {
                pkg: Some(0.0),
                pp0: Some(0.0),
                pp1: Some(0.0),
                dram: Some(0.0),
            },
            instant: Instant::now(),
        }
    }
}

pub struct WindowsCPUSensor {
    measurement_source: MeasurementSource,
    last_energy_measurement: RefCell<EnergyMeasurement>,
    sys: RefCell<System>,
}

impl WindowsCPUSensor {
    pub fn new(vendor_id: &str) -> Self {
        let vendor = CPUVendor::from_str(vendor_id);
        let measurement_source = WinRing0Reader::new()
            .map(|ring0_reader| MeasurementSource::MSR(MSRReader::new(ring0_reader, vendor)))
            .unwrap_or(MeasurementSource::Estimation);

        let last_energy_measurement = EnergyMeasurement::default();
        let mut sys = System::new_all();
        sys.refresh_all();

        WindowsCPUSensor {
            measurement_source,
            last_energy_measurement: RefCell::new(last_energy_measurement),
            sys: RefCell::new(sys),
        }
    }

    fn read_raw_power(&self) -> Result<CPUValues, SensorError> {
        match &self.measurement_source {
            MeasurementSource::MSR(msr_reader) => {
                let current_energy = msr_reader.read_energy()?;
                let power_values = msr_reader.calculate_power(&current_energy, &self.last_energy_measurement.borrow());
                *self.last_energy_measurement.borrow_mut() = current_energy;

                if power_values.pkg.is_none() {
                    return Err(SensorError::ReadError("Failed to calculate power".to_string()));
                }
                Ok(power_values)
            }
            _ => Err(SensorError::NotSupported),
        }
    }

    fn read_cpu_usage(&self) -> Result<f64, SensorError> {
        let mut sys = self.sys.borrow_mut();
        sys.refresh_all();

        // Calculate total usage across all CPUs and divide by number of CPUs
        let total_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
        Ok(total_usage as f64)
    }
}

impl Sensor for WindowsCPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let cpu_power_values = self.read_raw_power()?;
        let usage = self.read_cpu_usage()?;

        let data = CPUData {
            total_power_watts: cpu_power_values.pkg,
            pp0_power_watts: cpu_power_values.pp0,
            pp1_power_watts: cpu_power_values.pp1,
            dram_power_watts: cpu_power_values.dram,
            usage_percent: usage,
        };
        Ok(data.into())
    }
}

struct MSRReader {
    ring0_reader: WinRing0Reader,
    vendor: CPUVendor,
    energy_unit: f64,
}

impl MSRReader {
    fn new(ring0_reader: WinRing0Reader, vendor: CPUVendor) -> Self {
        let energy_unit = Self::read_energy_unit(&ring0_reader, &vendor).unwrap_or(0.0);
        MSRReader {
            ring0_reader,
            vendor,
            energy_unit,
        }
    }

    fn read_energy_unit(ring0_reader: &WinRing0Reader, vendor: &CPUVendor) -> Result<f64, SensorError> {
        let read_fn = match vendor {
            CPUVendor::Intel => IntelMSR::read_energy_unit,
            CPUVendor::Amd => AMDMSR::read_energy_unit,
            CPUVendor::Other => return Err(SensorError::NotSupported),
        };
        read_fn(ring0_reader).map_err(SensorError::ReadError)
    }

    fn read_energy(&self) -> Result<EnergyMeasurement, SensorError> {
        let read_fn = match self.vendor {
            CPUVendor::Intel => IntelMSR::read_energy_value,
            CPUVendor::Amd => AMDMSR::read_energy_value,
            CPUVendor::Other => return Err(SensorError::NotSupported),
        };
        let cpu_energy_values = read_fn(&self.ring0_reader).map_err(SensorError::ReadError)?;
        Ok(EnergyMeasurement {
            cpu_energy_values,
            instant: Instant::now(),
        })
    }

    fn calculate_power(&self, current_energy: &EnergyMeasurement, last_energy: &EnergyMeasurement) -> CPUValues {
        let duration: f64 = current_energy.instant.duration_since(last_energy.instant).as_secs_f64();
        if duration == 0.0 {
            return CPUValues::default(); // Division by zero protection
        }

        let pp1_value = self.calculate_component_power(
            current_energy.cpu_energy_values.pp1,
            last_energy.cpu_energy_values.pp1,
            duration,
        );

        let mut pkg_value = self.calculate_component_power(
            current_energy.cpu_energy_values.pkg,
            last_energy.cpu_energy_values.pkg,
            duration,
        );

        // GPU is integrated in CPU
        if !pp1_value.is_none() && !pkg_value.is_none() {
            pkg_value = pkg_value.map(|p| p - pp1_value.unwrap());
        }

        CPUValues {
            pkg: pkg_value,
            pp0: self.calculate_component_power(
                current_energy.cpu_energy_values.pp0,
                last_energy.cpu_energy_values.pp0,
                duration,
            ),
            pp1: pp1_value,
            dram: self.calculate_component_power(
                current_energy.cpu_energy_values.dram,
                last_energy.cpu_energy_values.dram,
                duration,
            ),
        }
    }

    fn calculate_component_power(
        &self,
        current_energy_value: Option<f64>,
        last_energy_value: Option<f64>,
        duration: f64,
    ) -> Option<f64> {
        match (current_energy_value, last_energy_value) {
            (Some(current), Some(last)) => {
                let energy_diff = (current as u64).saturating_sub(last as u64);
                if current == 0.0 || last == 0.0 || energy_diff == 0 {
                    return None;
                }
                let power = (energy_diff as f64) * self.energy_unit / duration;
                Some(power)
            }
            _ => None,
        }
    }
}

trait MSR {
    const ENERGY_UNIT_OFFSET: u8 = 8;
    const ENERGY_UNIT_MASK: u32 = 0x1F;
    fn energy_unit_expression(edx: u32, eax: u32) -> f64;
    fn energy_expression(edx: u32, eax: u32) -> u64 {
        ((edx as u64) << 32) | (eax as u64)
    }
    fn read_msr<T>(
        ring0_reader: &WinRing0Reader,
        msr_addr: u32,
        expression: fn(edx: u32, eax: u32) -> T,
    ) -> Result<T, String> {
        let out = ring0_reader.read_msr(msr_addr)?;
        let edx = ((out >> 32) & 0xffffffff) as u32;
        let eax = (out & 0xffffffff) as u32;
        let result = expression(edx, eax);
        Ok(result)
    }
    fn read_energy_unit(ring0_reader: &WinRing0Reader) -> Result<f64, String>;
    fn read_energy_value(ring0_reader: &WinRing0Reader) -> Result<CPUValues, String>;
}

#[allow(non_camel_case_types)]
enum IntelMSR {
    MSR_RAPL_POWER_UNIT = 0x606,
    MSR_PKG_ENERGY_STATUS = 0x611,
    MSR_PP0_ENERGY_STATUS = 0x639,
    MSR_PP1_ENERGY_STATUS = 0x641,
    MSR_DRAM_ENERGY_STATUS = 0x618,
}

impl MSR for IntelMSR {
    fn energy_unit_expression(_edx: u32, eax: u32) -> f64 {
        let energy_unit_raw = (eax >> Self::ENERGY_UNIT_OFFSET) & Self::ENERGY_UNIT_MASK;
        1.0 / (1u64 << energy_unit_raw) as f64
    }
    fn read_energy_unit(ring0_reader: &WinRing0Reader) -> Result<f64, String> {
        Self::read_msr(
            ring0_reader,
            Self::MSR_RAPL_POWER_UNIT as u32,
            Self::energy_unit_expression,
        )
    }
    fn read_energy_value(ring0_reader: &WinRing0Reader) -> Result<CPUValues, String> {
        let pkg_energy = Self::read_msr(
            ring0_reader,
            Self::MSR_PKG_ENERGY_STATUS as u32,
            Self::energy_expression,
        )?;
        let pp0_energy = Self::read_msr(
            ring0_reader,
            Self::MSR_PP0_ENERGY_STATUS as u32,
            Self::energy_expression,
        )?;
        let pp1_energy = Self::read_msr(
            ring0_reader,
            Self::MSR_PP1_ENERGY_STATUS as u32,
            Self::energy_expression,
        )?;
        let dram_energy = Self::read_msr(
            ring0_reader,
            Self::MSR_DRAM_ENERGY_STATUS as u32,
            Self::energy_expression,
        )?;

        Ok(CPUValues {
            pkg: Some(pkg_energy as f64),
            pp0: Some(pp0_energy as f64),
            pp1: Some(pp1_energy as f64),
            dram: Some(dram_energy as f64),
        })
    }
}

#[allow(non_camel_case_types)]
enum AMDMSR {
    ENERGY_PWR_UNIT_MSR = 0xC0010299,
    ENERGY_PKG_MSR = 0xC001029B,
    ENERGY_CORE_MSR = 0xC001029A,
}

impl MSR for AMDMSR {
    fn energy_unit_expression(_edx: u32, eax: u32) -> f64 {
        let energy_unit_raw = (eax >> Self::ENERGY_UNIT_OFFSET) & Self::ENERGY_UNIT_MASK;
        1.0 / (1u64 << energy_unit_raw) as f64
    }

    fn read_energy_unit(ring0_reader: &WinRing0Reader) -> Result<f64, String> {
        Self::read_msr(
            ring0_reader,
            Self::ENERGY_PWR_UNIT_MSR as u32,
            Self::energy_unit_expression,
        )
    }

    fn read_energy_value(ring0_reader: &WinRing0Reader) -> Result<CPUValues, String> {
        let pkg_energy: u64 = Self::read_msr(ring0_reader, Self::ENERGY_PKG_MSR as u32, Self::energy_expression)?;
        let pp0_energy = Self::read_msr(ring0_reader, Self::ENERGY_CORE_MSR as u32, Self::energy_expression)?;

        Ok(CPUValues {
            pkg: Some(pkg_energy as f64),
            pp0: Some(pp0_energy as f64),
            pp1: None,
            dram: None,
        })
    }
}
