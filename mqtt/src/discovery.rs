use common::types::HardwareInfo;
use serde::Serialize;

use crate::topics::{ha_discovery_topic, sensor_type_to_topic, status_topic, topic_segment};

#[derive(Debug, Serialize, Clone)]
pub struct HaDevice {
    pub identifiers: Vec<String>,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub sw_version: String,
}

impl HaDevice {
    pub fn new(instance_id: &str, hostname: Option<&str>) -> Self {
        // Use user-configured id or hostname directly
        let name_str = match hostname {
            Some(h) if !h.is_empty() => h.to_string(),
            _ => instance_id.to_string(),
        };
        Self {
            // Group under a unique device per PC / node_id
            identifiers: vec![format!("wattseal_{}", instance_id)],
            name: name_str.clone(),
            manufacturer: "WattSeal".to_string(),
            model: name_str,
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
    pub suggested_display_precision: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub device: HaDevice,
}

#[derive(Debug, Serialize)]
pub struct MqttManifest<'a> {
    #[serde(flatten)]
    pub hardware_info: &'a HardwareInfo,
    pub schema_version: u8,
    pub status_topic: String,
    pub sensor_topics: Vec<String>,
}

/// Builds retained metadata describing the node and all state topics.
pub fn build_mqtt_manifest<'a>(node_id: &str, info: &'a HardwareInfo) -> MqttManifest<'a> {
    let mut sensor_topics = BASE_METRIC_DESCRIPTORS
        .iter()
        .map(|descriptor| sensor_type_to_topic(node_id, descriptor.sensor_type))
        .collect::<Vec<_>>();
    sensor_topics.extend(info.gpus.iter().map(|name| sensor_type_to_topic(node_id, name)));

    MqttManifest {
        hardware_info: info,
        schema_version: 1,
        status_topic: status_topic(node_id),
        sensor_topics,
    }
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
//   { "timestamp_ms": 1234, "total_energy": 9.99, "CPU": { "usage_percent": 42.0, ... } }
//
// In Home Assistant mode:
// - All cumulative energy and byte values are accumulated in memory for the current process lifetime.
// - Home Assistant treats a restart-time decrease as a new total_increasing cycle.
// - Total energy is extracted directly from the envelope's "total_energy" field.
pub static BASE_METRIC_DESCRIPTORS: &[MetricDescriptor] = &[
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
        value_template: "{{ value_json.total_energy }}",
        unit: Some("Wh"),
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
        value_template: "{{ value_json.total_energy }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Disk ─────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "disk",
        object_id: "disk_read",
        name: "Disk Read",
        value_template: "{{ (value_json.Disk.read_bytes / 1000000) }}",
        unit: Some("MB"),
        device_class: Some("data_size"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:harddisk"),
    },
    MetricDescriptor {
        sensor_type: "disk",
        object_id: "disk_write",
        name: "Disk Write",
        value_template: "{{ (value_json.Disk.written_bytes / 1000000) }}",
        unit: Some("MB"),
        device_class: Some("data_size"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:harddisk"),
    },
    MetricDescriptor {
        sensor_type: "disk",
        object_id: "disk_energy",
        name: "Disk Energy",
        value_template: "{{ value_json.total_energy }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Network ──────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "network",
        object_id: "network_download",
        name: "Network Download",
        value_template: "{{ (value_json.Network.downloaded_bytes / 1000000) }}",
        unit: Some("MB"),
        device_class: Some("data_size"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:download"),
    },
    MetricDescriptor {
        sensor_type: "network",
        object_id: "network_upload",
        name: "Network Upload",
        value_template: "{{ (value_json.Network.uploaded_bytes / 1000000) }}",
        unit: Some("MB"),
        device_class: Some("data_size"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:upload"),
    },
    MetricDescriptor {
        sensor_type: "network",
        object_id: "network_energy",
        name: "Network Energy",
        value_template: "{{ value_json.total_energy }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Total ─────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "total",
        object_id: "total_energy",
        name: "Total Energy",
        value_template: "{{ value_json.total_energy }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total_increasing"),
        icon: Some("mdi:lightning-bolt"),
    },
];

struct DiscoveryMetric {
    object_id: String,
    name: String,
    state_topic: String,
    value_template: &'static str,
    unit: Option<&'static str>,
    device_class: Option<&'static str>,
    state_class: Option<&'static str>,
    suggested_display_precision: Option<u8>,
    icon: Option<&'static str>,
}

fn discovery_metrics(node_id: &str, info: &HardwareInfo) -> Vec<DiscoveryMetric> {
    let mut metrics = BASE_METRIC_DESCRIPTORS
        .iter()
        .map(|descriptor| DiscoveryMetric {
            object_id: descriptor.object_id.to_string(),
            name: descriptor.name.to_string(),
            state_topic: sensor_type_to_topic(node_id, descriptor.sensor_type),
            value_template: descriptor.value_template,
            unit: descriptor.unit,
            device_class: descriptor.device_class,
            state_class: descriptor.state_class,
            suggested_display_precision: Some(2),
            icon: descriptor.icon,
        })
        .collect::<Vec<_>>();

    for (index, gpu_name) in info.gpus.iter().enumerate() {
        let key = topic_segment(gpu_name);
        let state_topic = sensor_type_to_topic(node_id, gpu_name);
        for (suffix, label, template, unit, device_class, state_class, icon) in [
            (
                "usage",
                "Usage",
                "{{ value_json.GPU.usage_percent | round(1) }}",
                Some("%"),
                None,
                Some("measurement"),
                Some("mdi:expansion-card"),
            ),
            (
                "vram_usage",
                "VRAM Usage",
                "{{ value_json.GPU.vram_usage_percent | round(1) }}",
                Some("%"),
                None,
                Some("measurement"),
                Some("mdi:expansion-card"),
            ),
            (
                "energy",
                "Energy",
                "{{ value_json.total_energy }}",
                Some("Wh"),
                Some("energy"),
                Some("total_increasing"),
                Some("mdi:lightning-bolt"),
            ),
        ] {
            metrics.push(DiscoveryMetric {
                object_id: format!("{}_{}", key, suffix),
                name: format!("GPU {} {}", index + 1, label),
                state_topic: state_topic.clone(),
                value_template: template,
                unit,
                device_class,
                state_class,
                suggested_display_precision: Some(2),
                icon,
            });
        }
    }

    metrics
}

/// Builds retained Home Assistant Discovery configurations for all sensors.
pub fn build_ha_discovery_configs(
    node_id: &str,
    instance_id: &str,
    info: &HardwareInfo,
) -> Vec<(String, HaSensorConfig)> {
    let device = HaDevice::new(instance_id, Some(info.system.hostname.as_str()));
    let avail_topic = status_topic(node_id);
    discovery_metrics(node_id, info)
        .into_iter()
        .map(|metric| {
            let discovery_topic = ha_discovery_topic(node_id, &metric.object_id);
            let config = HaSensorConfig {
                name: metric.name,
                unique_id: format!("wattseal_{}_{}", instance_id, metric.object_id),
                state_topic: metric.state_topic,
                availability_topic: Some(avail_topic.clone()),
                payload_available: Some("online".to_string()),
                payload_not_available: Some("offline".to_string()),
                value_template: Some(metric.value_template.to_string()),
                unit_of_measurement: metric.unit.map(str::to_string),
                device_class: metric.device_class.map(str::to_string),
                state_class: metric.state_class.map(str::to_string),
                suggested_display_precision: metric.suggested_display_precision,
                icon: metric.icon.map(str::to_string),
                device: device.clone(),
            };
            (discovery_topic, config)
        })
        .collect()
}
