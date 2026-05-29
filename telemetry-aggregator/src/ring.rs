use std::sync::atomic::{fence, AtomicBool, AtomicU16, AtomicUsize, Ordering};

use crate::error::Result;

const DEFAULT_FRAME_COUNT: usize = 256;
const DEFAULT_FRAME_SIZE: usize = 2048;
const RING_TOTAL_SIZE: usize = DEFAULT_FRAME_COUNT * DEFAULT_FRAME_SIZE;

pub struct PacketFrame {
    data: UnsafeCell<[u8; DEFAULT_FRAME_SIZE]>,
    pub len: AtomicU16,
    pub ready: AtomicBool,
}

unsafe impl Sync for PacketFrame {}

use std::cell::UnsafeCell;

impl PacketFrame {
    fn new() -> Self {
        Self {
            data: UnsafeCell::new([0u8; DEFAULT_FRAME_SIZE]),
            len: AtomicU16::new(0),
            ready: AtomicBool::new(false),
        }
    }
}

pub struct PacketRing {
    frames: Box<[PacketFrame]>,
    read_idx: AtomicUsize,
}

impl PacketRing {
    pub fn new() -> Self {
        let mut frames_vec = Vec::with_capacity(DEFAULT_FRAME_COUNT);
        for _ in 0..DEFAULT_FRAME_COUNT {
            frames_vec.push(PacketFrame::new());
        }

        Self {
            frames: frames_vec.into_boxed_slice(),
            read_idx: AtomicUsize::new(0),
        }
    }
}

impl Default for PacketRing {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketRing {
    pub fn has_available(&self) -> bool {
        let idx = self.read_idx.load(Ordering::Relaxed);
        self.frames[idx].ready.load(Ordering::Acquire)
    }

    pub fn read_frame(&self) -> Option<(&[u8], usize)> {
        let idx = self.read_idx.load(Ordering::Relaxed);

        if !self.frames[idx].ready.load(Ordering::Acquire) {
            return None;
        }

        let len = self.frames[idx].len.load(Ordering::Acquire) as usize;
        let data = unsafe {
            let data_ptr = self.frames[idx].data.get();
            std::slice::from_raw_parts(data_ptr as *const u8, len)
        };

        Some((data, idx))
    }

    pub fn mark_consumed(&self, idx: usize) {
        self.frames[idx].ready.store(false, Ordering::Release);
        self.frames[idx].len.store(0, Ordering::Release);

        let next = (idx + 1) % self.frames.len();
        self.read_idx.store(next, Ordering::Release);
    }

    pub fn write_frame(&self, data: &[u8]) -> Result<usize> {
        let write_len = data.len().min(DEFAULT_FRAME_SIZE);

        let start_idx = self.read_idx.load(Ordering::Relaxed);
        let mut idx = start_idx;

        loop {
            if !self.frames[idx].ready.load(Ordering::Acquire) {
                let data_ptr = self.frames[idx].data.get();
                unsafe { data_ptr.write([0u8; DEFAULT_FRAME_SIZE]); }
                let buf = unsafe { &mut *data_ptr };
                buf[..write_len].copy_from_slice(&data[..write_len]);
                fence(Ordering::SeqCst);
                self.frames[idx].len.store(write_len as u16, Ordering::Release);
                fence(Ordering::SeqCst);
                self.frames[idx].ready.store(true, Ordering::Release);
                return Ok(idx);
            }

            idx = (idx + 1) % self.frames.len();
            if idx == start_idx {
                return Err(crate::error::AggregatorError::BufferFull);
            }
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn frame_size(&self) -> usize {
        DEFAULT_FRAME_SIZE
    }

    pub fn total_size(&self) -> usize {
        RING_TOTAL_SIZE
    }
}

unsafe impl Send for PacketRing {}
unsafe impl Sync for PacketRing {}
