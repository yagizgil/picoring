use crate::ring::PicoRing;

// A high-performance, std-independent queue that focuses on
// reservation-based zero-copy operations.
pub struct PicoQueue<T> {
    ring: PicoRing<T>,
}

impl<T> PicoQueue<T> {
    // create a new queue with specified capacity (in items of type T)
    pub fn new(capacity: usize) -> Result<Self, String> {
        Ok(Self {
            ring: PicoRing::new(capacity)?,
        })
    }

    // returns how many items are currently in the queue
    #[inline]
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    // returns the total capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.ring.capacity()
    }

    // --- PRODUCER API (Writing) ---

    // reserves a contiguous block of space for writing
    // returns None if not enough space is available
    #[inline]
    pub fn reserve(&mut self, len: usize) -> Option<&mut [T]> {
        if self.ring.available_space() < len {
            return None;
        }
        // with mirroring, any len <= capacity is always contiguous at ring.head()
        self.ring.view_mut(self.ring.head(), len)
    }

    // commits the written items, making them available for reading
    #[inline]
    pub fn commit(&mut self, len: usize) {
        self.ring.advance_head(len);
    }

    // --- CONSUMER API (Reading) ---

    // returns a contiguous slice of all items currently ready to be read
    #[inline]
    pub fn peek(&self) -> &[T] {
        self.ring.readable_slice()
    }

    // releases the oldest N items from the queue
    #[inline]
    pub fn release(&mut self, len: usize) {
        let n = len.min(self.len());
        self.ring.advance_tail(n);
    }
}

// Support for single item access without slices
impl<T: Copy> PicoQueue<T> {
    #[inline]
    pub fn try_push(&mut self, item: T) -> bool {
        self.ring.push(item)
    }

    #[inline]
    pub fn try_pop(&mut self) -> Option<T> {
        self.ring.pop()
    }
}

// Global conversion for PicoQueue
impl<T: Copy> From<Vec<T>> for PicoQueue<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            ring: PicoRing::from(v),
        }
    }
}
