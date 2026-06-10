use std::cell::RefCell;

use common::types::EnergyUj;
use driver::ScaphandreMsrReader;

use super::{CPUVendor, Sensor, SensorError};
use crate::database::{CPUData, SensorData};

mod driver;

pub fn install() -> bool {
    match ScaphandreMsrReader::install() {
        Ok(()) => {
            crate::clog!("✓ CPU MSR driver installed successfully");
            true
        }
        Err(e) => {
            crate::clog!("✗ Failed to install CPU MSR driver: {e}");
            false
        }
    }
}

pub fn uninstall() -> bool {
    match ScaphandreMsrReader::uninstall() {
        Ok(()) => {
            crate::clog!("✓ CPU MSR driver uninstalled successfully");
            true
        }
        Err(e) => {
            crate::clog!("✗ Failed to uninstall CPU MSR driver: {e}");
            false
        }
    }
}

pub fn setup() {
    let installed = match ScaphandreMsrReader::is_installed() {
        Ok(installed) => installed,
        Err(e) => {
            crate::clog!("\u{26a0} {e}");
            false
        }
    };

    if installed && ScaphandreMsrReader::new().is_ok() {
        return;
    }

    if installed {
        crate::clog!("\u{26a0} CPU MSR driver is installed but not running. Admin approval is required to start it.");
    } else {
        crate::clog!("\u{26a0} CPU MSR driver not installed. Admin approval is required once to install it.");
    }

    if let Ok(exe) = std::env::current_exe() {
        match runas::Command::new(&exe).arg("--install-cpu-driver").gui(true).status() {
            Ok(status) if status.success() => {
                crate::clog!("✓ CPU MSR driver installation/start completed");
            }
            Ok(_) => {
                crate::clog!("\u{26a0} CPU MSR driver installation/start canceled or failed; using estimation");
            }
            Err(e) => {
                crate::clog!("\u{26a0} Failed to launch driver installer: {e}");
            }
        }
    } else {
        crate::clog!("\u{26a0} Unable to locate executable to install/start the CPU driver");
    }
}

#[derive(Clone)]
struct CPUValues {
    pkg: Option<u64>,
    pp0: Option<u64>,
    pp1: Option<u64>,
    dram: Option<u64>,
}

impl Default for CPUValues {
    fn default() -> Self {
        CPUValues {
            pkg: Some(0),
            pp0: Some(0),
            pp1: Some(0),
            dram: Some(0),
        }
    }
}

/// Windows CPU energy sensor using MSR (Model-Specific Registers) via Scaphandre.
pub struct WindowsCPUSensor {
    msr_reader: MSRReader,
    last_energy_measurement: RefCell<CPUValues>,
}

impl WindowsCPUSensor {
    /// Initializes the Scaphandre driver and MSR reader for the given CPU vendor.
    pub fn new(vendor_id: &str) -> Result<Self, SensorError> {
        let vendor = CPUVendor::from_str(vendor_id);
        let msr_driver = ScaphandreMsrReader::new()
            .map_err(|e| SensorError::ReadError(format!("Scaphandre driver init failed: {}", e)))?;
        let msr_reader = MSRReader::new(msr_driver, vendor);

        Ok(WindowsCPUSensor {
            msr_reader,
            last_energy_measurement: RefCell::new(CPUValues::default()),
        })
    }

    /// Reads raw energy counters and computes delta since last call.
    fn read_raw_energy_delta(&self) -> Result<CPUData, SensorError> {
        let current_energy = self.msr_reader.read_energy()?;
        let energy_values = {
            let last_energy = self
                .last_energy_measurement
                .try_borrow()
                .map_err(|e| SensorError::ReadError(format!("Failed to borrow last energy measurement: {}", e)))?;
            self.msr_reader.compute_delta_energy(&current_energy, &last_energy)
        };

        let mut last_energy_mut = self
            .last_energy_measurement
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to update last energy measurement: {}", e)))?;
        *last_energy_mut = current_energy;

        if energy_values.total_energy.is_none() {
            return Err(SensorError::ReadError("Failed to calculate energy".to_string()));
        }
        Ok(energy_values)
    }
}

impl Sensor for WindowsCPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let cpu_energy_values = self.read_raw_energy_delta()?;
        Ok(cpu_energy_values.into())
    }
}

struct MSRReader {
    msr_reader: ScaphandreMsrReader,
    vendor: CPUVendor,
    energy_unit: f64,
}

impl MSRReader {
    fn new(msr_reader: ScaphandreMsrReader, vendor: CPUVendor) -> Self {
        let energy_unit = Self::read_energy_unit(&msr_reader, &vendor).unwrap_or(0.0);
        MSRReader {
            msr_reader,
            vendor,
            energy_unit,
        }
    }

