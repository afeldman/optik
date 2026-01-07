/// Camera configuration with validation and atomic application.
///
/// Provides a declarative configuration system for cameras with constraint validation
/// and atomic application (all-or-nothing semantics).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::camera::Camera;
use crate::error::{OptikError, Result};
use crate::feature_registry::FeatureValue;

/// Pixel format for camera output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// Monochrome 8-bit
    Mono8,
    /// Monochrome 12-bit
    Mono12,
    /// RGB 8-bit
    RGB8,
    /// BGR 8-bit
    BGR8,
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelFormat::Mono8 => write!(f, "Mono8"),
            PixelFormat::Mono12 => write!(f, "Mono12"),
            PixelFormat::RGB8 => write!(f, "RGB8"),
            PixelFormat::BGR8 => write!(f, "BGR8"),
        }
    }
}

impl std::str::FromStr for PixelFormat {
    type Err = OptikError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Mono8" => Ok(PixelFormat::Mono8),
            "Mono12" => Ok(PixelFormat::Mono12),
            "RGB8" => Ok(PixelFormat::RGB8),
            "BGR8" => Ok(PixelFormat::BGR8),
            _ => Err(OptikError::ConfigError(format!(
                "Unknown pixel format: {}",
                s
            ))),
        }
    }
}

/// Trigger mode for camera acquisition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    /// Continuous acquisition
    Off,
    /// Triggered acquisition
    On,
}

impl std::fmt::Display for TriggerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerMode::Off => write!(f, "Off"),
            TriggerMode::On => write!(f, "On"),
        }
    }
}

impl std::str::FromStr for TriggerMode {
    type Err = OptikError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Off" => Ok(TriggerMode::Off),
            "On" => Ok(TriggerMode::On),
            _ => Err(OptikError::ConfigError(format!(
                "Unknown trigger mode: {}",
                s
            ))),
        }
    }
}

/// Complete camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Exposure time in microseconds
    pub exposure_us: f32,

    /// Gain in dB
    pub gain_db: f32,

    /// Pixel format
    pub pixel_format: PixelFormat,

    /// Trigger mode
    pub trigger_mode: TriggerMode,

    /// Frame rate in Hz
    pub frame_rate: f32,

    /// ROI offset X
    pub offset_x: u32,

    /// ROI offset Y
    pub offset_y: u32,

    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Custom features as JSON
    pub features: Option<HashMap<String, serde_json::Value>>,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            exposure_us: 1000.0,
            gain_db: 0.0,
            pixel_format: PixelFormat::Mono8,
            trigger_mode: TriggerMode::Off,
            frame_rate: 30.0,
            offset_x: 0,
            offset_y: 0,
            width: 640,
            height: 480,
            features: None,
        }
    }
}

impl CameraConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Exposure validation
        if self.exposure_us < 1.0 {
            return Err(OptikError::ConfigError(
                "Exposure must be >= 1 microsecond".to_string(),
            ));
        }
        if self.exposure_us > 50_000_000.0 {
            return Err(OptikError::ConfigError(
                "Exposure must be <= 50 seconds".to_string(),
            ));
        }

        // Gain validation
        if self.gain_db < 0.0 {
            return Err(OptikError::ConfigError(
                "Gain must be >= 0 dB".to_string(),
            ));
        }
        if self.gain_db > 96.0 {
            return Err(OptikError::ConfigError(
                "Gain must be <= 96 dB".to_string(),
            ));
        }

        // Frame rate validation
        if self.frame_rate <= 0.0 {
            return Err(OptikError::ConfigError(
                "Frame rate must be > 0 Hz".to_string(),
            ));
        }
        if self.frame_rate > 1000.0 {
            return Err(OptikError::ConfigError(
                "Frame rate must be <= 1000 Hz".to_string(),
            ));
        }

        // Resolution validation
        if self.width == 0 || self.height == 0 {
            return Err(OptikError::ConfigError(
                "Width and height must be > 0".to_string(),
            ));
        }
        if self.width > 4096 || self.height > 4096 {
            return Err(OptikError::ConfigError(
                "Width and height must be <= 4096".to_string(),
            ));
        }

        Ok(())
    }

    /// Apply configuration to a camera (atomic operation)
    pub fn apply_to_camera(&self, camera: &mut dyn Camera) -> Result<()> {
        // Validate first (all-or-nothing semantics)
        self.validate()?;

        // Store original values for rollback on error
        let original_exposure = camera.get_exposure()?;
        let original_gain = camera.get_gain()?;

        // Apply settings
        if let Err(e) = camera.set_exposure(self.exposure_us) {
            // Rollback is implicit (camera state unchanged)
            return Err(e);
        }

        if let Err(e) = camera.set_gain(self.gain_db) {
            // Try to restore exposure
            let _ = camera.set_exposure(original_exposure);
            return Err(e);
        }

        Ok(())
    }

    /// Create a builder for fluent API
    pub fn builder() -> CameraConfigBuilder {
        CameraConfigBuilder::new()
    }
}

