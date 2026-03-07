use crate::ring::PicoRing;

// A high-performance, std-independent queue that focuses on
// reservation-based zero-copy operations.
pub struct PicoQueue<T, const N: usize = 0> {
    ring: PicoRing<T, N>,
}

impl<T, const N: usize> PicoQueue<T, N> {
    // creates a new PicoQueue with a capacity determined by the const generic N.
    pub fn new_static() -> Result<Self, String> {
        Ok(Self {
            ring: PicoRing::new()?,
        })
    }
}

impl<T> PicoQueue<T, 0> {
    // creates a new PicoQueue with a specified dynamic capacity.
    pub fn new(capacity: usize) -> Result<Self, String> {
        Ok(Self {
            ring: PicoRing::with_capacity(capacity)?,
        })
    }
}

impl<T, const N: usize> PicoQueue<T, N> {
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

    // returns an iterator over the readable items
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.peek().iter()
    }

    // returns a mutable iterator over the readable items
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        // since we have hardware mirroring, readable_slice can be mutable too
        unsafe {
            let ptr = self.ring.as_mut_ptr().add(self.ring.tail());
            core::slice::from_raw_parts_mut(ptr, self.len()).iter_mut()
        }
    }
}

// support for single item access without slices
impl<T: Copy, const N: usize> PicoQueue<T, N> {
    #[inline]
    pub fn try_push(&mut self, item: T) -> bool {
        self.ring.push(item)
    }

    #[inline]
    pub fn try_pop(&mut self) -> Option<T> {
        self.ring.pop()
    }
}

// global conversion for PicoQueue
impl<T: Copy> From<Vec<T>> for PicoQueue<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            ring: PicoRing::from(v),
        }
    }
}

// -- Ergonomics: Indexing and Iteration --

impl<T, const N: usize> core::ops::Index<usize> for PicoQueue<T, N> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.peek()[index]
    }
}

impl<T, const N: usize> core::ops::IndexMut<usize> for PicoQueue<T, N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            let ptr = self.ring.as_mut_ptr().add(self.ring.tail());
            let slice = core::slice::from_raw_parts_mut(ptr, self.len());
            &mut slice[index]
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a PicoQueue<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut PicoQueue<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
