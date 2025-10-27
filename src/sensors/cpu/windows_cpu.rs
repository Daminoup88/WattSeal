use win_ring0::WinRing0;
use super::Sensor;

enum MeasurementSource {
    MSR(Box<WinRing0>),
}

struct WindowsCPUSensor {
    measurement_source: MeasurementSource,
    energy_unit: f64,
}

impl Sensor for WindowsCPUSensor {
    fn name(&self) -> &'static str {
        "Windows CPU"
    }

    fn read_power_watts(&self) -> Result<f64, super::SensorError> {
        match &self.measurement_source {
            MeasurementSource::MSR(ring0) => {
                let energy_value = match self.get_cpu_vendor()? {
                    CPUVendor::Intel => IntelMSR::read_energy_value(ring0)?,
                    CPUVendor::Amd => AMDMSR::read_energy_value(ring0)?,
                    CPUVendor::Other => return Err(super::SensorError::UnsupportedVendor),
                };
                let power_watts = (energy_value as f64) * self.energy_unit;
                Ok(power_watts)
            }
        }
    }

    fn read_usage_percent(&self) -> Result<f64, super::SensorError> {
        // Placeholder implementation
        Ok(0.0)
    }
}

fn get_measurement_source(ring0: Box<WinRing0>) -> Result<MeasurementSource, String> {
    Ok(MeasurementSource::MSR(ring0))
}

trait MSR {
    const ENERGY_UNIT_OFFSET: u8 = 8;
    const ENERGY_UNIT_MASK: u32 = 0x1F;
    fn energy_unit_expression(edx: u32, eax: u32) -> f64;
    fn energy_expression(edx: u32, eax: u32) -> u64 {
        ((edx as u64) << 32) | (eax as u64)
    }
    fn read_msr<T>(ring0: &WinRing0, msr_addr: u32, expression: fn(edx: u32, eax: u32) -> T) -> Result<T, String> {
        let out = ring0.readMsr(msr_addr)?;
        let edx = ((out >> 32) & 0xffffffff) as u32;
        let eax = (out & 0xffffffff) as u32;
        let result = expression(edx, eax);
        Ok(result)
    }
    fn read_energy_unit(ring0: &WinRing0) -> Result<f64, String>;
    fn read_energy_value(ring0: &WinRing0) -> Result<u64, String>;
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
        // power_unit = 1/2^PU where PU is bits 3:0 of EAX
        // let power_unit = 1.0 / f64::from(1 << (eax & 0xf));
        // energy_unit = 1/2^EU where EU is bits 12:8 of EAX
        let energy_unit = 1.0 / f64::from(1 << ((eax >> Self::ENERGY_UNIT_OFFSET) & Self::ENERGY_UNIT_MASK)) / 3600.0;
        // time_unit = 1/2^TU where TU is bits 19:16 of EAX
        // let time_unit = 1.0 / f64::from(1 << ((eax >> 16) & 0xf));
        energy_unit
    }
    fn read_energy_unit(ring0: &WinRing0) -> Result<f64, String> {
        Self::read_msr(ring0, Self::MSR_RAPL_POWER_UNIT as u32, Self::energy_unit_expression)
    }
    fn read_energy_value(ring0: &WinRing0) -> Result<u64, String> {
        Self::read_msr(ring0, Self::MSR_PKG_ENERGY_STATUS as u32, Self::energy_expression)
    }
}

#[allow(non_camel_case_types)]
enum AMDMSR {
    ENERGY_PWR_UNIT_MSR = 0xC0010299,
    ENERGY_CORE_MSR = 0xC001029A,
}

impl MSR for AMDMSR {
    fn energy_unit_expression(_edx: u32, eax: u32) -> f64 {
        // energy_unit = 1/2^EU where EU is bits 12:8 of EAX
        let energy_unit = 1.0 / f64::from(1 << ((eax >> Self::ENERGY_UNIT_OFFSET) & Self::ENERGY_UNIT_MASK));
        energy_unit
    }

    fn read_energy_unit(ring0: &WinRing0) -> Result<f64, String> {
        Self::read_msr(ring0, Self::ENERGY_PWR_UNIT_MSR as u32, Self::energy_unit_expression)
    }

    fn read_energy_value(ring0: &WinRing0) -> Result<u64, String> {
        Self::read_msr(ring0, Self::ENERGY_CORE_MSR as u32, Self::energy_expression)
    }
}