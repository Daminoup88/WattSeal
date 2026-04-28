use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::ser::Serialize;

pub struct MQTTPublisher {
    client: AsyncClient,
}

#[derive(Debug)]
pub enum MQTTPublisherError {
    SerializationError,
    PublishError,
}

impl MQTTPublisher {
    pub async fn new(name: &str, host: &str, port: u16) -> Self {
        let mut options = MqttOptions::new(name, host, port);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(options, 10);

        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(_) => {}
                    Err(e) => eprintln!("Failed to poll event from mqtt client: {}", e),
                }
            }
        });

        Self { client }
    }

    pub async fn publish(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTPublisherError> {
        let payload = serde_json::to_vec(data).map_err(|_| MQTTPublisherError::SerializationError)?;
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await
            .map_err(|_| MQTTPublisherError::PublishError)
    }
}