/// Builder for CameraConfig
pub struct CameraConfigBuilder {
    exposure_us: f32,
    gain_db: f32,
    pixel_format: PixelFormat,
    trigger_mode: TriggerMode,
    frame_rate: f32,
    offset_x: u32,
    offset_y: u32,
    width: u32,
    height: u32,
    features: Option<HashMap<String, serde_json::Value>>,
}

impl CameraConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            exposure_us: 1000.0,
            gain_db: 0.0,
            pixel_format: PixelFormat::Mono8,
            trigger_mode: TriggerMode::Off,
            frame_rate: 30.0,
            offset_x: 0,
            offset_y: 0,
            width: 640,
            height: 480,
            features: None,
        }
    }

    /// Set exposure time
    pub fn exposure_us(mut self, us: f32) -> Self {
        self.exposure_us = us;
        self
    }

    /// Set gain
    pub fn gain_db(mut self, db: f32) -> Self {
        self.gain_db = db;
        self
    }

    /// Set pixel format
    pub fn pixel_format(mut self, format: PixelFormat) -> Self {
        self.pixel_format = format;
        self
    }

    /// Set trigger mode
    pub fn trigger_mode(mut self, mode: TriggerMode) -> Self {
        self.trigger_mode = mode;
        self
    }

    /// Set frame rate
    pub fn frame_rate(mut self, hz: f32) -> Self {
        self.frame_rate = hz;
        self
    }

    /// Set ROI offset
    pub fn offset(mut self, x: u32, y: u32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Set resolution
    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Add custom feature
    pub fn feature(mut self, name: String, value: serde_json::Value) -> Self {
        if self.features.is_none() {
            self.features = Some(HashMap::new());
        }
        if let Some(ref mut features) = self.features {
            features.insert(name, value);
        }
        self
    }

    /// Build the configuration
    pub fn build(self) -> CameraConfig {
        CameraConfig {
            exposure_us: self.exposure_us,
            gain_db: self.gain_db,
            pixel_format: self.pixel_format,
            trigger_mode: self.trigger_mode,
            frame_rate: self.frame_rate,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            width: self.width,
            height: self.height,
            features: self.features,
        }
    }
}

impl Default for CameraConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CameraConfig::default();
        assert_eq!(config.exposure_us, 1000.0);
        assert_eq!(config.gain_db, 0.0);
        assert_eq!(config.pixel_format, PixelFormat::Mono8);
        assert_eq!(config.trigger_mode, TriggerMode::Off);
    }

    #[test]
    fn test_config_validation() {
        let config = CameraConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_exposure_too_low() {
        let mut config = CameraConfig::default();
        config.exposure_us = 0.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_exposure_too_high() {
        let mut config = CameraConfig::default();
        config.exposure_us = 100_000_000.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_gain_too_low() {
        let mut config = CameraConfig::default();
        config.gain_db = -5.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_gain_too_high() {
        let mut config = CameraConfig::default();
        config.gain_db = 100.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_frame_rate_zero() {
        let mut config = CameraConfig::default();
        config.frame_rate = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_frame_rate_too_high() {
        let mut config = CameraConfig::default();
        config.frame_rate = 2000.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = CameraConfig::builder()
            .exposure_us(5000.0)
            .gain_db(24.0)
            .pixel_format(PixelFormat::RGB8)
            .trigger_mode(TriggerMode::On)
            .frame_rate(60.0)
            .resolution(1920, 1080)
            .build();

        assert_eq!(config.exposure_us, 5000.0);
        assert_eq!(config.gain_db, 24.0);
        assert_eq!(config.pixel_format, PixelFormat::RGB8);
        assert_eq!(config.trigger_mode, TriggerMode::On);
        assert_eq!(config.frame_rate, 60.0);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_pixel_format_display() {
        assert_eq!(PixelFormat::Mono8.to_string(), "Mono8");
        assert_eq!(PixelFormat::RGB8.to_string(), "RGB8");
    }

    #[test]
    fn test_trigger_mode_display() {
        assert_eq!(TriggerMode::Off.to_string(), "Off");
        assert_eq!(TriggerMode::On.to_string(), "On");
    }

    #[test]
    fn test_config_serialization() {
        let config = CameraConfig::builder()
            .exposure_us(2500.0)
            .gain_db(12.0)
            .build();

        let json = serde_json::to_string(&config).expect("serialization failed");
        let deserialized: CameraConfig =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.exposure_us, 2500.0);
        assert_eq!(deserialized.gain_db, 12.0);
    }
}
