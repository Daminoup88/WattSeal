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
        // Use user-configured id or hostname directly
        let name_str = match hostname {
            Some(h) if !h.is_empty() => h.to_string(),
            _ => node_id.to_string(),
        };
        let model_str = format!("{} / {}", os_info.unwrap_or("System"), cpu_model.unwrap_or("CPU"));

        Self {
            // Group under a unique device per PC / node_id
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
//   { "timestamp_ms": 1234, "total_energy": 9.99, "CPU": { "usage_percent": 42.0, ... } }
//
// In Home Assistant mode:
// - All cumulative energy values are published in Wh.
// - state_class is "total", allowing Home Assistant to calculate deltas and handle restarts properly.
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
        value_template: "{{ value_json.total_energy | round(3) }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total"),
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
        value_template: "{{ value_json.total_energy | round(3) }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total"),
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
        value_template: "{{ value_json.total_energy | round(3) }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total"),
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
        value_template: "{{ value_json.total_energy | round(3) }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total"),
        icon: Some("mdi:lightning-bolt"),
    },
    // ── Total ─────────────────────────────────────────────────────────────────
    MetricDescriptor {
        sensor_type: "total",
        object_id: "total_energy",
        name: "Total Energy",
        value_template: "{{ value_json.total_energy | round(3) }}",
        unit: Some("Wh"),
        device_class: Some("energy"),
        state_class: Some("total"),
        icon: Some("mdi:lightning-bolt"),
    },
];

pub fn build_ha_discovery_configs(
    node_id: &str,
    device: &HaDevice,
    detected_gpus: &[String],
) -> Vec<(String, HaSensorConfig)> {
    let avail_topic = status_topic(node_id);
    let mut configs = Vec::new();

    // 1. Base non-GPU descriptors
    for desc in BASE_METRIC_DESCRIPTORS {
        let state_top = sensor_type_to_topic(node_id, desc.sensor_type);
        let discovery_top = ha_discovery_topic(node_id, desc.object_id);

        let config = HaSensorConfig {
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

    // 2. Dynamic GPU descriptors
    if detected_gpus.is_empty() {
        // Fallback default GPU sensors if no hardware list detected yet
        add_gpu_sensors(node_id, device, "gpu", "GPU", &mut configs, &avail_topic);
    } else {
        for gpu_name in detected_gpus {
            let sanitized = gpu_name
                .chars()
                .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
                .collect::<String>();
            let clean = sanitized.trim_matches('_');
            let topic_key = if clean.is_empty() {
                "gpu".to_string()
            } else {
                format!("gpu_{}", clean)
            };
            let display_name = format!("GPU ({})", gpu_name);
            add_gpu_sensors(node_id, device, &topic_key, &display_name, &mut configs, &avail_topic);
        }
    }

    configs
}

fn add_gpu_sensors(
    node_id: &str,
    device: &HaDevice,
    topic_key: &str,
    display_prefix: &str,
    configs: &mut Vec<(String, HaSensorConfig)>,
    avail_topic: &str,
) {
    let state_top = sensor_type_to_topic(node_id, topic_key);

    let gpu_metrics = [
        (
            format!("{}_usage", topic_key),
            format!("{} Usage", display_prefix),
            "{{ value_json.GPU.usage_percent | round(1) }}",
            Some("%"),
            None,
            Some("measurement"),
            Some("mdi:expansion-card"),
        ),
        (
            format!("{}_vram_usage", topic_key),
            format!("{} VRAM Usage", display_prefix),
            "{{ value_json.GPU.vram_usage_percent | round(1) }}",
            Some("%"),
            None,
            Some("measurement"),
            Some("mdi:expansion-card"),
        ),
        (
            format!("{}_energy", topic_key),
            format!("{} Energy", display_prefix),
            "{{ value_json.total_energy | round(3) }}",
            Some("Wh"),
            Some("energy"),
            Some("total"),
            Some("mdi:lightning-bolt"),
        ),
    ];

    for (obj_id, name, template, unit, dev_class, state_class, icon) in gpu_metrics {
        let discovery_top = ha_discovery_topic(node_id, &obj_id);
        let config = HaSensorConfig {
            name,
            unique_id: format!("wattseal_{}_{}", node_id, obj_id),
            state_topic: state_top.clone(),
            availability_topic: Some(avail_topic.to_string()),
            payload_available: Some("online".to_string()),
            payload_not_available: Some("offline".to_string()),
            value_template: Some(template.to_string()),
            unit_of_measurement: unit.map(|u| u.to_string()),
            device_class: dev_class.map(|d| d.to_string()),
            state_class: state_class.map(|s| s.to_string()),
            icon: icon.map(|i| i.to_string()),
            device: device.clone(),
        };
        configs.push((discovery_top, config));
    }
}
