pub mod topics;

use std::{fmt, net::SocketAddr, time::Duration};

use rumqttc::{Client, MqttOptions, QoS};
use serde::ser::Serialize;

#[derive(Debug)]
pub enum MQTTError {
    SerializationError,
    PublishError,
}

impl fmt::Display for MQTTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MQTTError::SerializationError => write!(f, "Failed to serialize data to JSON"),
            MQTTError::PublishError => write!(f, "Failed to publish message to MQTT broker"),
        }
    }
}

pub trait MQTTClient {
    fn publish(&self, topic: &str, payload: Vec<u8>) -> Result<(), MQTTError>;
}

impl MQTTClient for Client {
    fn publish(&self, topic: &str, payload: Vec<u8>) -> Result<(), MQTTError> {
        self.publish(topic, QoS::AtLeastOnce, false, payload)
            .map_err(|_| MQTTError::PublishError)
    }
}

pub struct MQTTPublisher<T: MQTTClient> {
    client: T,
}

impl<T: MQTTClient> MQTTPublisher<T> {
    pub fn publish(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTError> {
        let payload = serde_json::to_vec(data).map_err(|_| MQTTError::SerializationError)?;
        self.client.publish(topic, payload)
    }
}

impl MQTTPublisher<Client> {
    pub fn new(addr: &SocketAddr) -> Self {
        let host = addr.ip().to_string().to_string();
        let port = addr.port();

        let mut options = MqttOptions::new("mqtt_broker", host, port);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, mut connection) = Client::new(options, 10);

        std::thread::spawn(move || {
            for event in connection.iter() {
                if let Err(e) = event {
                    eprintln!("MQTT Publisher connection error: {}", e);
                    std::thread::sleep(Duration::from_secs(5));
                }
            }
        });

        Self { client }
    }
}
