/// Basler Pylon camera driver.
///
/// Supports Basler cameras (GigE, USB3, CoaXPress) via FFI bindings.
/// Production version would use pypylon or direct Pylon C API.
/// This version demonstrates the trait implementation and feature registry pattern.

use crate::camera::{Camera, CameraError};
use crate::device::{DeviceInfo, ControllerType};
use crate::feature_registry::{FeatureRegistry, FeatureDescriptor, FeatureValue, FeatureDescriptorBuilder};
use crate::frame::Frame;
use crate::Result;
use std::time::{SystemTime, UNIX_EPOCH};

/// Basler camera implementation
pub struct BaslerCamera {
    device_info: DeviceInfo,
    is_open: bool,
    exposure_us: f64,
    gain_db: f64,
    frame_counter: u64,
    features: FeatureRegistry,
}

impl BaslerCamera {
    /// Create a new Basler camera from device info.
    pub fn new(device_info: DeviceInfo) -> Self {
        let features = FeatureRegistry::new();

        // Register standard Basler features
        features.register(
            FeatureDescriptorBuilder::new(
                "ExposureTime".to_string(),
                FeatureValue::Float(1000.0),
            )
            .description("Exposure time in microseconds".to_string())
            .min(10.0)
            .max(10_000_000.0)
            .writable(true)
            .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new("Gain".to_string(), FeatureValue::Float(0.0))
                .description("Gain in dB".to_string())
                .min(0.0)
                .max(48.0)
                .writable(true)
                .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new(
                "PixelFormat".to_string(),
                FeatureValue::Enum("Mono8".to_string()),
            )
            .description("Pixel format".to_string())
            .enum_values(vec![
                "Mono8".to_string(),
                "Mono12".to_string(),
                "RGB8".to_string(),
            ])
            .writable(true)
            .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new(
                "Width".to_string(),
                FeatureValue::Integer(2048),
            )
            .description("Image width in pixels".to_string())
            .readable(true)
            .writable(false)
            .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new(
                "Height".to_string(),
                FeatureValue::Integer(2048),
            )
            .description("Image height in pixels".to_string())
            .readable(true)
            .writable(false)
            .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new(
                "SerialNumber".to_string(),
                FeatureValue::String(device_info.serial_number.clone()),
            )
            .readable(true)
            .writable(false)
            .build(),
        );

        BaslerCamera {
            device_info,
            is_open: false,
            exposure_us: 1000.0,
            gain_db: 0.0,
            frame_counter: 0,
            features,
        }
    }

    /// Get the feature registry.
    pub fn feature_registry(&self) -> &FeatureRegistry {
        &self.features
    }

    /// Get device info.
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }
}

