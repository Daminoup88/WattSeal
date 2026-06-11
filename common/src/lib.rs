pub mod database;
pub mod logging;
pub mod singleton;
pub mod types;
pub mod utils;

/// In debug → `println!`. In release → append timestamped line to log file.
#[macro_export]
macro_rules! clog {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        println!($($arg)*);
        #[cfg(not(debug_assertions))]
        $crate::logging::log_to_file(&format!($($arg)*));
    }};
}

pub use database::{DATABASE_PATH, Database, DatabaseEntry, DatabaseError, UiSettings, generic_name_for_table};
pub use singleton::SingletonGuard;
pub use types::{
    AllTimeData, CPUData, DiskData, EnergyUj, Event, GPUData, GeneralData, HardwareInfo, IconData, LabeledValue,
    MICROJOULES_PER_JOULE, MetricKind, NetworkData, ProcessData, RamData, SECONDS_PER_HOUR, SecondaryValues,
    SensorData, TotalData,
};
pub use utils::set_current_dir_to_exe_dir;

/// Exit code the UI subprocess uses to signal "stop the collector too".
pub const EXIT_CODE_SHUTDOWN_ALL: i32 = 42;

pub const WINDOW_ICON_BYTES: &[u8] = include_bytes!("../../resources/icon.png");
pub const WINDOW_ICON_TYPE: image::ImageFormat = image::ImageFormat::Png;
