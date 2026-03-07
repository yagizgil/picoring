use crate::buffer::MirrorBuffer;
use core::marker::PhantomData;
use core::{ptr, slice};

pub struct PicoRing<T, const N: usize = 0> {
    buffer: MirrorBuffer,
    head: usize,     // write position (measured in items of type T)
    tail: usize,     // read position
    capacity: usize, // how many items of type T fit (physical capacity)
    mask: usize,     // capacity - 1 if it's a power of two, otherwise 0
    _marker: PhantomData<T>,
}

impl<T, const N: usize> PicoRing<T, N> {
    // creates a new PicoRing with a capacity determined by the const generic N.
    // returns an error if N is 0 (use with_capacity instead).
    pub fn new() -> Result<Self, String> {
        if N == 0 {
            return Err(
                "Cannot use new() without a const generic size. Use with_capacity(size) instead."
                    .into(),
            );
        }
        Self::create(N)
    }

    // internal helper
    fn create(capacity_count: usize) -> Result<Self, String> {
        let item_size = core::mem::size_of::<T>();
        if item_size == 0 {
            return Err("Zero sized types are not supported in PicoRing".into());
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

        Ok(Self {
            buffer,
            head: 0,
            tail: 0,
            capacity: actual_capacity,
            mask,
            _marker: PhantomData,
        })
    }
}

impl<T> PicoRing<T, 0> {
    // creates a new PicoRing with a dynamic capacity.
    pub fn with_capacity(capacity_count: usize) -> Result<Self, String> {
        Self::create(capacity_count)
    }
}

impl<T, const N: usize> PicoRing<T, N> {
    #[inline(always)]
    fn wrap(&self, val: usize) -> usize {
        if self.mask != 0 {
            val & self.mask
        } else {
            val % self.capacity
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) -> bool {
        if self.is_full() {
            return false;
        }

        unsafe {
            // find the write address (it can exceed physical capacity in virtual space, no problem!)
            let ptr = (self.buffer.as_mut_ptr() as *mut T).add(self.head);
            ptr::write(ptr, item);
        }

        // update index and wrap around based on capacity
        self.head = self.wrap(self.head + 1);
        true
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = unsafe {
            let ptr = (self.buffer.as_mut_ptr() as *const T).add(self.tail);
            ptr::read(ptr)
        };

        self.tail = self.wrap(self.tail + 1);
        Option::Some(item)
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.wrap(self.head + 1) == self.tail
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    #[inline]
    pub fn head(&self) -> usize {
        self.head
    }

    #[inline]
    pub fn tail(&self) -> usize {
        self.tail
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.buffer.as_mut_ptr() as *mut T
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.capacity * 2)
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let ptr = self.buffer.as_slice().as_ptr() as *const T;
            slice::from_raw_parts(ptr, self.capacity * 2)
        }
    }

    // returns a contiguous slice from any position in the ring
    // returns None if the range exceeds the mirrored virtual memory space
    #[inline]
    pub fn view(&self, offset: usize, len: usize) -> Option<&[T]> {
        self.as_slice().get(offset..offset + len)
    }

    // mutable version of view
    #[inline]
    pub fn view_mut(&mut self, offset: usize, len: usize) -> Option<&mut [T]> {
        self.as_mut_slice().get_mut(offset..offset + len)
    }

    // returns how many items are currently in the ring buffer
    #[inline]
    pub fn len(&self) -> usize {
        self.wrap(self.head + self.capacity - self.tail)
    }

    // returns the physical capacity of the ring buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // --- High Level Slice APIs ---

    // returns a contiguous slice of all readable data
    #[inline]
    pub fn readable_slice(&self) -> &[T] {
        let len = self.len();
        self.view(self.tail, len).unwrap_or(&[])
    }

    // returns a contiguous slice of all available writable space
    pub fn writable_slice(&mut self) -> &mut [T] {
        let space = self.available_space();
        let head = self.head;
        self.view_mut(head, space).unwrap_or(&mut [])
    }

    // manually advance the write pointer
    #[inline]
    pub fn advance_head(&mut self, n: usize) {
        self.head = self.wrap(self.head + n);
    }

    // manually advance the read pointer
    #[inline]
    pub fn advance_tail(&mut self, n: usize) {
        self.tail = self.wrap(self.tail + n);
    }

    #[inline]
    pub fn available_space(&self) -> usize {
        if self.head >= self.tail {
            self.capacity - (self.head - self.tail) - 1
        } else {
            self.tail - self.head - 1
        }
    }
}
impl<T: Copy, const N: usize> PicoRing<T, N> {
    // pushes multiple items at once using hardware mirroring
    #[inline]
    pub fn push_slice(&mut self, data: &[T]) -> bool {
        let n = data.len();
        if self.available_space() < n {
            return false;
        }

        unsafe {
            // even if the data doesnt fit at the end, its a single copy
            let dest_ptr = self.as_mut_ptr().add(self.head);
            ptr::copy_nonoverlapping(data.as_ptr(), dest_ptr, n);
        }

        self.head = self.wrap(self.head + n);
        true
    }
}

// Support converting from standard Vec
impl<T: Copy> From<Vec<T>> for PicoRing<T> {
    fn from(v: Vec<T>) -> Self {
        let mut ring =
            PicoRing::with_capacity(v.len() + 1).expect("Failed to convert Vec to PicoRing");
        ring.push_slice(&v);
        ring
    }
}
