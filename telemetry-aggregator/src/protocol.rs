use crate::error::{AggregatorError, Result};

#[derive(Debug, Clone, Copy)]
pub struct SensorPoint {
    pub meter_id: u128,
    pub timestamp_us: u64,
    pub sensor_type: u8,
    pub sensor_value: f64,
}

impl SensorPoint {
    pub fn new(meter_id: u128, timestamp_us: u64, sensor_type: u8, sensor_value: f64) -> Self {
        Self { meter_id, timestamp_us, sensor_type, sensor_value }
    }
}

pub fn parse_coap_payload(data: &[u8]) -> Result<Vec<SensorPoint>> {
    if data.len() < 4 {
        return Err(AggregatorError::InvalidPacket("too short".into()));
    }

    let payload = if data[0] == 0xFF {
        &data[4..]
    } else {
        data
    };

    let text = std::str::from_utf8(payload)
        .map_err(|e| AggregatorError::InvalidPacket(format!("invalid UTF-8: {}", e)))?;

    let mut points = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(5, ',').collect();
        if parts.len() < 4 {
            continue;
        }

        let meter_id = u128::from_str_radix(parts[0].trim(), 16).unwrap_or(0);
        let timestamp = parts[1].trim().parse::<u64>().unwrap_or(0);
        let sensor_type = parts[2].trim().parse::<u8>().unwrap_or(0);
        let value = parts[3].trim().parse::<f64>().unwrap_or(0.0);

        points.push(SensorPoint::new(meter_id, timestamp, sensor_type, value));
    }

    Ok(points)
}
