use bytes::BytesMut;

pub struct IngestBuffer {
    buf: BytesMut,
    max_capacity: usize,
}

impl IngestBuffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(max_capacity.min(65536)),
            max_capacity,
        }
    }

    pub fn extend(&mut self, data: &[u8]) -> Result<usize, crate::error::IngestError> {
        if self.buf.len() + data.len() > self.max_capacity {
            return Err(crate::error::IngestError::BufferFull);
        }
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }

    pub fn drain(&mut self) -> BytesMut {
        std::mem::replace(&mut self.buf, BytesMut::with_capacity(65536))
    }

    pub fn available(&self) -> usize {
        self.max_capacity - self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

pub struct Pipeline {
    buffer: IngestBuffer,
    blocks_processed: u64,
    bytes_ingested: u64,
}

impl Pipeline {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            buffer: IngestBuffer::new(max_capacity),
            blocks_processed: 0,
            bytes_ingested: 0,
        }
    }

    pub fn ingest(&mut self, data: &[u8]) -> crate::error::Result<usize> {
        let n = self.buffer.extend(data)?;
        self.bytes_ingested += n as u64;

        if self.buffer.len() >= 4096 {
            self.process_block();
        }

        Ok(n)
    }

    fn process_block(&mut self) {
        let _block = self.buffer.drain();
        self.blocks_processed += 1;
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.blocks_processed, self.bytes_ingested)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_new_is_empty() {
        let buf = IngestBuffer::new(65536);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.available(), 65536);
    }

    #[test]
    fn test_buffer_extend_and_drain() {
        let mut buf = IngestBuffer::new(1024);
        let data = b"hello world test data";
        let n = buf.extend(data).unwrap();
        assert_eq!(n, data.len());
        assert_eq!(buf.len(), data.len());

        let drained = buf.drain();
        assert_eq!(&drained[..], data);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffer_rejects_overflow() {
        let mut buf = IngestBuffer::new(10);
        assert!(buf.extend(&[0u8; 11]).is_err());
        assert!(buf.is_empty());
    }

    #[test]
    fn test_pipeline_ingest_and_stats() {
        let mut pipe = Pipeline::new(65536);
        let data = vec![0u8; 5000];
        let n = pipe.ingest(&data).unwrap();
        assert_eq!(n, 5000);

        let (blocks, bytes) = pipe.stats();
        assert_eq!(bytes, 5000);
        assert_eq!(blocks, 1);
    }

    #[test]
    fn test_pipeline_multiple_ingest_triggers_blocks() {
        let mut pipe = Pipeline::new(65536);
        for _ in 0..5 {
            pipe.ingest(&[0u8; 4096]).unwrap();
        }
        let (blocks, bytes) = pipe.stats();
        assert_eq!(bytes, 5 * 4096);
        assert_eq!(blocks, 5);
    }

    #[test]
    fn test_buffer_available_decreases_after_extend() {
        let mut buf = IngestBuffer::new(1000);
        assert_eq!(buf.available(), 1000);
        buf.extend(&[0u8; 300]).unwrap();
        assert_eq!(buf.available(), 700);
    }
}
