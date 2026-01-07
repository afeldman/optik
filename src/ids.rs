/// IDS Ensenso camera driver.
///
/// Supports IDS cameras (USB3, Ethernet) via FFI bindings.
/// Production version would use ids_peak SDK or direct C API.
/// This version demonstrates the trait implementation and feature registry pattern.

use crate::camera::{Camera, CameraError};
use crate::device::{DeviceInfo, ControllerType};
use crate::feature_registry::{FeatureRegistry, FeatureDescriptor, FeatureValue, FeatureDescriptorBuilder};
use crate::frame::Frame;
use crate::Result;
use std::time::{SystemTime, UNIX_EPOCH};

/// IDS camera implementation
pub struct IDSCamera {
    device_info: DeviceInfo,
    is_open: bool,
    exposure_us: f64,
    gain_db: f64,
    frame_counter: u64,
    features: FeatureRegistry,
}

impl IDSCamera {
    /// Create a new IDS camera from device info.
    pub fn new(device_info: DeviceInfo) -> Self {
        let features = FeatureRegistry::new();

        // Register standard IDS features
        features.register(
            FeatureDescriptorBuilder::new(
                "ExposureTime".to_string(),
                FeatureValue::Float(5000.0),
            )
            .description("Exposure time in microseconds".to_string())
            .min(5.0)
            .max(30_000_000.0)
            .writable(true)
            .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new("Gain".to_string(), FeatureValue::Float(1.0))
                .description("Gain in dB".to_string())
                .min(0.0)
                .max(96.0) // IDS allows higher gain
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
                "RGB8".to_string(),
                "BGR8".to_string(),
            ])
            .writable(true)
            .build(),
        );

        features.register(
            FeatureDescriptorBuilder::new(
                "Width".to_string(),
                FeatureValue::Integer(2560),
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

        features.register(
            FeatureDescriptorBuilder::new(
                "TriggerMode".to_string(),
                FeatureValue::Enum("Off".to_string()),
            )
            .description("Trigger mode".to_string())
            .enum_values(vec!["Off".to_string(), "On".to_string()])
            .writable(true)
            .build(),
        );

        IDSCamera {
            device_info,
            is_open: false,
            exposure_us: 5000.0,
            gain_db: 1.0,
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

impl Camera for IDSCamera {
    fn open(&mut self) -> Result<()> {
        if self.is_open {
            return Err(CameraError::AlreadyOpen.into());
        }
        // In real implementation, this would initialize IDS SDK
        self.is_open = true;
        self.frame_counter = 0;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }
        // In real implementation, this would cleanup IDS SDK resources
        self.is_open = false;
        Ok(())
    }

    fn grab_frame(&mut self) -> Result<Frame> {
        if !self.is_open {
            return Err(CameraError::NotOpen.into());
        }

        let width = 2560u32;
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
            vendor: "IDS".to_string(),
            model: self.device_info.model_name.clone(),
            width: 2560,
            height: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_creation() {
        let device = DeviceInfo::new(
            "ids_001".to_string(),
            "IDS Ensenso N35".to_string(),
            "IDS_SN001".to_string(),
            ControllerType::IDS,
        );

        let cam = IDSCamera::new(device);
        assert!(!cam.is_open());
        assert_eq!(cam.get_exposure().unwrap(), 5000.0);
        assert_eq!(cam.get_gain().unwrap(), 1.0);
    }

    #[test]
    fn test_ids_open_close() {
        let device = DeviceInfo::new(
            "ids_002".to_string(),
            "IDS Ensenso".to_string(),
            "IDS_SN002".to_string(),
            ControllerType::IDS,
        );

        let mut cam = IDSCamera::new(device);

        assert!(cam.open().is_ok());
        assert!(cam.is_open());

        assert!(cam.close().is_ok());
        assert!(!cam.is_open());
    }

    #[test]
    fn test_ids_grab_frame() {
        let device = DeviceInfo::new(
            "ids_003".to_string(),
            "IDS Ensenso".to_string(),
            "IDS_SN003".to_string(),
            ControllerType::IDS,
        );

        let mut cam = IDSCamera::new(device);
        cam.open().unwrap();

        let frame1 = cam.grab_frame().unwrap();
        assert_eq!(frame1.sequence, 1);
        assert_eq!(frame1.width, 2560);
        assert_eq!(frame1.height, 2048);

        let frame2 = cam.grab_frame().unwrap();
        assert_eq!(frame2.sequence, 2);
    }

    #[test]
    fn test_ids_exposure_control() {
        let device = DeviceInfo::new(
            "ids_004".to_string(),
            "IDS Ensenso".to_string(),
            "IDS_SN004".to_string(),
            ControllerType::IDS,
        );

        let mut cam = IDSCamera::new(device);
        cam.open().unwrap();

        // Valid exposure
        assert!(cam.set_exposure(10000.0).is_ok());
        assert_eq!(cam.get_exposure().unwrap(), 10000.0);

        // Out of range exposure (too high for IDS)
        assert!(cam.set_exposure(100_000_000.0).is_err());
    }

    #[test]
    fn test_ids_gain_control() {
        let device = DeviceInfo::new(
            "ids_005".to_string(),
            "IDS Ensenso".to_string(),
            "IDS_SN005".to_string(),
            ControllerType::IDS,
        );

        let mut cam = IDSCamera::new(device);
        cam.open().unwrap();

        // IDS allows higher gain than Basler
        assert!(cam.set_gain(48.0).is_ok());
        assert_eq!(cam.get_gain().unwrap(), 48.0);

        // Out of range gain
        assert!(cam.set_gain(200.0).is_err());
    }

    #[test]
    fn test_ids_features() {
        let device = DeviceInfo::new(
            "ids_006".to_string(),
            "IDS Ensenso".to_string(),
            "IDS_SN006".to_string(),
            ControllerType::IDS,
        );

        let cam = IDSCamera::new(device);
        let registry = cam.feature_registry();

        // Check registered features
        let features = registry.list();
        assert!(features.contains(&"ExposureTime".to_string()));
        assert!(features.contains(&"Gain".to_string()));
        assert!(features.contains(&"TriggerMode".to_string()));
    }

    #[test]
    fn test_ids_camera_not_open() {
        let device = DeviceInfo::new(
            "ids_007".to_string(),
            "IDS Ensenso".to_string(),
            "IDS_SN007".to_string(),
            ControllerType::IDS,
        );

        let mut cam = IDSCamera::new(device);

        // Operations should fail when not open
        assert!(cam.grab_frame().is_err());
        assert!(cam.set_exposure(1000.0).is_err());
        assert!(cam.set_gain(10.0).is_err());
    }

    #[test]
    fn test_ids_info() {
        let device = DeviceInfo::new(
            "ids_008".to_string(),
            "IDS Ensenso N35".to_string(),
            "IDS_SN008".to_string(),
            ControllerType::IDS,
        );

        let cam = IDSCamera::new(device);
        let info = cam.info();
        assert_eq!(info.serial, "IDS_SN008");
        assert_eq!(info.model, "IDS Ensenso N35");
        assert_eq!(info.vendor, "IDS");
    }
}
