pub mod buffer;
pub mod collections;
pub mod ring;
pub mod spsc;
pub mod system;

#[cfg(feature = "python")]
pub mod python;

pub use buffer::MirrorBuffer;
pub use collections::{PicoByteStream, PicoList, PicoQueue};
pub use ring::PicoRing;
pub use spsc::{create_spsc, PicoConsumer, PicoProducer, PicoSPSC};

#[cfg(feature = "python")]
#[pyo3::prelude::pymodule]
fn picoring(m: &pyo3::prelude::Bound<'_, pyo3::prelude::PyModule>) -> pyo3::prelude::PyResult<()> {
    python::init_module(m)
}

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
        let ps = system::get_page_size();
        assert_eq!(buffer::align_to_page(0), 0);
        assert_eq!(buffer::align_to_page(1), ps);
        assert_eq!(buffer::align_to_page(ps - 1), ps);
        assert_eq!(buffer::align_to_page(ps), ps);
        assert_eq!(buffer::align_to_page(ps + 1), 2 * ps);
        assert_eq!(buffer::align_to_page(2 * ps - 1), 2 * ps);
    }
}
