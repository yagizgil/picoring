use crate::collections::{PicoByteStream, PicoList, PicoQueue};
use crate::ring::PicoRing;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyMemoryView;

// --- PicoRingByte ---
#[pyclass]
pub struct PicoRingByte {
    inner: PicoRing<u8, 0>,
}

#[pymethods]
impl PicoRingByte {
    #[new]
    pub fn new(capacity: usize) -> PyResult<Self> {
        let inner = PicoRing::with_capacity(capacity).map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    pub fn push(&mut self, item: u8) -> bool {
        self.inner.push(item)
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.inner.pop()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn get_readable_view<'py>(slf: Bound<'py, Self>) -> PyResult<Bound<'py, PyMemoryView>> {
        PyMemoryView::from_bound(&slf)
    }

    unsafe fn __getbuffer__<'py>(
        slf: Bound<'py, Self>,
        view: *mut pyo3::ffi::Py_buffer,
        flags: std::os::raw::c_int,
    ) -> PyResult<()> {
        let ring = slf.borrow();
        let slice = ring.inner.readable_slice();
        let res = pyo3::ffi::PyBuffer_FillInfo(
            view,
            slf.as_ptr(),
            slice.as_ptr() as *mut _,
            slice.len() as isize,
            1, // readonly
            flags,
        );
        if res == 0 {
            Ok(())
        } else {
            Err(PyValueError::new_err("Buffer Fill Error"))
        }
    }

    unsafe fn __releasebuffer__(&self, _view: *mut pyo3::ffi::Py_buffer) {}

    pub fn push_bytes(&mut self, data: &[u8]) -> bool {
        self.inner.push_slice(data)
    }

    pub fn pop_bytes<'py>(
        &mut self,
        py: Python<'py>,
        n: usize,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let len = self.inner.len().min(n);
        let slice = self.inner.readable_slice();
        let result = pyo3::types::PyBytes::new_bound(py, &slice[..len]);
        self.inner.advance_tail(len);
        Ok(result)
    }

    pub fn advance_head(&mut self, n: usize) {
        self.inner.advance_head(n);
    }
    pub fn advance_tail(&mut self, n: usize) {
        self.inner.advance_tail(n);
    }
}

// --- PicoQueueByte ---
#[pyclass]
pub struct PicoQueueByte {
    inner: PicoQueue<u8, 0>,
}

#[pymethods]
impl PicoQueueByte {
    #[new]
    pub fn new(capacity: usize) -> PyResult<Self> {
        let inner = PicoQueue::new(capacity).map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn push(&mut self, item: u8) -> bool {
        self.inner.try_push(item)
    }
    pub fn pop(&mut self) -> Option<u8> {
        self.inner.try_pop()
    }

    pub fn push_bulk(&mut self, data: &[u8]) -> bool {
        if let Some(space) = self.inner.reserve(data.len()) {
            space.copy_from_slice(data);
            self.inner.commit(data.len());
            true
        } else {
            false
        }
    }

    pub fn pop_bulk<'py>(
        &mut self,
        py: Python<'py>,
        n: usize,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let count = self.inner.len().min(n);
        let data = pyo3::types::PyBytes::new_bound(py, &self.inner.peek()[..count]);
        self.inner.release(count);
        Ok(data)
    }

    pub fn get_view<'py>(slf: Bound<'py, Self>) -> PyResult<Bound<'py, PyMemoryView>> {
        PyMemoryView::from_bound(&slf)
    }

    unsafe fn __getbuffer__<'py>(
        slf: Bound<'py, Self>,
        view: *mut pyo3::ffi::Py_buffer,
        flags: std::os::raw::c_int,
    ) -> PyResult<()> {
        let q = slf.borrow();
        let slice = q.inner.peek();
        let res = pyo3::ffi::PyBuffer_FillInfo(
            view,
            slf.as_ptr(),
            slice.as_ptr() as *mut _,
            slice.len() as isize,
            1,
            flags,
        );
        if res == 0 {
            Ok(())
        } else {
            Err(PyValueError::new_err("Buffer Error"))
        }
    }
}

// --- PicoByteStream ---
#[pyclass]
pub struct PicoByteStreamPy {
    inner: PicoByteStream<0>,
}

#[pymethods]
impl PicoByteStreamPy {
    #[new]
    pub fn new(capacity: usize) -> PyResult<Self> {
        let inner = PicoByteStream::new(capacity).map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    pub fn write(&mut self, data: &[u8]) -> PyResult<usize> {
        use std::io::Write;
        self.inner
            .write(data)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn read<'py>(
        &mut self,
        py: Python<'py>,
        n: usize,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        use std::io::Read;
        let mut buf = vec![0u8; n];
        let bytes_read = self
            .inner
            .read(&mut buf)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(pyo3::types::PyBytes::new_bound(py, &buf[..bytes_read]))
    }

    pub fn available_to_read(&self) -> usize {
        self.inner.available_to_read()
    }
    pub fn available_to_write(&self) -> usize {
        self.inner.available_to_write()
    }
}

// --- PicoListByte --- (Simplified to u8 for performance)
#[pyclass]
pub struct PicoListByte {
    inner: PicoList<u8, 16384>,
}

#[pymethods]
impl PicoListByte {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: PicoList::new(),
        }
    }

    pub fn push(&mut self, item: u8) {
        self.inner.push(item);
    }
    pub fn get(&self, index: usize) -> Option<u8> {
        self.inner.get(index).copied()
    }
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        self.inner.set(index, value)
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn extend(&mut self, data: &[u8]) {
        self.inner.extend_from_slice(data);
    }
}

pub fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PicoRingByte>()?;
    m.add_class::<PicoQueueByte>()?;
    m.add_class::<PicoByteStreamPy>()?;
    m.add_class::<PicoListByte>()?;
    Ok(())
}
