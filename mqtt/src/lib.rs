use std::time::Duration;

use rumqttc::{Client, MqttOptions, QoS};
use serde::ser::Serialize;

pub struct MQTTPublisher {
    client: Client,
}

#[derive(Debug)]
pub enum MQTTPublisherError {
    SerializationError,
    PublishError,
}

impl MQTTPublisher {
    pub fn new(name: &str, host: &str, port: u16) -> Self {
        let mut options = MqttOptions::new(name, host, port);
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

    pub fn publish(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTPublisherError> {
        let payload = serde_json::to_vec(data).map_err(|_| MQTTPublisherError::SerializationError)?;
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .map_err(|_| MQTTPublisherError::PublishError)
    }
}
