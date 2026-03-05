pub mod database;
pub mod singleton;
pub mod types;
pub mod utils;

pub use database::{DATABASE_PATH, Database, DatabaseEntry, DatabaseError, UiSettings, generic_name_for_table};
pub use singleton::SingletonGuard;
pub use types::{
    AllTimeData, CPUData, DiskData, Event, GPUData, GeneralData, HardwareInfo, IconData, LabeledValue, MetricType,
    NetworkData, ProcessData, RamData, SecondaryValues, SensorData, TotalData,
};

/// Exit code the UI subprocess uses to signal "stop the collector too".
pub const EXIT_CODE_SHUTDOWN_ALL: i32 = 42;
