use common::DatabaseTable;

use crate::sensors::{CPUSensor, GPUSensor, SensorType};

impl DatabaseTable for SensorType {
    fn table_name(&self) -> &'static str {
        match self {
            SensorType::CPU(s) => s.table_name(),
            SensorType::GPU(s) => s.table_name(),
        }
    }

    fn columns(&self) -> &'static [&'static str] {
        match self {
            SensorType::CPU(s) => s.columns(),
            SensorType::GPU(s) => s.columns(),
        }
    }
}

impl DatabaseTable for CPUSensor {
    fn table_name(&self) -> &'static str {
        "cpu_data"
    }

    fn columns(&self) -> &'static [&'static str] {
        &[
            "id                    INTEGER PRIMARY KEY",
            "timestamp_id          INTEGER REFERENCES timestamp(id)",
            "total_power_watts     REAL",
            "pp0_power_watts       REAL",
            "pp1_power_watts       REAL",
            "dram_power_watts      REAL",
            "usage_percent         REAL NOT NULL",
        ]
    }
}

impl DatabaseTable for GPUSensor {
    fn table_name(&self) -> &'static str {
        "gpu_data"
    }

    fn columns(&self) -> &'static [&'static str] {
        &[
            "id                    INTEGER PRIMARY KEY",
            "timestamp_id          INTEGER REFERENCES timestamp(id)",
            "total_power_watts     REAL",
            "usage_percent         REAL",
            "vram_usage_percent    REAL",
        ]
    }
}

// impl DatabaseTable for ScreenSensor {
//     fn table_name(&self) -> &'static str {
//         "screen_data"
//     }

//     fn columns(&self) -> &'static [&'static str] {
//         &[
//             "id                    INTEGER PRIMARY KEY",
//             "timestamp_id          INTEGER REFERENCES timestamp(id)",
//             "resolution_width      INTEGER NOT NULL",
//             "resolution_height     INTEGER NOT NULL",
//             "refresh_rate_hz       INTEGER NOT NULL",
//             "technology            TEXT NOT NULL",
//             "luminosity_nits       INTEGER NOT NULL",
//         ]
//     }
// }

// impl DatabaseTable for BatterySensor {
//     fn table_name(&self) -> &'static str {
//         "battery_data"
//     }

//     fn columns(&self) -> &'static [&'static str] {
//         &[
//             "id                    INTEGER PRIMARY KEY",
//             "timestamp_id          INTEGER REFERENCES timestamp(id)",
//             "manufacturer          TEXT NOT NULL",
//             "model                 TEXT NOT NULL",
//             "serial_number         TEXT NOT NULL",
//             "design_capacity_mwh   INTEGER NOT NULL",
//             "full_charge_capacity_mwh INTEGER NOT NULL",
//             "cycle_count           INTEGER NOT NULL",
//         ]
//     }
// }

// impl DatabaseTable for PeripheralsSensor {
//     fn table_name(&self) -> &'static str {
//         "peripherals_data"
//     }

//     fn columns(&self) -> &'static [&'static str] {
//         &[
//             "id                    INTEGER PRIMARY KEY",
//             "timestamp_id          INTEGER REFERENCES timestamp(id)",
//             "device_name           TEXT NOT NULL",
//             "device_type           TEXT NOT NULL",
//             "manufacturer          TEXT NOT NULL",
//             "is_connected          INTEGER NOT NULL",
//         ]
//     }
// }
