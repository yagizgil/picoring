pub mod buffer;
pub mod collections;
pub mod ring;
pub mod system;

pub use collections::{PicoByteStream, PicoQueue};

pub use buffer::MirrorBuffer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut buf = MirrorBuffer::new(4096).unwrap();
        let slice = buf.as_mut_slice();
        slice[0] = 42;
        assert_eq!(slice[4096], 42);
    }

    #[test]
    fn test_alignment() {
        assert_eq!(buffer::align_to_page(0), 0);
        assert_eq!(buffer::align_to_page(1), 4096);
        assert_eq!(buffer::align_to_page(4095), 4096);
        assert_eq!(buffer::align_to_page(4096), 4096);
        assert_eq!(buffer::align_to_page(4097), 8192);
        assert_eq!(buffer::align_to_page(5000), 8192);
    }
}
