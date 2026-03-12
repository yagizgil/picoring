use crate::buffer::MirrorBuffer;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// a wrapper for AtomicUsize that ensures 64-byte alignment.
// this prevents "false sharing" where pointers reside in the same CPU cache line.
#[repr(align(64))]
pub(crate) struct PaddedAtomic(pub(crate) AtomicUsize);

impl core::ops::Deref for PaddedAtomic {
    type Target = AtomicUsize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// core state shared between multiple producers and a single consumer.
pub(crate) struct SharedState<T> {
    pub(crate) buffer: MirrorBuffer,
    // reservation index, updated by producers using compare_exchange.
    pub(crate) head: PaddedAtomic,
    // read index, owned/updated by the consumer.
    pub(crate) tail: PaddedAtomic,
    // commit index, updated by producers after they finish writing.
    // the consumer can only read up to this point.
    pub(crate) commit: PaddedAtomic,
    pub(crate) capacity: usize,
    pub(crate) mask: usize,
    pub(crate) _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for SharedState<T> {}
unsafe impl<T: Send> Sync for SharedState<T> {}

// main handle for a multi-producer single-consumer ring buffer.
pub struct PicoMPSC<T> {
    shared: Arc<SharedState<T>>,
}

// writing half of the mpsc buffer. can be cloned for multiple producers.
pub struct PicoMpscProducer<T> {
    pub(crate) shared: Arc<SharedState<T>>,
}

// reading half of the mpsc buffer. only one consumer should exist.
pub struct PicoMpscConsumer<T> {
    pub(crate) shared: Arc<SharedState<T>>,
}

impl<T> Clone for PicoMpscProducer<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<T> PicoMPSC<T> {
    // creates a new mpsc ring buffer with the given capacity.
    pub fn new(capacity_count: usize) -> Result<Self, String> {
        let item_size = core::mem::size_of::<T>();
        if item_size == 0 {
            return Err("Zero sized types are not supported in PicoMPSC".into());
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

        let shared = Arc::new(SharedState {
            buffer,
            head: PaddedAtomic(AtomicUsize::new(0)),
            tail: PaddedAtomic(AtomicUsize::new(0)),
            commit: PaddedAtomic(AtomicUsize::new(0)),
            capacity: actual_capacity,
            mask,
            _marker: PhantomData,
        });

        Ok(Self { shared })
    }

    // splits the mpsc buffer into its producer and consumer halves.
    pub fn split(self) -> (PicoMpscProducer<T>, PicoMpscConsumer<T>) {
        (
            PicoMpscProducer {
                shared: self.shared.clone(),
            },
            PicoMpscConsumer {
                shared: self.shared,
            },
        )
    }
}

impl<T> PicoMpscProducer<T> {
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
    // uses wait-free space reservation bit blocks briefly during commit
    // to ensure strict fifo ordering for the consumer.
    #[inline]
    pub fn push(&self, item: T) -> bool {
        let mut head = self.shared.head.load(Ordering::Relaxed);
        let mut next_head;

        loop {
            let tail = self.shared.tail.load(Ordering::Acquire);

            let current_len = if head >= tail {
                head - tail
            } else {
                self.shared.capacity - (tail - head)
            };

            if current_len + 1 >= self.shared.capacity {
                return false;
            }

            next_head = self.wrap(head + 1);

            match self.shared.head.compare_exchange_weak(
                head,
                next_head,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(h) => head = h,
            }
        }

        unsafe {
            let ptr = (self.shared.buffer.as_mut_ptr() as *mut T).add(head);
            ptr::write(ptr, item);
        }

        // wait for our turn to commit (important for hardware mirroring consistency).
        while self.shared.commit.load(Ordering::Acquire) != head {
            core::hint::spin_loop();
        }

        self.shared.commit.store(next_head, Ordering::Release);
        true
    }
}

impl<T: Copy> PicoMpscProducer<T> {
    // pushes a slice of items into the buffer in a single operation.
    // returns false if there isn't enough space.
    pub fn push_slice(&self, data: &[T]) -> bool {
        let n = data.len();
        if n == 0 {
            return true;
        }

        let mut head = self.shared.head.load(Ordering::Relaxed);
        let mut next_head;

        loop {
            let tail = self.shared.tail.load(Ordering::Acquire);
            let current_len = if head >= tail {
                head - tail
            } else {
                self.shared.capacity - (tail - head)
            };

            if current_len + n >= self.shared.capacity {
                return false;
            }

            next_head = self.wrap(head + n);

            match self.shared.head.compare_exchange_weak(
                head,
                next_head,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(h) => head = h,
            }
        }

        unsafe {
            let dest_ptr = (self.shared.buffer.as_mut_ptr() as *mut T).add(head);
            ptr::copy_nonoverlapping(data.as_ptr(), dest_ptr, n);
        }

        while self.shared.commit.load(Ordering::Acquire) != head {
            core::hint::spin_loop();
        }

        self.shared.commit.store(next_head, Ordering::Release);
        true
    }
}

impl<T> PicoMpscConsumer<T> {
    #[inline]
    fn wrap(&self, val: usize) -> usize {
        if self.shared.mask != 0 {
            val & self.shared.mask
        } else {
            val % self.shared.capacity
        }
    }

    // attempts to pop a single item from the buffer.
    // returns none if the buffer is empty.
    #[inline]
    pub fn pop(&self) -> Option<T> {
        let tail = self.shared.tail.load(Ordering::Relaxed);
        let commit = self.shared.commit.load(Ordering::Acquire);

        if tail == commit {
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
        let commit = self.shared.commit.load(Ordering::Acquire);
        let tail = self.shared.tail.load(Ordering::Relaxed);

        if commit >= tail {
            commit - tail
        } else {
            self.shared.capacity - (tail - commit)
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
        let commit = self.shared.commit.load(Ordering::Acquire);
        let tail = self.shared.tail.load(Ordering::Relaxed);

        let len = if commit >= tail {
            commit - tail
        } else {
            self.shared.capacity - (tail - commit)
        };

        unsafe {
            let ptr = (self.shared.buffer.as_mut_ptr() as *const T).add(tail);
            core::slice::from_raw_parts(ptr, len)
        }
    }

    // manually advances the tail pointer by n.
    #[inline]
    pub fn advance_tail(&self, n: usize) {
        let tail = self.shared.tail.load(Ordering::Relaxed);
        self.shared
            .tail
            .store(self.wrap(tail + n), Ordering::Release);
    }
}

// creates a new mpsc ring buffer with the given capacity.
pub fn create_mpsc<T>(
    capacity_count: usize,
) -> Result<(PicoMpscProducer<T>, PicoMpscConsumer<T>), String> {
    Ok(PicoMPSC::new(capacity_count)?.split())
}
