use std::cell::UnsafeCell;
use std::hint::spin_loop;
use std::sync::atomic::{fence, AtomicU64, Ordering};

use crate::error::Result;
use crate::slot::Slot;

#[repr(align(64))]
struct PaddedU64 {
    value: AtomicU64,
}

impl PaddedU64 {
    const fn new(v: u64) -> Self {
        Self { value: AtomicU64::new(v) }
    }
}

pub struct MpmcRingBuffer<T, const N: usize> {
    slots: UnsafeCell<Vec<Slot<T>>>,
    mask: usize,
    write_cursor: PaddedU64,
    read_cursor: PaddedU64,
}

impl<T, const N: usize> MpmcRingBuffer<T, N> {
    pub fn new() -> Self {
        assert!(N.is_power_of_two(), "capacity must be power of 2");

        let mut vec = Vec::with_capacity(N);
        for _ in 0..N {
            vec.push(Slot::new(0));
        }

        Self {
            slots: UnsafeCell::new(vec),
            mask: N - 1,
            write_cursor: PaddedU64::new(0),
            read_cursor: PaddedU64::new(0),
        }
    }

    pub fn try_write(&self, value: T) -> Result<u64> {
        let idx = loop {
            let write_seq = self.write_cursor.value.load(Ordering::Acquire);
            let next_seq = write_seq.wrapping_add(1);
            let i = next_seq as usize & self.mask;

            let slots = unsafe { &(*self.slots.get()) };
            let seq = slots[i].sequence.value.load(Ordering::Acquire);

            if seq >= next_seq {
                spin_loop();
                continue;
            }

            match self.write_cursor.value.compare_exchange_weak(
                write_seq, next_seq, Ordering::AcqRel, Ordering::Acquire,
            ) {
                Ok(_) => break i,
                Err(_) => {
                    spin_loop();
                    continue;
                }
            }
        };

        let next_seq = self.write_cursor.value.load(Ordering::Relaxed);
        let slots = unsafe { &mut (*self.slots.get()) };

        unsafe { slots[idx].write(value); }
        fence(Ordering::SeqCst);
        slots[idx].sequence.value.store(next_seq, Ordering::Release);

        Ok(next_seq)
    }

    pub fn try_read_batch(&self, dest: &mut Vec<T>, max_count: usize) -> Result<usize> {
        let mut count = 0usize;
        let slots = unsafe { &(*self.slots.get()) };

        for _ in 0..max_count {
            let read_seq = self.read_cursor.value.load(Ordering::Relaxed);
            let next_seq = read_seq.wrapping_add(1);
            let idx = next_seq as usize & self.mask;

            let seq = slots[idx].sequence.value.load(Ordering::Acquire);

            if seq != next_seq {
                break;
            }

            let value = unsafe { slots[idx].read() };
            fence(Ordering::Acquire);

            unsafe { slots[idx].drop_value(); }

            self.read_cursor.value.store(next_seq, Ordering::Release);
            dest.push(value);
            count += 1;
        }

        Ok(count)
    }

    pub fn capacity(&self) -> usize {
        self.mask + 1
    }
}

unsafe impl<T: Send, const N: usize> Send for MpmcRingBuffer<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for MpmcRingBuffer<T, N> {}
