pub mod topics;

use std::{fmt, net::SocketAddr, time::Duration};

use mockall::automock;
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

#[automock]
pub trait MQTTClient {
    /// Publish `payload` to the self client `topic`
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
    /// Create a new MQTT publisher from a client
    pub fn new(client: T) -> Self {
        Self { client }
    }

    /// Publish `data` to the self client `topic`
    pub fn publish(&self, topic: &str, data: &impl Serialize) -> Result<(), MQTTError> {
        let payload = serde_json::to_vec(data).map_err(|_| MQTTError::SerializationError)?;
        self.client.publish(topic, payload)
    }
}

impl MQTTPublisher<Client> {
    /// Create a new MQTT publisher of rumqttc client from a broker address
    pub fn new_from_addr(addr: &SocketAddr) -> Self {
        let host = addr.ip().to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_publish() {
        let test_topic = "wattseal_collector/CPU";
        let mut mock = MockMQTTClient::new();

        mock.expect_publish()
            .withf(move |topic, _| topic == test_topic)
            .times(1)
            .returning(|_, _| Ok(()));

        let publisher = MQTTPublisher::new(mock);
        let data = serde_json::json!({"test_value": 6});

        let result = publisher.publish(test_topic, &data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_publish_not_serializable() {
        struct NotSerializable;
        impl serde::Serialize for NotSerializable {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("Forced serialization error"))
            }
        }

        let test_topic = "error_collector/not_serializable";
        let mut mock = MockMQTTClient::new();

        mock.expect_publish().times(0);

        let publisher = MQTTPublisher::new(mock);

        let result = publisher.publish(test_topic, &NotSerializable);

        assert!(matches!(result, Err(MQTTError::SerializationError)))
    }

    #[test]
    fn test_publish_send_error() {
        let test_topic = "error_collector/public_error";
        let mut mock = MockMQTTClient::new();

        mock.expect_publish()
            .withf(move |topic, _| topic == test_topic)
            .times(1)
            .returning(|_, _| Err(MQTTError::PublishError));

        let publisher = MQTTPublisher::new(mock);
        let data = serde_json::json!({"test_value": 6});

        let result = publisher.publish(test_topic, &data);

        assert!(matches!(result, Err(MQTTError::PublishError)));
    }
}
