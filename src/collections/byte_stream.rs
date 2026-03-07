use crate::ring::PicoRing;
use std::io::{self, Read, Write};

// implements Read and Write traits for seamless integration with std::io
pub struct PicoByteStream {
    ring: PicoRing<u8>,
}

impl PicoByteStream {
    // create a new byte stream with specified capacity
    pub fn new(capacity: usize) -> Result<Self, String> {
        Ok(Self {
            ring: PicoRing::new(capacity)?,
        })
    }

    // helper to get how many bytes are available to read
    pub fn available_to_read(&self) -> usize {
        self.ring.len()
    }

    // helper to get how much space is left to write
    pub fn available_to_write(&self) -> usize {
        self.ring.available_space()
    }
}

impl Read for PicoByteStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.available_to_read().min(buf.len());

        if n == 0 {
            return Ok(0);
        }

        unsafe {
            let src = self.ring.as_mut_ptr().add(self.ring.tail());
            core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), n);
        }

        self.ring.advance_tail(n);
        Ok(n)
    }
}

impl Write for PicoByteStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.available_to_write().min(buf.len());

        if n == 0 {
            return Ok(0);
        }

        unsafe {
            let dest = self.ring.as_mut_ptr().add(self.ring.head());
            core::ptr::copy_nonoverlapping(buf.as_ptr(), dest, n);
        }

        self.ring.advance_head(n);
        Ok(n)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Extra methods for zero-copy I/O operations (e.g., with TCP sockets)
impl PicoByteStream {
    // returns a direct slice of data to be read from
    pub fn as_read_slice(&self) -> &[u8] {
        self.ring.readable_slice()
    }

    // returns a direct mutable slice where data can be written to
    pub fn as_write_slice(&mut self) -> &mut [u8] {
        self.ring.writable_slice()
    }

    // manually advance markers after using as_read_slice or as_write_slice
    pub fn consume(&mut self, n: usize) {
        self.ring.advance_tail(n);
    }

    pub fn produce(&mut self, n: usize) {
        self.ring.advance_head(n);
    }
}
