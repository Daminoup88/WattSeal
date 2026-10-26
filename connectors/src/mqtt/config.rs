use std::net::SocketAddr;

use common::types::ConsumptionUnit;

#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub id: String,
    pub instance_id: String,
    pub addr: SocketAddr,
    pub unit: Option<ConsumptionUnit>,
    pub home_assistant: bool,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub tls: bool,
}

impl MqttConfig {
    /// Creates a configuration with stable identity and conservative defaults.
    pub fn new(id: String, addr: SocketAddr) -> Self {
        let instance_id = machine_uid::get().unwrap_or(id.clone());
        Self {
            id: format!("{id}_{instance_id}"),
            instance_id,
            addr,
            unit: None,
            home_assistant: false,
            user: None,
            pass: None,
            tls: false,
        }
    }
}
