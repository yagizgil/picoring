use crate::buffer::MirrorBuffer;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// a wrapper for AtomicUsize that ensures 64-byte alignment.
// this prevents "false sharing" where the head and tail pointers
// reside in the same CPU cache line, significantly boosting performance.
#[repr(align(64))]
pub(crate) struct PaddedAtomic(pub(crate) AtomicUsize);

impl core::ops::Deref for PaddedAtomic {
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// core state shared between the producer and the consumer.
pub(crate) struct SharedState<T> {
    // the virtual memory mirrored buffer.
    pub(crate) buffer: MirrorBuffer,
    // write index, owned/updated by the producer.
    pub(crate) head: PaddedAtomic,
    // read index, owned/updated by the consumer.
    pub(crate) tail: PaddedAtomic,
    // maximum number of items the buffer can hold.
    pub(crate) capacity: usize,
    // bitmask for fast wrapping (if capacity is a power of two).
    pub(crate) mask: usize,
    pub(crate) _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for SharedState<T> {}
unsafe impl<T: Send> Sync for SharedState<T> {}

// the main handle for a single-producer single-consumer ring buffer.
pub struct PicoSPSC<T> {
    shared: Arc<SharedState<T>>,
}

// the writing half of the SPSC buffer.
pub struct PicoProducer<T> {
    shared: Arc<SharedState<T>>,
}

// the reading half of the SPSC buffer.
pub struct PicoConsumer<T> {
    shared: Arc<SharedState<T>>,
}

impl<T> PicoSPSC<T> {
    // creates a new SPSC ring buffer with the given capacity.
    pub fn new(capacity_count: usize) -> Result<Self, String> {
        let item_size = core::mem::size_of::<T>();
        if item_size == 0 {
            return Err("Zero sized types are not supported in PicoSPSC".into());
        }
        let total_bytes = capacity_count
            .checked_mul(item_size)
            .ok_or_else(|| "Requested capacity is too large (overflow)".to_string())?;

        let buffer = MirrorBuffer::new(total_bytes)?;
        let actual_capacity = buffer.size() / item_size;
        let mask = if actual_capacity > 0 && (actual_capacity & (actual_capacity - 1)) == 0 {
            actual_capacity - 1
        } else {
            0
        };

        // initialize shared state with padded atomics.
        let shared = Arc::new(SharedState {
            buffer,
            head: PaddedAtomic(AtomicUsize::new(0)),
            tail: PaddedAtomic(AtomicUsize::new(0)),
            capacity: actual_capacity,
            mask,
            _marker: PhantomData,
        });

        Ok(Self { shared })
    }

    // splits the SPSC buffer into its producer and consumer halves.
    // this allows moving each half into a different thread.
    pub fn split(self) -> (PicoProducer<T>, PicoConsumer<T>) {
        (
            PicoProducer {
                shared: self.shared.clone(),
            },
            PicoConsumer {
                shared: self.shared,
            },
        )
    }
}

impl<T> PicoProducer<T> {
    // internal helper to wrap indexes based on capacity or mask.
    #[inline]
    fn wrap(&self, val: usize) -> usize {
        if self.shared.mask != 0 {
            val & self.shared.mask
        } else {
            val % self.shared.capacity
        }
    }

    // attempts to push a single item into the buffer.
    // returns false if the buffer is full.
    // utilizes Ordering::Release to ensure the item write is visible before head is updated.
    #[inline]
    pub fn push(&self, item: T) -> bool {
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);

        let next_head = self.wrap(head + 1);
        if next_head == tail {
            return false;
        }

        unsafe {
            let ptr = (self.shared.buffer.as_mut_ptr() as *mut T).add(head);
            ptr::write(ptr, item);
        }

        self.shared.head.store(next_head, Ordering::Release);
        true
    }

    // returns the number of items that can still be written.
    pub fn available_space(&self) -> usize {
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);

        if head >= tail {
            self.shared.capacity - (head - tail) - 1
        } else {
            tail - head - 1
        }
    }

    // returns a mutable slice of the available writable space.
    // thanks to memory mirroring, this is always a contiguous slice.
    #[inline]
    pub fn writable_slice(&self) -> &mut [T] {
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);
        let space = if head >= tail {
            self.shared.capacity - (head - tail) - 1
        } else {
            tail - head - 1
        };
        unsafe {
            let ptr = (self.shared.buffer.as_mut_ptr() as *mut T).add(head);
            core::slice::from_raw_parts_mut(ptr, space)
        }
    }

    // manually advances the head pointer by n.
    // use this after writing directly to the writable_slice.
    #[inline]
    pub fn advance_head(&self, n: usize) {
        let head = self.shared.head.load(Ordering::Relaxed);
        self.shared
            .head
            .store(self.wrap(head + n), Ordering::Release);
    }
}

