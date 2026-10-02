pub mod config;
pub mod discovery;
pub mod topics;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use common::types::HardwareInfo;
pub use config::{MqttConfig, ProcessPublishMode};
use mockall::automock;
use rumqttc::{Client, MqttOptions, QoS};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MQTTError {
    #[error("Failed to serialize data to JSON: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Failed to publish message to MQTT broker")]
    PublishError,
}

#[derive(Debug, Serialize)]
pub struct Envelope<'a, T: Serialize, E: Serialize = u64> {
    /// Timestamp of the measurement represented by the envelope.
    pub timestamp_ms: i64,
    /// Process-lifetime cumulative energy, when the source provides energy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_energy: Option<E>,
    #[serde(flatten)]
    pub data: &'a T,
}

fn accumulate_field(topic: &str, envelope: &mut Value, path: &str, totals: &mut HashMap<String, f64>) {
    let pointer = format!("/{}", path.replace('.', "/"));
    let Some(value) = envelope.pointer_mut(&pointer).and_then(|value| value.as_f64()) else {
        return;
    };
    let total = totals.entry(format!("{topic}:{path}")).or_default();
    *total += value.max(0.0);
    if let Some(target) = envelope.pointer_mut(&pointer) {
        *target = Value::from(*total);
    }
}

const DISK_CUMULATIVE_FIELDS: &[&str] = &["Disk.read_bytes", "Disk.written_bytes"];
const NETWORK_CUMULATIVE_FIELDS: &[&str] = &["Network.downloaded_bytes", "Network.uploaded_bytes"];

fn cumulative_fields_for_topic(topic: &str) -> &'static [&'static str] {
    match topic.rsplit('/').next() {
        Some("disk") => DISK_CUMULATIVE_FIELDS,
        Some("network") => NETWORK_CUMULATIVE_FIELDS,
        _ => &[],
    }
}

#[automock]
pub trait MQTTClient {
    /// Non-blocking publish of `payload` to the MQTT topic
    fn try_publish_bytes(&self, topic: &str, qos: QoS, retain: bool, payload: Vec<u8>) -> Result<(), MQTTError>;
}

impl MQTTClient for Client {
    fn try_publish_bytes(&self, topic: &str, qos: QoS, retain: bool, payload: Vec<u8>) -> Result<(), MQTTError> {
        self.try_publish(topic, qos, retain, payload)
            .map_err(|_| MQTTError::PublishError)
    }
}

pub struct MQTTPublisher<T: MQTTClient> {
    client: T,
    hardware_info: std::sync::Arc<std::sync::RwLock<Option<HardwareInfo>>>,
    config: MqttConfig,
    cumulative_values: Arc<Mutex<HashMap<String, f64>>>,
}

