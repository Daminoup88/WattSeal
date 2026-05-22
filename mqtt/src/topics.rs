use common::types::SensorData;

pub fn sensor_data_to_topic<T: Clone>(id: &str, sensor_data: &SensorData<T>) -> String {
    let topic = "sensor_data";
    let type_topic = sensor_data.sensor_kind().to_string().to_lowercase();

    println!("{type_topic}");
    format!("{}/{}/{}", id, topic, type_topic)
}

pub fn hardware_info_topic(id: &str) -> String {
    let topic = "hardware_info";
    format!("{}/{}", id, topic)
}
