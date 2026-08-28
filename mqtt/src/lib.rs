pub mod config;
pub mod discovery;
pub mod topics;

use std::{
    net::SocketAddr,
    time::{Duration, SystemTime},
};

use common::types::HardwareInfo;
pub use config::{MqttConfig, ProcessPublishMode};
use mockall::automock;
use rumqttc::{Client, MqttOptions, QoS};
use serde::Serialize;
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
    pub timestamp_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_energy: Option<E>,
    #[serde(flatten)]
    pub data: &'a T,
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
}

impl<T: MQTTClient> MQTTPublisher<T> {
    /// Create a new MQTT publisher from a client implementation
    pub fn new(client: T) -> Self {
        Self {
            client,
            hardware_info: std::sync::Arc::new(std::sync::RwLock::new(None)),
            config: MqttConfig::new("wattseal_collector".to_string(), "127.0.0.1:1883".parse().unwrap()),
        }
    }

    /// Non-blocking publish of `data` to `topic`
    pub fn publish(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTError> {
        let payload = serde_json::to_vec(data)?;
        self.client.try_publish_bytes(topic, QoS::AtLeastOnce, false, payload)
    }

    /// Retained non-blocking publish of `data` to `topic`
    pub fn publish_retained(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTError> {
        let payload = serde_json::to_vec(data)?;
        self.client.try_publish_bytes(topic, QoS::AtLeastOnce, true, payload)
    }

    /// Publish `data` wrapped in a generic timestamped `Envelope`
    pub fn publish_envelope<TData: Serialize, E: Serialize>(
        &self,
        topic: &str,
        data: &TData,
        timestamp_ms: i64,
        total_energy: Option<E>,
    ) -> Result<(), MQTTError> {
        let envelope = Envelope {
            timestamp_ms,
            total_energy,
            data,
        };
        self.publish(topic, &envelope)
    }

    /// Update hardware info and publish Home Assistant discovery configs if enabled
    pub fn set_hardware_info(&self, info: &HardwareInfo) {
        if let Ok(mut lock) = self.hardware_info.write() {
            *lock = Some(info.clone());
        }
        if self.config.home_assistant {
            self.publish_ha_discovery(info);
        }
    }

    /// Publish Home Assistant discovery configs
    pub fn publish_ha_discovery(&self, hw_info: &HardwareInfo) {
        let device = discovery::HaDevice::new(
            &self.config.id,
            Some(hw_info.system.hostname.as_str()),
            Some(hw_info.system.os.as_str()),
            Some(hw_info.cpu.name.as_str()),
        );
        let ha_configs = discovery::build_ha_discovery_configs(&self.config.id, &device, &hw_info.gpus);
        for (top, cfg) in ha_configs {
            if let Ok(payload) = serde_json::to_vec(&cfg) {
                let _ = self.client.try_publish_bytes(&top, QoS::AtLeastOnce, true, payload);
            }
        }
    }

    /// Signal graceful offline status
    pub fn publish_offline(&self, node_id: &str) -> Result<(), MQTTError> {
        let topic = topics::status_topic(node_id);
        self.client
            .try_publish_bytes(&topic, QoS::AtLeastOnce, true, b"offline".to_vec())
    }
}

impl MQTTPublisher<Client> {
    /// Create a new MQTT publisher of rumqttc client from a broker address (default options)
    pub fn new_from_addr(addr: &SocketAddr) -> Self {
        let config = MqttConfig::new("wattseal_collector".to_string(), *addr);
        Self::new_from_config(&config, None)
    }

    /// Create a new MQTT publisher based on full `MqttConfig` configuration and optional hardware info
    pub fn new_from_config(config: &MqttConfig, hardware_info: Option<&HardwareInfo>) -> Self {
        let host = config.addr.ip().to_string();
        let port = config.addr.port();

        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let suffix = (nanos & 0xFFFF) as u16;
        let client_id = format!("{}_{:04x}", config.id, suffix);

        let mut options = MqttOptions::new(&client_id, host, port);
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
        let is_ha = config.home_assistant;
        let client_clone = client.clone();
        let status_top_clone = status_top.clone();
        let hw_info_arc = std::sync::Arc::new(std::sync::RwLock::new(hardware_info.cloned()));
        let hw_info_clone = hw_info_arc.clone();

        std::thread::spawn(move || {
            let mut is_connected = false;
            for event in connection.iter() {
                match event {
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                        if !is_connected {
                            is_connected = true;
                            // Publish retained birth message
                            let _ = client_clone.try_publish(&status_top_clone, QoS::AtLeastOnce, true, "online");

                            // Publish Home Assistant discovery configs if enabled
                            if is_ha {
                                let hw_info_guard = hw_info_clone.read().ok();
                                let hw_info = hw_info_guard.as_ref().and_then(|g| g.as_ref());
                                let device = discovery::HaDevice::new(
                                    &node_id,
                                    hw_info.map(|h| h.system.hostname.as_str()),
                                    hw_info.map(|h| h.system.os.as_str()),
                                    hw_info.map(|h| h.cpu.name.as_str()),
                                );
                                let gpus: Vec<String> = hw_info.map(|h| h.gpus.clone()).unwrap_or_default();
                                let ha_configs = discovery::build_ha_discovery_configs(&node_id, &device, &gpus);
                                for (top, cfg) in ha_configs {
                                    if let Ok(payload) = serde_json::to_vec(&cfg) {
                                        let _ = client_clone.try_publish(&top, QoS::AtLeastOnce, true, payload);
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
        }
    }
}

#[cfg(test)]
mod tests {
    use discovery::{HaDevice, build_ha_discovery_configs};

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
        let data = serde_json::json!({"usage_percent": 45.5});

        let result = publisher.publish_envelope(test_topic, &data, 1700000000000, Some(500000));

        assert!(result.is_ok());
    }

    #[test]
    fn test_ha_discovery_building() {
        let device = HaDevice::new("test_node", Some("my-host"), Some("Windows 11"), Some("Core i7"));
        let gpus = vec!["NVIDIA GeForce RTX 3070".to_string()];

        let configs = build_ha_discovery_configs("test_node", &device, &gpus);
        assert!(!configs.is_empty());

        let (cpu_topic, cpu_cfg) = configs.iter().find(|(t, _)| t.contains("cpu_usage")).unwrap();
        assert_eq!(cpu_topic, "homeassistant/sensor/test_node/cpu_usage/config");
        assert_eq!(cpu_cfg.state_topic, "test_node/sensor_data/cpu");
        assert_eq!(cpu_cfg.device.name, "my-host");

        let (gpu_energy_topic, gpu_energy_cfg) = configs.iter().find(|(t, _)| t.contains("gpu_nvidia_geforce_rtx_3070_energy")).unwrap();
        assert_eq!(gpu_energy_topic, "homeassistant/sensor/test_node/gpu_nvidia_geforce_rtx_3070_energy/config");
        assert_eq!(gpu_energy_cfg.state_topic, "test_node/sensor_data/gpu_nvidia_geforce_rtx_3070");
        assert_eq!(gpu_energy_cfg.unit_of_measurement.as_deref(), Some("Wh"));
        assert_eq!(gpu_energy_cfg.state_class.as_deref(), Some("total"));
    }
}
