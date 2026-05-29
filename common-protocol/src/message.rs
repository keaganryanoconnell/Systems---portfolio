#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    SqlQuery = 10,
    SqlResult = 11,
    RaftAppend = 20,
    RaftRequestVote = 21,
    RaftInstallSnapshot = 22,
    StoragePut = 30,
    StorageGet = 31,
    StorageScan = 32,
    StorageDelete = 33,
    ComputeTask = 40,
    ComputeResult = 41,
    ComputeHealth = 42,
    BrokerProduce = 50,
    BrokerFetch = 51,
    BrokerCommit = 52,
    ContainerRun = 60,
    ContainerStatus = 61,
    ContainerKill = 62,
    HealthCheck = 70,
    TelemetryQuery = 80,
    TelemetryResponse = 81,
}

impl MessageType {
    pub fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(n: u32) -> Option<Self> {
        match n {
            10 => Some(Self::SqlQuery),
            11 => Some(Self::SqlResult),
            20 => Some(Self::RaftAppend),
            21 => Some(Self::RaftRequestVote),
            22 => Some(Self::RaftInstallSnapshot),
            30 => Some(Self::StoragePut),
            31 => Some(Self::StorageGet),
            32 => Some(Self::StorageScan),
            33 => Some(Self::StorageDelete),
            40 => Some(Self::ComputeTask),
            41 => Some(Self::ComputeResult),
            42 => Some(Self::ComputeHealth),
            50 => Some(Self::BrokerProduce),
            51 => Some(Self::BrokerFetch),
            52 => Some(Self::BrokerCommit),
            60 => Some(Self::ContainerRun),
            61 => Some(Self::ContainerStatus),
            62 => Some(Self::ContainerKill),
            70 => Some(Self::HealthCheck),
            80 => Some(Self::TelemetryQuery),
            81 => Some(Self::TelemetryResponse),
            _ => None,
        }
    }
}
