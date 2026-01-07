use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use ndarray::Array3;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod camera;
pub mod frame;
pub mod gige;
pub mod multi_camera;
pub mod lock_utils;
pub mod shmem;

use camera::{Camera, CameraError};
use frame::Frame;

#[derive(Error, Debug)]
pub enum OptikError {
    #[error("Camera error: {0}")]
    CameraError(#[from] CameraError),
    #[error("Frame error: {0}")]
    FrameError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lock error: {0}")]
    LockError(String),
    #[error("Lock timeout: {0}")]
    LockTimeout(String),
    #[error("Frame queue error: {0}")]
    QueueError(String),
    #[error("Shared memory error: {0}")]
    ShmemError(String),
}

pub type Result<T> = std::result::Result<T, OptikError>;

/// Rust core module for optik
#[pymodule]
fn _core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", "0.1.0")?;
    
    // Core classes
    m.add_class::<PyCamera>()?;
    m.add_class::<PyFrame>()?;
    m.add_class::<PyFrameBuffer>()?;
    m.add_class::<PyFrameMetadata>()?;
    
    Ok(())
}

/// Python wrapper for Camera
#[pyclass]
pub struct PyCamera {
    inner: Arc<Mutex<Box<dyn Camera>>>,
}

#[pymethods]
impl PyCamera {
    #[new]
    fn new(camera_type: &str, index: u32) -> PyResult<Self> {
        let cam: Box<dyn Camera> = match camera_type {
            "rpi" => Box::new(camera::RpiCamera::new(index)),
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Unknown camera type: {}", camera_type),
            )),
        };

        Ok(PyCamera {
            inner: Arc::new(Mutex::new(cam)),
        })
    }

    fn open(&self) -> PyResult<()> {
        self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .open()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn close(&self) -> PyResult<()> {
        self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .close()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn grab_frame(&self) -> PyResult<PyFrame> {
        let frame = self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .grab_frame()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyFrame {
            inner: Arc::new(frame),
        })
    }

    fn set_exposure(&self, exposure_us: f32) -> PyResult<()> {
        self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .set_exposure(exposure_us)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn get_exposure(&self) -> PyResult<f32> {
        self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .get_exposure()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn set_gain(&self, gain: f32) -> PyResult<()> {
        self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .set_gain(gain)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn get_gain(&self) -> PyResult<f32> {
        self.inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to acquire lock: {}", e)
            ))?
            .get_gain()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn is_open(&self) -> bool {
        self.inner.lock().unwrap().is_open()
    }
}

/// Python wrapper for Frame
#[pyclass]
pub struct PyFrame {
    inner: Arc<Frame>,
}

#[pymethods]
impl PyFrame {
    fn width(&self) -> u32 {
        self.inner.width
    }

    fn height(&self) -> u32 {
        self.inner.height
    }

    fn channels(&self) -> u8 {
        self.inner.channels
    }

    fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    fn sequence(&self) -> u64 {
        self.inner.sequence
    }

    fn data(&self) -> Vec<u8> {
        self.inner.data.clone()
    }

    fn as_numpy(&self, _py: Python) -> PyResult<PyObject> {
        // Return None for now - numpy conversion requires proper FFI
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "numpy conversion: use frame.metadata() and frame.data() separately"
        ))
    }

    fn metadata(&self) -> PyFrameMetadata {
        PyFrameMetadata {
            timestamp: self.inner.timestamp,
            sequence: self.inner.sequence,
            exposure_us: self.inner.exposure_us,
            gain: self.inner.gain,
        }
    }
}

/// Python wrapper for FrameMetadata
#[pyclass]
#[derive(Clone)]
pub struct PyFrameMetadata {
    #[pyo3(get)]
    pub timestamp: u64,
    #[pyo3(get)]
    pub sequence: u64,
    #[pyo3(get)]
    pub exposure_us: f32,
    #[pyo3(get)]
    pub gain: f32,
}

#[pymethods]
impl PyFrameMetadata {
    #[new]
    fn new(timestamp: u64, sequence: u64, exposure_us: f32, gain: f32) -> Self {
        PyFrameMetadata {
            timestamp,
            sequence,
            exposure_us,
            gain,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "FrameMetadata(ts={}, seq={}, exp={:.0}µs, gain={:.1}dB)",
            self.timestamp, self.sequence, self.exposure_us, self.gain
        )
    }
}

/// Optimized frame buffer for camera frames
#[pyclass]
pub struct PyFrameBuffer {
    #[pyo3(get)]
    width: u32,
    #[pyo3(get)]
    height: u32,
    #[pyo3(get)]
    channels: u8,
    data: Vec<u8>,
}

#[pymethods]
impl PyFrameBuffer {
    #[new]
    fn new(width: u32, height: u32, channels: u8) -> Self {
        let size = (width * height * channels as u32) as usize;
        PyFrameBuffer {
            width,
            height,
            channels,
            data: vec![0u8; size],
        }
    }

    #[getter]
    fn size(&self) -> usize {
        self.data.len()
    }

    fn as_numpy(&self, _py: Python) -> PyResult<PyObject> {
        // Return None for now - numpy conversion will be implemented with proper FFI
        // For production, use: PyBytes::new(_py, &self.data[..]).into()
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "numpy conversion requires numpy to be installed. Use frame.data bytes directly."
        ))
    }

    fn clear(&mut self) {
        self.data.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_metadata() {
        let meta = PyFrameMetadata::new(1000, 1, 10000.0, 5.0);
        assert_eq!(meta.timestamp, 1000);
        assert_eq!(meta.sequence, 1);
    }

    #[test]
    fn test_frame_buffer() {
        let buf = PyFrameBuffer::new(640, 480, 3);
        assert_eq!(buf.width, 640);
        assert_eq!(buf.height, 480);
        assert_eq!(buf.size(), 640 * 480 * 3);
    }
}
