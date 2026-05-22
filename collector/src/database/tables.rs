use common::DatabaseEntry;

use super::{CPUDataDB, DiskDataDB, GPUDataDB, NetworkDataDB, RamDataDB};
use crate::sensors::SensorType;

impl SensorType {
    /// Returns the database table name for this sensor variant.
    pub fn table_name(&self) -> &'static str {
        match self {
            SensorType::CPU(_) => CPUDataDB::table_name_static(),
            SensorType::GPU(_) => GPUDataDB::table_name_static(),
            SensorType::RAM(_) => RamDataDB::table_name_static(),
            SensorType::Disk(_) => DiskDataDB::table_name_static(),
            SensorType::Network(_) => NetworkDataDB::table_name_static(),
        }
    }
}
