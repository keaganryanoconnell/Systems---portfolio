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

    pub unsafe fn write(&self, value: T) {
        (*self.data.get()).write(value);
    }

    pub unsafe fn read(&self) -> T {
        (*self.data.get()).assume_init_read()
    }

    pub unsafe fn drop_value(&self) {
        (*self.data.get()).assume_init_drop();
    }
}

unsafe impl<T: Send> Send for Slot<T> {}
unsafe impl<T: Send> Sync for Slot<T> {}