impl<T: MQTTClient> MQTTPublisher<T> {
    /// Creates a publisher around an MQTT client implementation.
    pub fn new(client: T) -> Self {
        Self {
            client,
            hardware_info: std::sync::Arc::new(std::sync::RwLock::new(None)),
            config: MqttConfig::new("wattseal_collector".to_string(), "127.0.0.1:1883".parse().unwrap()),
            cumulative_values: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Publishes a non-retained JSON message at QoS 1.
    pub fn publish(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTError> {
        let payload = serde_json::to_vec(data)?;
        self.client.try_publish_bytes(topic, QoS::AtLeastOnce, false, payload)
    }

    /// Publishes a retained JSON message at QoS 1.
    pub fn publish_retained(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTError> {
        let payload = serde_json::to_vec(data)?;
        self.client.try_publish_bytes(topic, QoS::AtLeastOnce, true, payload)
    }

    /// Publishes a non-retained QoS 0 envelope with process-lifetime totals.
    pub fn publish_envelope<TData: Serialize, E: Serialize>(
        &self,
        topic: &str,
        data: &TData,
        timestamp_ms: i64,
        total_energy: Option<E>,
    ) -> Result<(), MQTTError> {
        let mut envelope = serde_json::to_value(Envelope {
            timestamp_ms,
            total_energy,
            data,
        })?;

        let mut totals = self.cumulative_values.lock().map_err(|_| MQTTError::PublishError)?;
        if envelope.get("total_energy").is_some() {
            accumulate_field(topic, &mut envelope, "total_energy", &mut totals);
        }
        for path in cumulative_fields_for_topic(topic) {
            accumulate_field(topic, &mut envelope, path, &mut totals);
        }

        let payload = serde_json::to_vec(&envelope)?;
        self.client.try_publish_bytes(topic, QoS::AtMostOnce, false, payload)
    }

    /// Stores hardware metadata and publishes its retained MQTT representations.
    pub fn set_hardware_info(&self, info: &HardwareInfo) -> Result<(), MQTTError> {
        if let Ok(mut lock) = self.hardware_info.write() {
            *lock = Some(info.clone());
        }
        let manifest = discovery::build_mqtt_manifest(&self.config.id, info);
        let topic = topics::hardware_info_topic(&self.config.id);
        if self.config.home_assistant {
            self.publish_ha_discovery(info)?;
        }
        self.publish_retained(&topic, &manifest)?;
        Ok(())
    }

    /// Publishes retained Home Assistant MQTT Discovery configurations.
    pub fn publish_ha_discovery(&self, hw_info: &HardwareInfo) -> Result<(), MQTTError> {
        let ha_configs = discovery::build_ha_discovery_configs(&self.config.id, &self.config.instance_id, hw_info);
        for (top, cfg) in ha_configs {
            let payload = serde_json::to_vec(&cfg)?;
            self.client.try_publish_bytes(&top, QoS::AtLeastOnce, true, payload)?;
        }
        Ok(())
    }

    /// Publishes the retained offline availability state.
    pub fn publish_offline(&self, node_id: &str) -> Result<(), MQTTError> {
        let topic = topics::status_topic(node_id);
        self.client
            .try_publish_bytes(&topic, QoS::AtLeastOnce, true, b"offline".to_vec())
    }
}

impl MQTTPublisher<Client> {
    /// Creates a publisher using default MQTT settings for a broker address.
    pub fn new_from_addr(addr: &SocketAddr) -> Self {
        let config = MqttConfig::new("wattseal_collector".to_string(), *addr);
        Self::new_from_config(&config, None)
    }

    /// Creates a publisher and starts its reconnecting MQTT event loop.
    pub fn new_from_config(config: &MqttConfig, hardware_info: Option<&HardwareInfo>) -> Self {
        let host = config.addr.ip().to_string();
        let port = config.addr.port();

        let mut options = MqttOptions::new(&config.id, host, port);
        options.set_clean_session(false);
        options.set_keep_alive(Duration::from_secs(5));

        if let (Some(u), Some(p)) = (&config.user, &config.pass) {
            options.set_credentials(u, p);
        }

        if config.tls {
            options.set_transport(rumqttc::Transport::tls_with_default_config());
        }

        let status_top = topics::status_topic(&config.id);
        options.set_last_will(rumqttc::LastWill::new(&status_top, "offline", QoS::AtLeastOnce, true));

        let (client, mut connection) = Client::new(options, 10);

        let node_id = config.id.clone();
        let instance_id = config.instance_id.clone();
        let is_ha = config.home_assistant;
        let client_clone = client.clone();
        let status_top_clone = status_top.clone();
        let hw_info_arc = std::sync::Arc::new(std::sync::RwLock::new(hardware_info.cloned()));
        let hw_info_clone = hw_info_arc.clone();
        let cumulative_values = Arc::new(Mutex::new(HashMap::<String, f64>::new()));

        std::thread::spawn(move || {
            let mut is_connected = false;
            for event in connection.iter() {
                match event {
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                        if !is_connected {
                            is_connected = true;
                            // Publish retained birth message
                            let _ = client_clone.try_publish(&status_top_clone, QoS::AtLeastOnce, true, "online");

                            if let Ok(hw_info_guard) = hw_info_clone.read() {
                                if let Some(info) = hw_info_guard.as_ref() {
                                    let manifest = discovery::build_mqtt_manifest(&node_id, info);
                                    if let Ok(payload) = serde_json::to_vec(&manifest) {
                                        let manifest_topic = topics::hardware_info_topic(&node_id);
                                        let _ =
                                            client_clone.try_publish(&manifest_topic, QoS::AtLeastOnce, true, payload);
                                    }
                                }
                            }

                            // Publish Home Assistant discovery configs if enabled
                            if is_ha {
                                let hw_info_guard = hw_info_clone.read().ok();
                                let hw_info = hw_info_guard.as_ref().and_then(|g| g.as_ref());
                                if let Some(info) = hw_info {
                                    let ha_configs =
                                        discovery::build_ha_discovery_configs(&node_id, &instance_id, info);
                                    for (top, cfg) in ha_configs {
                                        if let Ok(payload) = serde_json::to_vec(&cfg) {
                                            let _ = client_clone.try_publish(&top, QoS::AtLeastOnce, true, payload);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        is_connected = false;
                        common::logging::log_component_error("mqtt", &format!("MQTT connection error: {}", e));
                        std::thread::sleep(Duration::from_secs(5));
                    }
                    _ => {}
                }
            }
        });

        Self {
            client,
            hardware_info: hw_info_arc,
            config: config.clone(),
            cumulative_values,
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{CPUData, ComputedSensorData, DiskData, EnergyUj, HardwareInfo, types::Byte};
    use discovery::build_ha_discovery_configs;

    use super::*;

    #[test]
    fn test_valid_publish() {
        let test_topic = "wattseal_collector/sensor_data/cpu";
        let mut mock = MockMQTTClient::new();

        mock.expect_try_publish_bytes()
            .withf(move |topic, qos, retain, _| topic == test_topic && *qos == QoS::AtLeastOnce && !*retain)
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let publisher = MQTTPublisher::new(mock);
        let data = serde_json::json!({"test_value": 6});

        let result = publisher.publish(test_topic, &data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_envelope_serialization() {
        let test_topic = "wattseal_collector/sensor_data/cpu";
        let mut mock = MockMQTTClient::new();

        mock.expect_try_publish_bytes()
            .withf(move |topic, _, _, payload| {
                let text = String::from_utf8_lossy(payload);
                topic == test_topic
                    && text.contains("\"timestamp_ms\":1700000000000")
                    && text.contains("\"total_energy\":500000")
                    && text.contains("\"usage_percent\":45.5")
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let publisher = MQTTPublisher::new(mock);
        let data = ComputedSensorData::CPU(CPUData {
            total_energy: Some(EnergyUj::from_u64(0)),
            pp0_energy: None,
            pp1_energy: None,
            dram_energy: None,
            usage_percent: Some(45.5),
        });

        let result = publisher.publish_envelope(test_topic, &data, 1700000000000, Some(500000));

        assert!(result.is_ok());
    }

    #[test]
    fn test_energy_envelopes_are_cumulative_and_not_retained() {
        let test_topic = "wattseal_collector/sensor_data/disk";
        let payloads = Arc::new(Mutex::new(Vec::new()));
        let captured_payloads = payloads.clone();
        let mut mock = MockMQTTClient::new();

        mock.expect_try_publish_bytes()
            .withf(move |topic, qos, retain, _| topic == test_topic && *qos == QoS::AtMostOnce && !*retain)
            .times(2)
            .returning(move |_, _, _, payload| {
                captured_payloads.lock().unwrap().push(payload);
                Ok(())
            });

        let publisher = MQTTPublisher::new(mock);
        let data = ComputedSensorData::Disk(DiskData {
            total_energy: Some(EnergyUj::from_u64(0)),
            read_bytes: Byte::from(10),
            written_bytes: Byte::from(20),
        });
        let next_data = ComputedSensorData::Disk(DiskData {
            total_energy: Some(EnergyUj::from_u64(0)),
            read_bytes: Byte::from(20),
            written_bytes: Byte::from(40),
        });

        publisher
            .publish_envelope(test_topic, &data, 1, Some(500000_u64))
            .unwrap();
        publisher
            .publish_envelope(test_topic, &next_data, 2, Some(250000_u64))
            .unwrap();

        let payloads = payloads.lock().unwrap();
        assert!(String::from_utf8_lossy(&payloads[0]).contains("\"total_energy\":500000"));
        assert!(String::from_utf8_lossy(&payloads[1]).contains("\"total_energy\":750000"));
        assert!(String::from_utf8_lossy(&payloads[0]).contains("\"read_bytes\":10"));
        assert!(String::from_utf8_lossy(&payloads[1]).contains("\"read_bytes\":30"));
        assert!(String::from_utf8_lossy(&payloads[1]).contains("\"written_bytes\":60"));
    }

    #[test]
    fn test_ha_discovery_building() {
        let mut info = HardwareInfo::default();
        info.system.hostname = "my-host".to_string();
        info.system.os = "Windows 11".to_string();
        info.cpu.name = "Core i7".to_string();
        info.gpus.push("NVIDIA GeForce RTX 3070 (0)".to_string());
        let configs = build_ha_discovery_configs("test_node", "test_machine", &info);
        assert!(!configs.is_empty());

        let (cpu_topic, cpu_cfg) = configs.iter().find(|(t, _)| t.contains("cpu_usage")).unwrap();
        assert_eq!(cpu_topic, "homeassistant/sensor/test_node/cpu_usage/config");
        assert_eq!(cpu_cfg.state_topic, "test_node/sensor_data/cpu");
        assert_eq!(cpu_cfg.device.name, "my-host");

        let (gpu_energy_topic, gpu_energy_cfg) = configs
            .iter()
            .find(|(t, _)| t.contains("nvidia_geforce_rtx_3070__0_energy"))
            .unwrap();
        assert_eq!(
            gpu_energy_topic,
            "homeassistant/sensor/test_node/nvidia_geforce_rtx_3070__0_energy/config"
        );
        assert_eq!(
            gpu_energy_cfg.state_topic,
            "test_node/sensor_data/nvidia_geforce_rtx_3070__0"
        );
        assert_eq!(gpu_energy_cfg.unit_of_measurement.as_deref(), Some("Wh"));
        assert_eq!(gpu_energy_cfg.state_class.as_deref(), Some("total_increasing"));
        assert_eq!(gpu_energy_cfg.suggested_display_precision, Some(2));
        assert_eq!(gpu_energy_cfg.name, "GPU 1 Energy");

        let runtime_gpu_topic = topics::sensor_type_to_topic("test_node", "NVIDIA GeForce RTX 3070 (0)");
        assert_eq!(runtime_gpu_topic, gpu_energy_cfg.state_topic);

        info.gpus.push("NVIDIA GeForce RTX 3070 (1)".to_string());
        let duplicate_configs = build_ha_discovery_configs("test_node", "test_machine", &info);
        let gpu_topics: Vec<_> = duplicate_configs
            .iter()
            .filter(|(topic, _)| topic.contains("nvidia_geforce_rtx_3070"))
            .map(|(topic, config)| (topic, &config.state_topic))
            .collect();
        assert_eq!(gpu_topics.len(), 6);
        assert_ne!(gpu_topics[0].0, gpu_topics[3].0);
        assert_ne!(gpu_topics[0].1, gpu_topics[3].1);
    }
}