impl<T: Copy> PicoProducer<T> {
    // pushes a slice of items into the buffer in a single operation.
    // returns false if there isn't enough space.
    pub fn push_slice(&self, data: &[T]) -> bool {
        let n = data.len();
        let head = self.shared.head.load(Ordering::Relaxed);
        let tail = self.shared.tail.load(Ordering::Acquire);

        let current_len = if head >= tail {
            head - tail
        } else {
            self.shared.capacity - (tail - head)
        };

        if self.shared.capacity - current_len - 1 < n {
            return false;
        }

        unsafe {
            let dest_ptr = (self.shared.buffer.as_mut_ptr() as *mut T).add(head);
            ptr::copy_nonoverlapping(data.as_ptr(), dest_ptr, n);
        }

        self.shared
            .head
            .store(self.wrap(head + n), Ordering::Release);
        true
    }
}

impl<T> PicoConsumer<T> {
    // internal helper to wrap indexes.
    #[inline]
    fn wrap(&self, val: usize) -> usize {
        if self.shared.mask != 0 {
            val & self.shared.mask
        } else {
            val % self.shared.capacity
        }
    }

    // attempts to pop a single item from the buffer.
    // returns None if the buffer is empty.
    // utilizes Ordering::Acquire to ensure the data is read after the producer's update is seen.
    #[inline]
    pub fn pop(&self) -> Option<T> {
        let tail = self.shared.tail.load(Ordering::Relaxed);
        let head = self.shared.head.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let item = unsafe {
            let ptr = (self.shared.buffer.as_mut_ptr() as *const T).add(tail);
            ptr::read(ptr)
        };

        self.shared
            .tail
            .store(self.wrap(tail + 1), Ordering::Release);
        Some(item)
    }

    // returns the number of items currently available for reading.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.shared.head.load(Ordering::Acquire);
        let tail = self.shared.tail.load(Ordering::Relaxed);

        if head >= tail {
            head - tail
        } else {
            self.shared.capacity - (tail - head)
        }
    }

    // returns true if the buffer contains no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // returns a contiguous slice of all available readable data.
    // hardware mirroring ensures this is a single slice even if it wraps.
    #[inline]
    pub fn readable_slice(&self) -> &[T] {
        let head = self.shared.head.load(Ordering::Acquire);
        let tail = self.shared.tail.load(Ordering::Relaxed);

        let len = if head >= tail {
            head - tail
        } else {
            self.shared.capacity - (tail - head)
        };

        unsafe {
            let ptr = (self.shared.buffer.as_mut_ptr() as *const T).add(tail);
            core::slice::from_raw_parts(ptr, len)
        }
    }

    // manually advances the tail pointer by n.
    // use this after reading directly from the readable_slice.
    #[inline]
    pub fn advance_tail(&self, n: usize) {
        let tail = self.shared.tail.load(Ordering::Relaxed);
        self.shared
            .tail
            .store(self.wrap(tail + n), Ordering::Release);
    }
}

// creates a new SPSC ring buffer with the given capacity.
// deprecated: use PicoSPSC::new(capacity).split() instead.
pub fn create_spsc<T>(capacity_count: usize) -> Result<(PicoProducer<T>, PicoConsumer<T>), String> {
    Ok(PicoSPSC::new(capacity_count)?.split())
}

impl<T, const N: usize> crate::ring::PicoRing<T, N> {
    // converts an existing PicoRing into its SPSC counterparts.
    // this consumes the original ring and enables multi-threaded lock-free access.
    pub fn into_spsc(self) -> (PicoProducer<T>, PicoConsumer<T>) {
        let shared = Arc::new(SharedState {
            buffer: self.buffer,
            head: PaddedAtomic(AtomicUsize::new(self.head)),
            tail: PaddedAtomic(AtomicUsize::new(self.tail)),
            capacity: self.capacity,
            mask: self.mask,
            _marker: PhantomData,
        });

        (
            PicoProducer {
                shared: shared.clone(),
            },
            PicoConsumer { shared },
        )
    }

    // converts an existing PicoRing into its MPSC counterparts.
    pub fn into_mpsc(self) -> (crate::mpsc::PicoMpscProducer<T>, crate::mpsc::PicoMpscConsumer<T>) {
        let shared = Arc::new(crate::mpsc::SharedState {
            buffer: self.buffer,
            head: crate::mpsc::PaddedAtomic(AtomicUsize::new(self.head)),
            tail: crate::mpsc::PaddedAtomic(AtomicUsize::new(self.tail)),
            commit: crate::mpsc::PaddedAtomic(AtomicUsize::new(self.head)),
            capacity: self.capacity,
            mask: self.mask,
            _marker: PhantomData,
        });

        (
            crate::mpsc::PicoMpscProducer {
                shared: shared.clone(),
            },
            crate::mpsc::PicoMpscConsumer { shared },
        )
    }
}
