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
