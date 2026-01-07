use crate::frame::Frame;
use crate::Result;
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("Camera not open")]
    NotOpen,
    #[error("Camera already open")]
    AlreadyOpen,
    #[error("Frame grab failed: {0}")]
    FrameGrabFailed(String),
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Camera trait for unified interface across different camera implementations
///
/// # Examples
///
/// ```ignore
/// use optik::camera::Camera;
///
/// let mut camera = create_camera();
/// camera.open().expect("Failed to open camera");
/// let frame = camera.grab_frame().expect("Failed to grab frame");
/// camera.set_exposure(5000.0).expect("Failed to set exposure");
/// camera.close().expect("Failed to close camera");
/// ```
pub trait Camera: Send + Sync {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn grab_frame(&mut self) -> Result<Frame>;
    fn set_exposure(&mut self, exposure_us: f32) -> Result<()>;
    fn get_exposure(&self) -> Result<f32>;
    fn set_gain(&mut self, gain: f32) -> Result<()>;
    fn get_gain(&self) -> Result<f32>;
    fn is_open(&self) -> bool;
    fn info(&self) -> CameraInfo;
}

/// Information about a camera device
///
/// # Fields
///
/// * `serial` - Serial number of the camera
/// * `vendor` - Vendor/manufacturer name
/// * `model` - Model identifier
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub serial: String,
    pub vendor: String,
    pub model: String,
    pub width: u32,
    pub height: u32,
}

/// Raspberry Pi Camera implementation
pub struct RpiCamera {
    serial: String,
    index: u32,
    is_open: bool,
    exposure_us: f32,
    gain: f32,
    frame_counter: u64,
}

impl RpiCamera {
    pub fn new(index: u32) -> Self {
        RpiCamera {
            serial: format!("rpi_camera_{}", index),
            index,
            is_open: false,
            exposure_us: 10000.0,
            gain: 0.0,
            frame_counter: 0,
        }
    }

    fn timestamp_us() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}

impl Camera for RpiCamera {
    fn open(&mut self) -> Result<()> {
        if self.is_open {
            return Err(CameraError::AlreadyOpen.into());
        }
        
        // In real implementation, would use libcamera/picamera2 via FFI
        // For now, just mark as open for testing
        self.is_open = true;
        tracing::info!("Opened RPi camera: {}", self.serial);
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if !self.is_open {
            return Ok(());
        }
        self.is_open = false;
        tracing::info!("Closed RPi camera: {}", self.serial);
        Ok(())
    }

    fn grab_frame(&mut self) -> Result<Frame> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }

        // Simulate frame grab
        let width = 4056;
        let height = 3040;
        let channels = 3;
        let size = (width * height * channels as u32) as usize;
        let data = vec![0u8; size];

        self.frame_counter += 1;

        Ok(Frame {
            timestamp: Self::timestamp_us(),
            sequence: self.frame_counter,
            width,
            height,
            channels,
            exposure_us: self.exposure_us,
            gain: self.gain,
            data,
        })
    }

    fn set_exposure(&mut self, exposure_us: f32) -> Result<()> {
        if exposure_us < 100.0 || exposure_us > 1000000.0 {
            return Err(CameraError::ConfigError("Exposure out of range".to_string()).into());
        }
        self.exposure_us = exposure_us;
        Ok(())
    }

    fn get_exposure(&self) -> Result<f32> {
        Ok(self.exposure_us)
    }

    fn set_gain(&mut self, gain: f32) -> Result<()> {
        if gain < 0.0 || gain > 48.0 {
            return Err(CameraError::ConfigError("Gain out of range".to_string()).into());
        }
        self.gain = gain;
        Ok(())
    }

    fn get_gain(&self) -> Result<f32> {
        Ok(self.gain)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn info(&self) -> CameraInfo {
        CameraInfo {
            serial: self.serial.clone(),
            vendor: "Raspberry Pi".to_string(),
            model: "HQ Camera".to_string(),
            width: 4056,
            height: 3040,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpi_camera_creation() {
        let cam = RpiCamera::new(0);
        assert_eq!(cam.serial, "rpi_camera_0");
        assert!(!cam.is_open);
    }

    #[test]
    fn test_rpi_camera_open_close() {
        let mut cam = RpiCamera::new(0);
        assert!(cam.open().is_ok());
        assert!(cam.is_open());
        assert!(cam.close().is_ok());
        assert!(!cam.is_open());
    }

    #[test]
    fn test_rpi_camera_exposure() {
        let mut cam = RpiCamera::new(0);
        assert!(cam.open().is_ok());
        
        assert!(cam.set_exposure(15000.0).is_ok());
        assert_eq!(cam.get_exposure().unwrap(), 15000.0);
        
        // Invalid exposure
        assert!(cam.set_exposure(0.0).is_err());
    }

    #[test]
    fn test_rpi_camera_gain() {
        let mut cam = RpiCamera::new(0);
        assert!(cam.open().is_ok());
        
        assert!(cam.set_gain(10.0).is_ok());
        assert_eq!(cam.get_gain().unwrap(), 10.0);
        
        // Invalid gain
        assert!(cam.set_gain(100.0).is_err());
    }

    #[test]
    fn test_rpi_camera_frame_grab() {
        let mut cam = RpiCamera::new(0);
        assert!(cam.open().is_ok());
        
        let frame = cam.grab_frame();
        assert!(frame.is_ok());
        
        let f = frame.unwrap();
        assert_eq!(f.sequence, 1);
        assert_eq!(f.width, 4056);
        assert_eq!(f.height, 3040);
    }
}
