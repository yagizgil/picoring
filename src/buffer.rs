use crate::system;

// A circular buffer that uses memory mirroring for contiguous access
pub struct MirrorBuffer {
    // pointer to the start of the mirrored virtual memory
    ptr: *mut u8,
    // the size of one physical memory block (the buffer repeats after this)
    size: usize,
}

impl MirrorBuffer {
    // create a new mirrored buffer
    pub fn new(mut size: usize) -> Result<Self, String> {
        size = align_to_page(size);

        // allocate the mirrored memory through platform-specific system calls
        let ptr = unsafe { system::allocate_mirror(size)? };
        Ok(Self { ptr, size })
    }

    // return the physical size of the buffer (half of the virtual space)
    pub fn size(&self) -> usize {
        self.size
    }

    // provide a mutable slice to the ENTIRE mirrored virtual range (2 * size)
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, 2 * self.size) }
    }

    // provide a read-only slice to the ENTIRE mirrored virtual range (2 * size)
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, 2 * self.size) }
    }

    /// Returns the raw base pointer.
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for MirrorBuffer {
    fn drop(&mut self) {
        unsafe {
            system::deallocate_mirror(self.ptr, self.size);
        }
    }
}

// align a size to the nearest page boundary
pub fn align_to_page(size: usize) -> usize {
    let page_size = system::get_page_size();
    (size + page_size - 1) & !(page_size - 1)
}
