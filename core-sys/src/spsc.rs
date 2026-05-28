//! Lock-Free Single Producer Single Consumer (SPSC) Circular Queue
//!
//! Provides high-performance, non-blocking data transfer between a single writer
//! thread and a single reader thread. Used for routing telemetry events to the
//! background logging daemon.

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A lock-free Single Producer Single Consumer (SPSC) queue of fixed capacity `N`.
///
/// Designed for hardware-sympathetic, low-overhead communication between threads.
pub struct SpscQueue<T, const N: usize> {
    buffer: [UnsafeCell<MaybeUninit<T>>; N],
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

unsafe impl<T: Send, const N: usize> Send for SpscQueue<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for SpscQueue<T, N> {}

impl<T, const N: usize> SpscQueue<T, N> {
    /// Creates a new, empty SpscQueue.
    pub fn new() -> Self {
        // Safe initialization of const-generic array containing UnsafeCell
        // Since UnsafeCell and MaybeUninit are transparent wrappers with no drop implementation
        // on uninitialized states, this is safe and memory-efficient.
        let buffer = unsafe {
            let array: MaybeUninit<[UnsafeCell<MaybeUninit<T>>; N]> = MaybeUninit::uninit();
            array.assume_init()
        };

        Self {
            buffer,
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        }
    }

    /// Attempts to push an item onto the queue.
    ///
    /// Returns `Err(value)` if the queue is full.
    /// Safe to call only from a single producer thread.
    pub fn push(&self, value: T) -> Result<(), T> {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Acquire); // Synchronize with consumer read progress

        if write - read >= N {
            // Queue capacity reached
            return Err(value);
        }

        let slot_idx = write % N;
        unsafe {
            let slot_ptr = self.buffer[slot_idx].get();
            std::ptr::write(slot_ptr, MaybeUninit::new(value));
        }

        // Increment write pointer, publishing the value to the consumer
        self.write_idx.store(write + 1, Ordering::Release);
        Ok(())
    }

    /// Attempts to pop an item from the queue.
    ///
    /// Returns `None` if the queue is empty.
    /// Safe to call only from a single consumer thread.
    pub fn pop(&self) -> Option<T> {
        let read = self.read_idx.load(Ordering::Relaxed);
        let write = self.write_idx.load(Ordering::Acquire); // Synchronize with producer write progress

        if read == write {
            // Queue is empty
            return None;
        }

        let slot_idx = read % N;
        let value = unsafe {
            let slot_ptr = self.buffer[slot_idx].get();
            let maybe_uninit = std::ptr::read(slot_ptr);
            maybe_uninit.assume_init()
        };

        // Increment read pointer, letting the producer know the slot is free
        self.read_idx.store(read + 1, Ordering::Release);
        Some(value)
    }

    /// Returns the approximate number of items currently in the queue.
    pub fn len(&self) -> usize {
        let write = self.write_idx.load(Ordering::Relaxed);
        let read = self.read_idx.load(Ordering::Relaxed);
        write.saturating_sub(read)
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, const N: usize> Default for SpscQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for SpscQueue<T, N> {
    fn drop(&mut self) {
        // Drain and drop any remaining elements in the queue
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_spsc_concurrency_race() {
        let queue = Arc::new(SpscQueue::<usize, 512>::new());
        let q_prod = queue.clone();
        let q_cons = queue.clone();

        let num_items = 50000;

        // Spawn producer thread
        let prod_handle = thread::spawn(move || {
            let mut i = 0;
            while i < num_items {
                match q_prod.push(i) {
                    Ok(_) => i += 1,
                    Err(_) => {
                        // Queue is full, yield and try again
                        thread::yield_now();
                    }
                }
            }
        });

        // Spawn consumer thread
        let cons_handle = thread::spawn(move || {
            let mut expected = 0;
            while expected < num_items {
                match q_cons.pop() {
                    Some(val) => {
                        assert_eq!(val, expected, "Data sequence is corrupt!");
                        expected += 1;
                    }
                    None => {
                        // Queue is empty, yield and try again
                        thread::yield_now();
                    }
                }
            }
        });

        let _ = prod_handle.join();
        let _ = cons_handle.join();
    }
}
