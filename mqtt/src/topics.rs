/// Builds the state topic for a node and sensor key.
pub fn sensor_type_to_topic(id: &str, sensor_key: &str) -> String {
    format!("{id}/sensor_data/{}", topic_segment(sensor_key))
}

/// Converts a value into a lowercase MQTT-safe topic segment.
pub fn topic_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Builds the retained hardware manifest topic for a node.
pub fn hardware_info_topic(id: &str) -> String {
    format!("{}/hardware_info", id)
}

/// Builds the retained availability topic for a node.
pub fn status_topic(id: &str) -> String {
    format!("{}/status", id)
}

/// Builds a Home Assistant MQTT Discovery configuration topic.
pub fn ha_discovery_topic(base_id: &str, object_id: &str) -> String {
    format!("homeassistant/sensor/{}/{}/config", base_id, object_id)
}

#[cfg(test)]
mod tests {
    use super::sensor_type_to_topic;

    #[test]
    fn identical_gpu_models_with_different_pci_addresses_have_unique_topics() {
        let first = sensor_type_to_topic("node", "AMD Radeon RX 7800 XT (0000:2f:00.0)");
        let second = sensor_type_to_topic("node", "AMD Radeon RX 7800 XT (0000:30:00.0)");

        assert_ne!(first, second);
        assert_eq!(first, "node/sensor_data/amd_radeon_rx_7800_xt__0000_2f_00_0",);
        assert_eq!(second, "node/sensor_data/amd_radeon_rx_7800_xt__0000_30_00_0",);
    }
}
