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
    // uses unsafe for high-performance unchecked access.
    #[inline]
    pub fn push(&mut self, item: T) {
        let chunk_idx = self.len / N;
        let local_idx = self.len % N;

        // allocate a new chunk if needed
        if chunk_idx >= self.chunks.len() {
            // allocate N + 1 because PicoRing leaves one slot empty for its full check.
            self.chunks
                .push(PicoRing::with_capacity(N + 1).expect("failed to allocate chunk"));
        }

        unsafe {
            // bypass bounds check for getting the chunk
            let chunk = self.chunks.get_unchecked_mut(chunk_idx);

            // calculate the direct memory address and write the item
            // this skips any overhead of ring buffer checks
            let ptr = (chunk.as_mut_ptr() as *mut T).add(local_idx);
            ptr::write(ptr, item);

            // sync the ring's head so methods like as_slice() still work
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

        let chunk_idx = index / N;
        let local_idx = index % N;

        unsafe {
            // safety: index < self.len ensures chunk_idx is valid
            let chunk = self.chunks.get_unchecked(chunk_idx);

            // direct pointer access to the mirrored memory
            let ptr = (chunk.as_mut_ptr() as *const T).add(local_idx);
            Some(&*ptr)
        }
    }

    // returns a mutable reference to the item at the specified index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        let chunk_idx = index / N;
        let local_idx = index % N;

        unsafe {
            // safety: index < self.len ensures chunk_idx is valid
            let chunk = self.chunks.get_unchecked_mut(chunk_idx);

            // direct pointer access to the mirrored memory
            let ptr = (chunk.as_mut_ptr() as *mut T).add(local_idx);
            Some(&mut *ptr)
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
            let chunk_idx = self.len / N;
            let local_idx = self.len % N;

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
