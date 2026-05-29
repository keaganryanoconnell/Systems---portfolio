use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU64;

#[repr(align(64))]
pub struct CachePaddedAtomicU64 {
    pub value: AtomicU64,
}

impl CachePaddedAtomicU64 {
    fn new(v: u64) -> Self {
        Self { value: AtomicU64::new(v) }
    }
}

pub struct Slot<T> {
    pub sequence: CachePaddedAtomicU64,
    data: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Slot<T> {
    pub fn new(seq: u64) -> Self {
        Self {
            sequence: CachePaddedAtomicU64::new(seq),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Writes a value into this slot.
    ///
    /// # Safety
    ///
    /// The caller must ensure no other thread is concurrently reading from
    /// or writing to this slot, as enforced by the ring buffer's CAS sequence protocol.
    pub unsafe fn write(&self, value: T) {
        (*self.data.get()).write(value);
    }

    /// Reads a value from this slot.
    ///
    /// # Safety
    ///
    /// The caller must ensure the slot contains a valid initialized value and no
    /// concurrent write is in progress. The ring buffer's sequence protocol guarantees
    /// this via Acquire/Release ordering.
    pub unsafe fn read(&self) -> T {
        (*self.data.get()).assume_init_read()
    }

    /// Drops the value stored in this slot without reading it.
    ///
    /// # Safety
    ///
    /// The caller must ensure the slot contains a valid initialized value and will
    /// not be read after this call. Used after `read()` to prevent double-drops.
    pub unsafe fn drop_value(&self) {
        (*self.data.get()).assume_init_drop();
    }
}

unsafe impl<T: Send> Send for Slot<T> {}
unsafe impl<T: Send> Sync for Slot<T> {}
