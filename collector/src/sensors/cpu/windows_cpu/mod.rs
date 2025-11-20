use super::CPUVendor;
use super::{Sensor, SensorError};
use crate::core::types::{CPUData, Event};
use driver::WinRing0Reader;
use std::cell::RefCell;
use std::time::Instant;

mod driver;

enum MeasurementSource {
    MSR(MSRReader),
    Estimation,
}

#[derive(Clone)]
struct EnergyMeasurement {
    energy_value: u64,
    instant: Instant,
}

impl Default for EnergyMeasurement {
    fn default() -> Self {
        EnergyMeasurement {
            energy_value: 0,
            instant: Instant::now(),
        }
    }
}

pub struct WindowsCPUSensor {
    measurement_source: MeasurementSource,
    last_energy_measurement: RefCell<EnergyMeasurement>,
}

impl Sensor<CPUData> for WindowsCPUSensor {
    fn new(vendor_id: &str) -> Self {
        let vendor = CPUVendor::from_str(vendor_id);
        let measurement_source = WinRing0Reader::new()
            .map(|ring0_reader| MeasurementSource::MSR(MSRReader::new(ring0_reader, vendor)))
            .unwrap_or(MeasurementSource::Estimation);

        let last_energy_measurement = EnergyMeasurement::default();

        WindowsCPUSensor {
            measurement_source,
            last_energy_measurement: RefCell::new(last_energy_measurement),
        }
    }

    fn name(&self) -> &'static str {
        "Windows CPU"
    }

    fn read_full_data(&self) -> Result<Event<CPUData>, SensorError> {
        let power = self.read_raw_power()?;
        let usage = self.read_cpu_usage()?;
        let data = CPUData {
            total_power_watts: power,

            pp0_power_watts: None,
            pp1_power_watts: None,
            dram_power_watts: None,
            usage_percent: usage,
        };
        Ok(Event::new(data))
    }
}

impl WindowsCPUSensor {
    fn read_raw_power(&self) -> Result<f64, SensorError> {
        match &self.measurement_source {
            MeasurementSource::MSR(msr_reader) => {
                let current_energy = msr_reader.read_energy()?;
                let power = msr_reader
                    .calculate_power(&current_energy, &self.last_energy_measurement.borrow());
                *self.last_energy_measurement.borrow_mut() = current_energy;

                if power.is_none() {
                    return Err(SensorError::ReadError("Failed to calculate power".to_string(),));
                }
                Ok(power.unwrap())
            }
            _ => Err(SensorError::NotSupported),
        }
    }

    fn read_cpu_usage(&self) -> Result<f64, SensorError> {
        // TODO: fetch CPU usage
        Ok(0.0)
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

    fn read_energy_unit(ring0_reader: &WinRing0Reader, vendor: &CPUVendor,) -> Result<f64, SensorError> {
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
        let energy_value = read_fn(&self.ring0_reader).map_err(SensorError::ReadError)?;
        Ok(EnergyMeasurement {
            energy_value,
            instant: Instant::now(),
        })
    }

    fn calculate_power(&self, current_energy: &EnergyMeasurement, last_energy: &EnergyMeasurement,) -> Option<f64> {
        let duration: f64 = current_energy
            .instant
            .duration_since(last_energy.instant)
            .as_secs_f64();
        if duration == 0.0 {
            return None; // Division by zero protection
        }
        let energy_diff = current_energy
            .energy_value
            .saturating_sub(last_energy.energy_value);

        if current_energy.energy_value == 0 || last_energy.energy_value == 0 || energy_diff == 0 {
            return None;
        }

        let power = (energy_diff as f64) * self.energy_unit / duration;
        Some(power)
    }
}

trait MSR {
    const ENERGY_UNIT_OFFSET: u8 = 8;
    const ENERGY_UNIT_MASK: u32 = 0x1F;
    fn energy_unit_expression(edx: u32, eax: u32) -> f64;
    fn energy_expression(edx: u32, eax: u32) -> u64 {
        ((edx as u64) << 32) | (eax as u64)
    }
    fn read_msr<T>(ring0_reader: &WinRing0Reader, msr_addr: u32, expression: fn(edx: u32, eax: u32) -> T,) -> Result<T, String> {
        let out = ring0_reader.read_msr(msr_addr)?;
        let edx = ((out >> 32) & 0xffffffff) as u32;
        let eax = (out & 0xffffffff) as u32;
        let result = expression(edx, eax);
        Ok(result)
    }
    fn read_energy_unit(ring0_reader: &WinRing0Reader) -> Result<f64, String>;
    fn read_energy_value(ring0_reader: &WinRing0Reader) -> Result<u64, String>;
}

#[allow(non_camel_case_types)]
enum IntelMSR {
    MSR_RAPL_POWER_UNIT = 0x606,
    MSR_PKG_ENERGY_STATUS = 0x611,
    MSR_PP0_ENERGY_STATUS = 0x639,
    MSR_PP1_ENERGY_STATUS = 0x641,
    MSR_DRAM_ENERGY_STATUS = 0x619,
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
    fn read_energy_value(ring0_reader: &WinRing0Reader) -> Result<u64, String> {
        Self::read_msr(
            ring0_reader,
            Self::MSR_PKG_ENERGY_STATUS as u32,
            Self::energy_expression,
        )
    }
}

#[allow(non_camel_case_types)]
enum AMDMSR {
    ENERGY_PWR_UNIT_MSR = 0xC0010299,
    ENERGY_CORE_MSR = 0xC001029A,
    ENERGY_PKG_MSR = 0xC001029B,
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

    fn read_energy_value(ring0_reader: &WinRing0Reader) -> Result<u64, String> {
        Self::read_msr(
            ring0_reader,
            Self::ENERGY_PKG_MSR as u32,
            Self::energy_expression,
        )
    }
}