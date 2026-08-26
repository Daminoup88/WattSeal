use serde::Serialize;

use crate::topics::{ha_discovery_topic, sensor_type_to_topic, status_topic};

#[derive(Debug, Serialize, Clone)]
pub struct HaDevice {
    pub identifiers: Vec<String>,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub sw_version: String,
}

impl HaDevice {
    pub fn new(node_id: &str, hostname: Option<&str>, os_info: Option<&str>, cpu_model: Option<&str>) -> Self {
        // Use just the hostname as the device name — HA already prepends the
        // manufacturer field, so avoid any additional "WattSeal" prefix here.
        let name_str = match hostname {
            Some(h) if !h.is_empty() => h.to_string(),
            _ => node_id.to_string(),
        };
        let model_str = format!("{} / {}", os_info.unwrap_or("System"), cpu_model.unwrap_or("CPU"));

        Self {
            identifiers: vec![format!("wattseal_{}", node_id)],
            name: name_str,
            manufacturer: "WattSeal".to_string(),
            model: model_str,
            sw_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct HaSensorConfig {
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_not_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_of_measurement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub device: HaDevice,
}

#[derive(Debug, Clone)]
pub struct MetricDescriptor {
    pub sensor_type: &'static str,
    pub object_id: &'static str,
    pub name: &'static str,
    pub value_template: &'static str,
    pub unit: Option<&'static str>,
    pub device_class: Option<&'static str>,
    pub state_class: Option<&'static str>,
    pub icon: Option<&'static str>,
}

// Envelope JSON shape (after serde flatten):
//   { "timestamp_ms": 1234, "total_energy_uj": 9999, "CPU": { "usage_percent": 42.0, ... } }
//
// "total_energy_uj" at the root is the cumulative monotonically increasing counter
// maintained by the collector — ideal for HA total_increasing energy sensors.
// Per-component struct fields are accessed via the variant key (e.g. "CPU", "Ram", "Disk"…).
pub static METRIC_DESCRIPTORS: &[MetricDescriptor] = &[
    // ── CPU ──────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "cpu",
        object_id: "cpu_usage",
        name: "CPU Usage",
        value_template: "{{ value_json.CPU.usage_percent | round(1) }}",
        unit: Some("%"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:cpu-64-bit"),
    },
    MetricDescriptor {
        sensor_type: "cpu",
        object_id: "cpu_energy",
        name: "CPU Energy",
        // total_energy_uj is the cumulative counter in the Envelope root
        value_template: "{{ (value_json.total_energy_uj / 3600000000) | round(2) }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── GPU ──────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "gpu",
        object_id: "gpu_usage",
        name: "GPU Usage",
        value_template: "{{ value_json.GPU.usage_percent | round(1) }}",
        unit: Some("%"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:expansion-card"),
    },
    MetricDescriptor {
        sensor_type: "gpu",
        object_id: "gpu_vram_usage",
        name: "GPU VRAM Usage",
        value_template: "{{ value_json.GPU.vram_usage_percent | round(1) }}",
        unit: Some("%"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:expansion-card"),
    },
    MetricDescriptor {
        sensor_type: "gpu",
        object_id: "gpu_energy",
        name: "GPU Energy",
        value_template: "{{ value_json.total_energy_uj }}",
        unit: Some("µJ"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── RAM ──────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "ram",
        object_id: "ram_usage",
        name: "RAM Usage",
        value_template: "{{ value_json.Ram.usage_percent | round(1) }}",
        unit: Some("%"),
        device_class: None,
        state_class: Some("measurement"),
        icon: Some("mdi:memory"),
    },
    MetricDescriptor {
        sensor_type: "ram",
        object_id: "ram_energy",
        name: "RAM Energy",
        value_template: "{{ value_json.total_energy_uj }}",
        unit: Some("µJ"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Disk ─────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "disk",
        object_id: "disk_read",
        name: "Disk Read",
        value_template: "{{ value_json.Disk.read_bytes }}",
        unit: Some("B"),
        device_class: Some("data_size"),
        state_class: Some("measurement"),
        icon: Some("mdi:harddisk"),
    },
    MetricDescriptor {
        sensor_type: "disk",
        object_id: "disk_write",
        name: "Disk Write",
        value_template: "{{ value_json.Disk.written_bytes }}",
        unit: Some("B"),
        device_class: Some("data_size"),
        state_class: Some("measurement"),
        icon: Some("mdi:harddisk"),
    },
    MetricDescriptor {
        sensor_type: "disk",
        object_id: "disk_energy",
        name: "Disk Energy",
        value_template: "{{ value_json.total_energy_uj }}",
        unit: Some("µJ"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Network ──────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "network",
        object_id: "network_download",
        name: "Network Download",
        value_template: "{{ value_json.Network.downloaded_bytes }}",
        unit: Some("B"),
        device_class: Some("data_size"),
        state_class: Some("measurement"),
        icon: Some("mdi:download"),
    },
    MetricDescriptor {
        sensor_type: "network",
        object_id: "network_upload",
        name: "Network Upload",
        value_template: "{{ value_json.Network.uploaded_bytes }}",
        unit: Some("B"),
        device_class: Some("data_size"),
        state_class: Some("measurement"),
        icon: Some("mdi:upload"),
    },
    MetricDescriptor {
        sensor_type: "network",
        object_id: "network_energy",
        name: "Network Energy",
        value_template: "{{ value_json.total_energy_uj }}",
        unit: Some("µJ"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Total ─────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "total",
        object_id: "total_energy",
        name: "Total Energy",
        // TotalData.total_energy is the per-interval sum published on the total topic.
        // The envelope's total_energy_uj is the cumulative running total for this component.
        value_template: "{{ value_json.total_energy_uj }}",
        unit: Some("µJ"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
];

pub fn build_ha_discovery_configs(node_id: &str, device: &HaDevice) -> Vec<(String, HaSensorConfig)> {
    let avail_topic = status_topic(node_id);
    let mut configs = Vec::new();

    for desc in METRIC_DESCRIPTORS {
        let state_top = sensor_type_to_topic(node_id, desc.sensor_type);
        let discovery_top = ha_discovery_topic(node_id, desc.object_id);

        let config = HaSensorConfig {
            // Use just the metric label — HA automatically prefixes it with the
            // device name when building entity IDs, so avoid double-prefixing.
            name: desc.name.to_string(),
            unique_id: format!("wattseal_{}_{}", node_id, desc.object_id),
            state_topic: state_top,
            availability_topic: Some(avail_topic.clone()),
            payload_available: Some("online".to_string()),
            payload_not_available: Some("offline".to_string()),
            value_template: Some(desc.value_template.to_string()),
            unit_of_measurement: desc.unit.map(|u| u.to_string()),
            device_class: desc.device_class.map(|d| d.to_string()),
            state_class: desc.state_class.map(|s| s.to_string()),
            icon: desc.icon.map(|i| i.to_string()),
            device: device.clone(),
        };

        configs.push((discovery_top, config));
    }

    configs
}