impl Camera for BaslerCamera {
    fn open(&mut self) -> Result<()> {
        if self.is_open {
            return Err(CameraError::AlreadyOpen.into());
        }
        // In real implementation, this would initialize Pylon API
        self.is_open = true;
        self.frame_counter = 0;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }
        // In real implementation, this would cleanup Pylon resources
        self.is_open = false;
        Ok(())
    }

    fn grab_frame(&mut self) -> Result<Frame> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }

        let width = 2048u32;
        let height = 2048u32;
        let channels = 1u8; // Mono

        // Simulate frame capture
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        self.frame_counter += 1;

        // Create minimal dummy frame data
        let data = vec![128u8; (width * height * channels as u32) as usize];

        Ok(Frame {
            timestamp,
            sequence: self.frame_counter,
            width,
            height,
            channels,
            exposure_us: self.exposure_us as f32,
            gain: self.gain_db as f32,
            data,
        })
    }

    fn set_exposure(&mut self, us: f32) -> Result<()> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }
        // Validate against feature constraints
        self.features
            .set_value("ExposureTime", FeatureValue::Float(us as f64))?;
        self.exposure_us = us as f64;
        Ok(())
    }

    fn get_exposure(&self) -> Result<f32> {
        Ok(self.exposure_us as f32)
    }

    fn set_gain(&mut self, db: f32) -> Result<()> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }
        // Validate against feature constraints
        self.features
            .set_value("Gain", FeatureValue::Float(db as f64))?;
        self.gain_db = db as f64;
        Ok(())
    }

    fn get_gain(&self) -> Result<f32> {
        Ok(self.gain_db as f32)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn info(&self) -> crate::camera::CameraInfo {
        crate::camera::CameraInfo {
            serial: self.device_info.serial_number.clone(),
            vendor: "Basler".to_string(),
            model: self.device_info.model_name.clone(),
            width: 2048,
            height: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basler_creation() {
        let device = DeviceInfo::new(
            "basler_001".to_string(),
            "Basler ace2 Pro".to_string(),
            "BASLER_SN001".to_string(),
            ControllerType::Basler,
        );

        let cam = BaslerCamera::new(device);
        assert!(!cam.is_open());
        assert_eq!(cam.get_exposure().unwrap(), 1000.0);
        assert_eq!(cam.get_gain().unwrap(), 0.0);
    }

    #[test]
    fn test_basler_open_close() {
        let device = DeviceInfo::new(
            "basler_002".to_string(),
            "Basler ace2".to_string(),
            "BASLER_SN002".to_string(),
            ControllerType::Basler,
        );

        let mut cam = BaslerCamera::new(device);

        assert!(cam.open().is_ok());
        assert!(cam.is_open());

        assert!(cam.close().is_ok());
        assert!(!cam.is_open());
    }

    #[test]
    fn test_basler_grab_frame() {
        let device = DeviceInfo::new(
            "basler_003".to_string(),
            "Basler ace2".to_string(),
            "BASLER_SN003".to_string(),
            ControllerType::Basler,
        );

        let mut cam = BaslerCamera::new(device);
        cam.open().unwrap();

        let frame1 = cam.grab_frame().unwrap();
        assert_eq!(frame1.sequence, 1);
        assert_eq!(frame1.width, 2048);
        assert_eq!(frame1.height, 2048);

        let frame2 = cam.grab_frame().unwrap();
        assert_eq!(frame2.sequence, 2);
    }

    #[test]
    fn test_basler_exposure_control() {
        let device = DeviceInfo::new(
            "basler_004".to_string(),
            "Basler ace2".to_string(),
            "BASLER_SN004".to_string(),
            ControllerType::Basler,
        );

        let mut cam = BaslerCamera::new(device);
        cam.open().unwrap();

        // Valid exposure
        assert!(cam.set_exposure(5000.0).is_ok());
        assert_eq!(cam.get_exposure().unwrap(), 5000.0);

        // Out of range exposure
        assert!(cam.set_exposure(50_000_000.0).is_err());
    }

    #[test]
    fn test_basler_gain_control() {
        let device = DeviceInfo::new(
            "basler_005".to_string(),
            "Basler ace2".to_string(),
            "BASLER_SN005".to_string(),
            ControllerType::Basler,
        );

        let mut cam = BaslerCamera::new(device);
        cam.open().unwrap();

        // Valid gain
        assert!(cam.set_gain(24.0).is_ok());
        assert_eq!(cam.get_gain().unwrap(), 24.0);

        // Out of range gain
        assert!(cam.set_gain(100.0).is_err());
    }

    #[test]
    fn test_basler_features() {
        let device = DeviceInfo::new(
            "basler_006".to_string(),
            "Basler ace2".to_string(),
            "BASLER_SN006".to_string(),
            ControllerType::Basler,
        );

        let cam = BaslerCamera::new(device);
        let registry = cam.feature_registry();

        // Check registered features
        let features = registry.list();
        assert!(features.contains(&"ExposureTime".to_string()));
        assert!(features.contains(&"Gain".to_string()));
        assert!(features.contains(&"PixelFormat".to_string()));
        assert!(features.contains(&"SerialNumber".to_string()));
    }

    #[test]
    fn test_basler_camera_not_open() {
        let device = DeviceInfo::new(
            "basler_007".to_string(),
            "Basler ace2".to_string(),
            "BASLER_SN007".to_string(),
            ControllerType::Basler,
        );

        let mut cam = BaslerCamera::new(device);

        // Operations should fail when not open
        assert!(cam.grab_frame().is_err());
        assert!(cam.set_exposure(1000.0).is_err());
        assert!(cam.set_gain(10.0).is_err());
    }

    #[test]
    fn test_basler_info() {
        let device = DeviceInfo::new(
            "basler_008".to_string(),
            "Basler ace2 Pro".to_string(),
            "BASLER_SN008".to_string(),
            ControllerType::Basler,
        );

        let cam = BaslerCamera::new(device);
        let info = cam.info();
        assert_eq!(info.serial, "BASLER_SN008");
        assert_eq!(info.model, "Basler ace2 Pro");
        assert_eq!(info.vendor, "Basler");
    }
}
