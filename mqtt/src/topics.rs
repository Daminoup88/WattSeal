pub fn sensor_type_to_topic(id: &str, sensor_type: &str) -> String {
    format!("{}/sensor_data/{}", id, sensor_type.to_lowercase())
}

pub fn hardware_info_topic(id: &str) -> String {
    format!("{}/hardware_info", id)
}

pub fn status_topic(id: &str) -> String {
    format!("{}/status", id)
}

pub fn ha_discovery_topic(base_id: &str, object_id: &str) -> String {
    format!("homeassistant/sensor/{}/{}/config", base_id, object_id)
}
