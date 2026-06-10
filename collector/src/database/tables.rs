use common::{DatabaseEntry, ProcessData};

use super::{CPUData, DiskData, GPUData, NetworkData, RamData, TotalData};
use crate::sensors::SensorType;

impl SensorType {
    /// Returns the database table name for this sensor variant.
    pub fn table_name(&self) -> &'static str {
        match self {
            SensorType::CPU(_) => CPUData::table_name_static(),
            SensorType::GPU(_) => GPUData::table_name_static(),
            SensorType::RAM(_) => RamData::table_name_static(),
            SensorType::Disk(_) => DiskData::table_name_static(),
            SensorType::Network(_) => NetworkData::table_name_static(),
            SensorType::Total => TotalData::table_name_static(),
            SensorType::Process => ProcessData::table_name_static(),
        }
    }
}
