use crate::buffer::MirrorBuffer;
use core::marker::PhantomData;
use core::{ptr, slice};

pub struct PicoRing<T> {
    buffer: MirrorBuffer,
    head: usize,     // write position (measured in items of type T)
    tail: usize,     // read position
    capacity: usize, // how many items of type T fit (physical capacity)
    _marker: PhantomData<T>,
}

impl<T> PicoRing<T> {
    pub fn new(capacity_count: usize) -> Result<Self, String> {
        // calculate the size of T and create a MirrorBuffer aligned to page boundaries
        let item_size = core::mem::size_of::<T>();

        // protect against Zero Sized Types (ZST) to avoid division by zero
        if item_size == 0 {
            return Err("Zero sized types are not supported in PicoRing".into());
        }

        let total_bytes = capacity_count
            .checked_mul(item_size)
            .ok_or_else(|| "Requested capacity is too large (overflow)".to_string())?;

        let buffer = MirrorBuffer::new(total_bytes)?;
        // update capacity based on the actual physical size (which may have been rounded up)
        let actual_capacity = buffer.size() / item_size;

        Ok(Self {
            buffer,
            head: 0,
            tail: 0,
            capacity: actual_capacity,
            _marker: PhantomData,
        })
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
        self.head = (self.head + 1) % self.capacity;
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

        self.tail = (self.tail + 1) % self.capacity;
        Option::Some(item)
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        ((self.head + 1) % self.capacity) == self.tail
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
    pub fn len(&self) -> usize {
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            self.capacity - (self.tail - self.head)
        }
    }

    // returns the physical capacity of the ring buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // --- High Level Slice APIs ---

    // returns a contiguous slice of all readable data
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
    pub fn advance_head(&mut self, n: usize) {
        self.head = (self.head + n) % self.capacity;
    }

    // manually advance the read pointer
    pub fn advance_tail(&mut self, n: usize) {
        self.tail = (self.tail + n) % self.capacity;
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
impl<T: Copy> PicoRing<T> {
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

        self.head = (self.head + n) % self.capacity;
        true
    }
}

// Support converting from standard Vec
impl<T: Copy> From<Vec<T>> for PicoRing<T> {
    fn from(v: Vec<T>) -> Self {
        let mut ring = PicoRing::new(v.len() + 1).expect("Failed to convert Vec to PicoRing");
        ring.push_slice(&v);
        ring
    }
}
