pub mod process;
mod tables;

pub use common::{
    DATABASE_PATH, Database, DatabaseEntry, Event,
    database::types::{
        CPUDataDB, DiskDataDB, GPUDataDB, NetworkDataDB, ProcessDataDB, RamDataDB, SensorDataDB, TotalDataDB,
    },
};
