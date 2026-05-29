pub struct IngestStats {
    pub packets_received: u64,
    pub points_processed: u64,
    pub bytes_ingested: u64,
    pub bytes_compressed: u64,
    pub blocks_written: u64,
    pub segments_flushed: u64,
}

impl IngestStats {
    pub fn new() -> Self {
        Self {
            packets_received: 0,
            points_processed: 0,
            bytes_ingested: 0,
            bytes_compressed: 0,
            blocks_written: 0,
            segments_flushed: 0,
        }
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.bytes_compressed == 0 {
            return 0.0;
        }
        self.bytes_ingested as f64 / self.bytes_compressed as f64
    }
}
