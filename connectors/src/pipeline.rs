use common::{MetricSample, clog};

use crate::{db::DatabaseConnector, mqtt::MqttConnector};

/// Static output pipeline managing all configured destinations.
/// Dispatches metric samples with zero heap allocations, zero vtables,
/// and full compiler inlining across crate boundaries.
pub struct Connectors {
    pub database: Option<DatabaseConnector>,
    pub mqtt: Option<MqttConnector>,
}

impl Connectors {
    /// Creates a new `Connectors` pipeline with the given optional output destinations.
    pub fn new(database: Option<DatabaseConnector>, mqtt: Option<MqttConnector>) -> Self {
        Self { database, mqtt }
    }

    /// Dispatches a collected metric sample to all enabled outputs.
    pub fn handle_sample(&mut self, sample: &MetricSample) {
        if let Some(db) = &mut self.database {
            if let Err(e) = db.handle_sample(sample) {
                clog!("⚠ [Database] Sample handling error: {e}");
            }
        }
        if let Some(mqtt) = &mut self.mqtt {
            if let Err(e) = mqtt.handle_sample(sample) {
                clog!("⚠ [MQTT] Sample handling error: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use common::MetricSample;

    use super::*;

    #[test]
    fn test_empty_pipeline_handles_sample() {
        let mut connectors = Connectors::new(None, None);
        let sample = MetricSample::new(SystemTime::now(), Duration::from_secs(1), Vec::new());
        connectors.handle_sample(&sample);
        assert!(connectors.database.is_none());
        assert!(connectors.mqtt.is_none());
    }
}
