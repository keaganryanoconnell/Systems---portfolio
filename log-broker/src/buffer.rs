use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::BrokerResult;

const RING_CAPACITY: usize = 1 << 20;

pub struct RingBuffer {
    buffer: Vec<u8>,
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl RingBuffer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let capacity = RING_CAPACITY;
        assert!(capacity.is_power_of_two(), "capacity must be a power of 2");

        let buffer = vec![0u8; capacity];

        Self {
            buffer,
            mask: capacity - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn available_write(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Relaxed);
        self.capacity() - (head.wrapping_sub(tail))
    }

    pub fn available_read(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    pub fn try_write(&self, data: &[u8]) -> BrokerResult<usize> {
        let len = data.len();
        if len == 0 {
            return Ok(0);
        }

        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let available = self.capacity() - (head.wrapping_sub(tail));

        if len > available {
            return Err(crate::error::BrokerError::BufferFull);
        }

        let head_idx = head & self.mask;
        let write_len = len.min(self.capacity() - head_idx);

        let buf = &self.buffer;
        let buf_ptr = buf.as_ptr() as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), buf_ptr.add(head_idx), write_len);
            if write_len < len {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr().add(write_len),
                    buf_ptr,
                    len - write_len,
                );
            }
        }

        std::sync::atomic::compiler_fence(Ordering::SeqCst);
        self.head.store(head.wrapping_add(len), Ordering::Release);

        Ok(len)
    }

    pub fn try_read(&self, dest: &mut [u8]) -> BrokerResult<usize> {
        let read_len = dest.len();
        if read_len == 0 {
            return Ok(0);
        }

        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = head.wrapping_sub(tail);

        if available == 0 {
            return Ok(0);
        }

        let actual = read_len.min(available);
        let tail_idx = tail & self.mask;
        let first_chunk = actual.min(self.capacity() - tail_idx);

        let buf = &self.buffer;
        let buf_ptr = buf.as_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(buf_ptr.add(tail_idx), dest.as_mut_ptr(), first_chunk);
            if first_chunk < actual {
                std::ptr::copy_nonoverlapping(
                    buf_ptr,
                    dest.as_mut_ptr().add(first_chunk),
                    actual - first_chunk,
                );
            }
        }

        std::sync::atomic::compiler_fence(Ordering::SeqCst);
        self.tail
            .store(tail.wrapping_add(actual), Ordering::Release);

        Ok(actual)
    }

    pub fn drain_into<F>(&self, mut consume: F) -> BrokerResult<usize>
    where
        F: FnMut(&[u8]) -> BrokerResult<usize>,
    {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = head.wrapping_sub(tail);

        if available == 0 {
            return Ok(0);
        }

        let mut total = 0usize;
        let mut read_pos = tail;

        while read_pos != head {
            let idx = read_pos & self.mask;
            let chunk_len = (head.wrapping_sub(read_pos)).min(self.capacity() - idx);
            let consumed = consume(&self.buffer[idx..idx + chunk_len])?;
            total += consumed;
            read_pos = read_pos.wrapping_add(consumed);
            if consumed < chunk_len {
                break;
            }
        }

        if total > 0 {
            std::sync::atomic::compiler_fence(Ordering::SeqCst);
            self.tail.store(tail.wrapping_add(total), Ordering::Release);
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_roundtrip() {
        let rb = RingBuffer::new();
        let data = b"hello world";

        let written = rb.try_write(data).unwrap();
        assert_eq!(written, 11);

        let mut dest = vec![0u8; 20];
        let read = rb.try_read(&mut dest).unwrap();
        assert_eq!(read, 11);
        assert_eq!(&dest[..11], data);
    }

    #[test]
    fn test_empty_read_returns_zero() {
        let rb = RingBuffer::new();
        let mut dest = vec![0u8; 10];
        let read = rb.try_read(&mut dest).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_write_exceeding_capacity_fails() {
        let rb = RingBuffer::new();
        let data = vec![0u8; RING_CAPACITY + 1];
        let result = rb.try_write(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_wraparound() {
        let rb = RingBuffer::new();
        let cap = rb.capacity();

        let fill = vec![0xAAu8; cap / 2];
        rb.try_write(&fill).unwrap();

        let mut drain = vec![0u8; cap / 2];
        rb.try_read(&mut drain).unwrap();

        let data = b"wrap test data";
        rb.try_write(data).unwrap();

        let mut dest = vec![0u8; data.len()];
        let read = rb.try_read(&mut dest).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&dest, data);
    }
}
