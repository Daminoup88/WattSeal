use common::types::SensorData;

pub fn sensor_data_to_topic(id: &str, sensor_data: &SensorData) -> String {
    let topic = "sensor_data";
    match sensor_data {
        SensorData::CPU(_) => format!("{}/{}/cpu", id, topic),
        SensorData::GPU(_) => format!("{}/{}/gpu", id, topic),
        SensorData::Ram(_) => format!("{}/{}/ram", id, topic),
        SensorData::Disk(_) => format!("{}/{}/disk", id, topic),
        SensorData::Network(_) => format!("{}/{}/network", id, topic),
        SensorData::Total(_) => format!("{}/{}/total", id, topic),
        SensorData::Process(_) => format!("{}/{}/process", id, topic),
    }
}

pub fn hardware_info_topic(id: &str) -> String {
    let topic = "hardware_info";
    format!("{}/{}", id, topic)
}
