use crate::ring::PicoRing;
use core::ptr;

// a dynamic, chunked list that grows as needed without reallocating existing data.
// each chunk is a fixed-size buffer leveraging hardware mirroring.
// N is the number of items per chunk (preferably a power of two for speed).
pub struct PicoList<T, const N: usize = 16384> {
    // we use PicoRing<T, 0> internally to allocate N + 1 items,
    // ensuring we can fit exactly N items per chunk.
    chunks: Vec<PicoRing<T, 0>>,
    len: usize,
}

impl<T, const N: usize> PicoList<T, N> {
    // pre-calculated constants for high-speed indexing
    const IS_POW2: bool = N > 0 && (N & (N - 1)) == 0;
    const SHIFT: u32 = N.trailing_zeros();
    const MASK: usize = if N > 0 { N - 1 } else { 0 };

    // creates a new, empty PicoList.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            len: 0,
        }
    }

    // returns the total number of items in the list.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    // returns true if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // pushes an item to the end of the list.
    // uses bitwise optimization if N is a power of two.
    #[inline]
    pub fn push(&mut self, item: T) {
        let (chunk_idx, local_idx) = if Self::IS_POW2 {
            (self.len >> Self::SHIFT, self.len & Self::MASK)
        } else {
            (self.len / N, self.len % N)
        };

        // allocate a new chunk if needed
        if chunk_idx >= self.chunks.len() {
            self.chunks
                .push(PicoRing::with_capacity(N + 1).expect("failed to allocate chunk"));
        }

        unsafe {
            let chunk = self.chunks.get_unchecked_mut(chunk_idx);
            let ptr = (chunk.as_mut_ptr() as *mut T).add(local_idx);
            ptr::write(ptr, item);
            chunk.advance_head(1);
        }

        self.len += 1;
    }

    // returns a reference to the item at the specified index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(self.get_unchecked(index)) }
    }

    // returns a reference to the item at the specified index without bounds checking.
    // safety: index must be less than self.len().
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        let (chunk_idx, local_idx) = if Self::IS_POW2 {
            (index >> Self::SHIFT, index & Self::MASK)
        } else {
            (index / N, index % N)
        };

        unsafe {
            let chunk = self.chunks.get_unchecked(chunk_idx);
            let ptr = (chunk.as_mut_ptr() as *const T).add(local_idx);
            &*ptr
        }
    }

    // returns a mutable reference to the item at the specified index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(self.get_unchecked_mut(index)) }
    }

    // returns a mutable reference to the item at the specified index without bounds checking.
    // safety: index must be less than self.len().
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        let (chunk_idx, local_idx) = if Self::IS_POW2 {
            (index >> Self::SHIFT, index & Self::MASK)
        } else {
            (index / N, index % N)
        };

        unsafe {
            let chunk = self.chunks.get_unchecked_mut(chunk_idx);
            let ptr = (chunk.as_mut_ptr() as *mut T).add(local_idx);
            &mut *ptr
        }
    }

    // returns the number of chunks currently allocated.
    #[inline]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

impl<T: Copy, const N: usize> PicoList<T, N> {
    // pushes a slice of items into the list, spanning multiple chunks if necessary.
    pub fn extend_from_slice(&mut self, data: &[T]) {
        let mut remaining = data;

        while !remaining.is_empty() {
            let (chunk_idx, local_idx) = if Self::IS_POW2 {
                (self.len >> Self::SHIFT, self.len & Self::MASK)
            } else {
                (self.len / N, self.len % N)
            };

            if chunk_idx >= self.chunks.len() {
                self.chunks
                    .push(PicoRing::with_capacity(N + 1).expect("failed to allocate chunk"));
            }

            let current_chunk = &mut self.chunks[chunk_idx];
            let space_left = N - local_idx;

            let to_write = remaining.len().min(space_left);
            current_chunk.push_slice(&remaining[..to_write]);

            remaining = &remaining[to_write..];
            self.len += to_write;
        }
    }
}

impl<T, const N: usize> Default for PicoList<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