    fn read_energy_unit(msr_reader: &ScaphandreMsrReader, vendor: &CPUVendor) -> Result<f64, SensorError> {
        let read_fn = match vendor {
            CPUVendor::Intel => IntelMSR::read_energy_unit,
            CPUVendor::Amd => AMDMSR::read_energy_unit,
            CPUVendor::Other => return Err(SensorError::NotSupported),
        };
        read_fn(msr_reader).map_err(SensorError::ReadError)
    }

    fn read_energy(&self) -> Result<CPUValues, SensorError> {
        let read_fn = match self.vendor {
            CPUVendor::Intel => IntelMSR::read_energy_value,
            CPUVendor::Amd => AMDMSR::read_energy_value,
            CPUVendor::Other => return Err(SensorError::NotSupported),
        };
        let cpu_energy_values = read_fn(&self.msr_reader).map_err(SensorError::ReadError)?;
        Ok(cpu_energy_values)
    }

    fn compute_delta_energy(&self, current_energy: &CPUValues, last_energy: &CPUValues) -> CPUData {
        CPUData {
            total_energy: self.compute_component_delta_energy(current_energy.pkg, last_energy.pkg),
            pp0_energy: self.compute_component_delta_energy(current_energy.pp0, last_energy.pp0),
            pp1_energy: self.compute_component_delta_energy(current_energy.pp1, last_energy.pp1),
            dram_energy: self.compute_component_delta_energy(current_energy.dram, last_energy.dram),
            usage_percent: None,
        }
    }

    fn compute_component_delta_energy(
        &self,
        current_energy_value: Option<u64>,
        last_energy_value: Option<u64>,
    ) -> Option<EnergyUj> {
        match (current_energy_value, last_energy_value) {
            (Some(current), Some(last)) => {
                // Handle wrap-around of the energy counter and cast to u32 for 32-bit counters (still works for 64-bit counters with real-world deltas)
                let energy_diff = (current.wrapping_sub(last)) as u32;
                if current == 0 || last == 0 || energy_diff == 0 {
                    return None;
                }
                let energy_joules = energy_diff as f64 * self.energy_unit;
                Some(EnergyUj::from_joules(energy_joules))
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
        msr_reader: &ScaphandreMsrReader,
        msr_addr: u32,
        expression: fn(edx: u32, eax: u32) -> T,
    ) -> Result<T, String> {
        let out = msr_reader.read_msr(msr_addr)?;
        let edx = ((out >> 32) & 0xffffffff) as u32;
        let eax = (out & 0xffffffff) as u32;
        let result = expression(edx, eax);
        Ok(result)
    }
    fn read_energy_unit(msr_reader: &ScaphandreMsrReader) -> Result<f64, String>;
    fn read_energy_value(msr_reader: &ScaphandreMsrReader) -> Result<CPUValues, String>;
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
    fn read_energy_unit(msr_reader: &ScaphandreMsrReader) -> Result<f64, String> {
        Self::read_msr(
            msr_reader,
            Self::MSR_RAPL_POWER_UNIT as u32,
            Self::energy_unit_expression,
        )
    }
    fn read_energy_value(msr_reader: &ScaphandreMsrReader) -> Result<CPUValues, String> {
        let pkg_energy = Self::read_msr(msr_reader, Self::MSR_PKG_ENERGY_STATUS as u32, Self::energy_expression)?;
        let pp0_energy = Self::read_msr(msr_reader, Self::MSR_PP0_ENERGY_STATUS as u32, Self::energy_expression)?;
        let pp1_energy = Self::read_msr(msr_reader, Self::MSR_PP1_ENERGY_STATUS as u32, Self::energy_expression)?;
        let dram_energy = Self::read_msr(msr_reader, Self::MSR_DRAM_ENERGY_STATUS as u32, Self::energy_expression)?;

        Ok(CPUValues {
            pkg: Some(pkg_energy),
            pp0: Some(pp0_energy),
            pp1: Some(pp1_energy),
            dram: Some(dram_energy),
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

    fn read_energy_unit(msr_reader: &ScaphandreMsrReader) -> Result<f64, String> {
        Self::read_msr(
            msr_reader,
            Self::ENERGY_PWR_UNIT_MSR as u32,
            Self::energy_unit_expression,
        )
    }

    fn read_energy_value(msr_reader: &ScaphandreMsrReader) -> Result<CPUValues, String> {
        let pkg_energy = Self::read_msr(msr_reader, Self::ENERGY_PKG_MSR as u32, Self::energy_expression)?;
        let pp0_energy = Self::read_msr(msr_reader, Self::ENERGY_CORE_MSR as u32, Self::energy_expression)?;

        Ok(CPUValues {
            pkg: Some(pkg_energy),
            pp0: Some(pp0_energy),
            pp1: None,
            dram: None,
        })
    }
}
