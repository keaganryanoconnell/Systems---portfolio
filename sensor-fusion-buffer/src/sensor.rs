#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    LiDAR = 0,
    Camera = 1,
    IMU = 2,
    GPS = 3,
    Radar = 4,
}

impl SensorType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::LiDAR),
            1 => Some(Self::Camera),
            2 => Some(Self::IMU),
            3 => Some(Self::GPS),
            4 => Some(Self::Radar),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SensorData {
    LiDAR { points: u32, range_m: f32 },
    Camera { width: u16, height: u16, exposure_us: u32 },
    IMU { ax: f32, ay: f32, az: f32, gx: f32, gy: f32, gz: f32 },
    GPS { lat: f64, lon: f64, alt: f32 },
    Radar { velocity: f32, distance: f32, angle: f32 },
}

#[derive(Debug, Clone, Copy)]
pub struct SensorFrame {
    pub sensor_type: SensorType,
    pub sensor_id: u32,
    pub timestamp_ns: u64,
    pub sequence: u64,
    pub data: SensorData,
}

impl SensorFrame {
    #[allow(clippy::too_many_arguments)]
    pub fn new_imu(sensor_id: u32, timestamp_ns: u64, sequence: u64, ax: f32, ay: f32, az: f32, gx: f32, gy: f32, gz: f32) -> Self {
        Self {
            sensor_type: SensorType::IMU,
            sensor_id,
            timestamp_ns,
            sequence,
            data: SensorData::IMU { ax, ay, az, gx, gy, gz },
        }
    }

    pub fn new_lidar(sensor_id: u32, timestamp_ns: u64, sequence: u64, points: u32, range_m: f32) -> Self {
        Self {
            sensor_type: SensorType::LiDAR,
            sensor_id,
            timestamp_ns,
            sequence,
            data: SensorData::LiDAR { points, range_m },
        }
    }

    pub fn new_camera(sensor_id: u32, timestamp_ns: u64, sequence: u64, width: u16, height: u16, exposure_us: u32) -> Self {
        Self {
            sensor_type: SensorType::Camera,
            sensor_id,
            timestamp_ns,
            sequence,
            data: SensorData::Camera { width, height, exposure_us },
        }
    }
}
