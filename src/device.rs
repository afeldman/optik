/// Device information and metadata structures.
///
/// Provides abstractions for camera devices discovered via various controllers.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of camera controllers supported by optik.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControllerType {
    /// Basler Pylon (GigE, USB3, CoaXPress)
    Basler,
    /// IDS (USB3, Ethernet)
    IDS,
    /// Raspberry Pi Camera Module
    RPi,
    /// Generic GigE Vision
    GigE,
}

impl fmt::Display for ControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControllerType::Basler => write!(f, "Basler"),
            ControllerType::IDS => write!(f, "IDS"),
            ControllerType::RPi => write!(f, "RPi"),
            ControllerType::GigE => write!(f, "GigE"),
        }
    }
}

/// Complete device information returned by device discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier (manufacturer-specific)
    pub device_id: String,

    /// Human-readable device model name
    pub model_name: String,

    /// Device serial number
    pub serial_number: String,

    /// Type of controller managing this device
    pub controller_type: ControllerType,

    /// Whether the device is currently available (not in use)
    pub available: bool,

    /// Optional vendor name
    pub vendor: Option<String>,

    /// Optional device firmware version
    pub firmware_version: Option<String>,

    /// Optional IP address (for network-based devices)
    pub ip_address: Option<String>,

    /// Optional MAC address (for network-based devices)
    pub mac_address: Option<String>,

    /// Additional metadata as JSON
    pub metadata: Option<serde_json::Value>,
}

impl DeviceInfo {
    /// Create a new DeviceInfo with minimal required fields.
    pub fn new(
        device_id: String,
        model_name: String,
        serial_number: String,
        controller_type: ControllerType,
    ) -> Self {
        Self {
            device_id,
            model_name,
            serial_number,
            controller_type,
            available: true,
            vendor: None,
            firmware_version: None,
            ip_address: None,
            mac_address: None,
            metadata: None,
        }
    }

    /// Create a DeviceInfo from a builder pattern.
    pub fn builder(device_id: String, model_name: String, controller_type: ControllerType) -> DeviceInfoBuilder {
        DeviceInfoBuilder {
            device_id,
            model_name,
            serial_number: String::from("UNKNOWN"),
            controller_type,
            available: true,
            vendor: None,
            firmware_version: None,
            ip_address: None,
            mac_address: None,
            metadata: None,
        }
    }

    /// Get a display-friendly identifier.
    pub fn friendly_name(&self) -> String {
        format!(
            "{} {} ({})",
            self.model_name, self.serial_number, self.controller_type
        )
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{} [{}]",
            self.model_name, self.device_id, self.controller_type
        )
    }
}

/// Builder for constructing DeviceInfo with optional fields.
pub struct DeviceInfoBuilder {
    device_id: String,
    model_name: String,
    serial_number: String,
    controller_type: ControllerType,
    available: bool,
    vendor: Option<String>,
    firmware_version: Option<String>,
    ip_address: Option<String>,
    mac_address: Option<String>,
    metadata: Option<serde_json::Value>,
}

impl DeviceInfoBuilder {
    /// Set the serial number.
    pub fn serial_number(mut self, serial: String) -> Self {
        self.serial_number = serial;
        self
    }

    /// Set the vendor name.
    pub fn vendor(mut self, vendor: String) -> Self {
        self.vendor = Some(vendor);
        self
    }

    /// Set the firmware version.
    pub fn firmware_version(mut self, version: String) -> Self {
        self.firmware_version = Some(version);
        self
    }

    /// Set the IP address.
    pub fn ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set the MAC address.
    pub fn mac_address(mut self, mac: String) -> Self {
        self.mac_address = Some(mac);
        self
    }

    /// Set the availability status.
    pub fn available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    /// Set additional metadata.
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build the final DeviceInfo.
    pub fn build(self) -> DeviceInfo {
        DeviceInfo {
            device_id: self.device_id,
            model_name: self.model_name,
            serial_number: self.serial_number,
            controller_type: self.controller_type,
            available: self.available,
            vendor: self.vendor,
            firmware_version: self.firmware_version,
            ip_address: self.ip_address,
            mac_address: self.mac_address,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_new() {
        let device = DeviceInfo::new(
            "dev_001".to_string(),
            "Basler ace2".to_string(),
            "SN123456".to_string(),
            ControllerType::Basler,
        );

        assert_eq!(device.device_id, "dev_001");
        assert_eq!(device.model_name, "Basler ace2");
        assert_eq!(device.serial_number, "SN123456");
        assert_eq!(device.controller_type, ControllerType::Basler);
        assert!(device.available);
    }

    #[test]
    fn test_device_info_builder() {
        let device = DeviceInfo::builder(
            "dev_002".to_string(),
            "IDS Ensenso".to_string(),
            ControllerType::IDS,
        )
        .serial_number("IDS_SN789".to_string())
        .vendor("IDS Imaging".to_string())
        .firmware_version("2.1.0".to_string())
        .build();

        assert_eq!(device.device_id, "dev_002");
        assert_eq!(device.model_name, "IDS Ensenso");
        assert_eq!(device.vendor, Some("IDS Imaging".to_string()));
        assert_eq!(device.firmware_version, Some("2.1.0".to_string()));
    }

    #[test]
    fn test_device_info_friendly_name() {
        let device = DeviceInfo::new(
            "dev_003".to_string(),
            "RPi Camera".to_string(),
            "RPi_001".to_string(),
            ControllerType::RPi,
        );

        assert_eq!(device.friendly_name(), "RPi Camera RPi_001 (RPi)");
    }

    #[test]
    fn test_controller_type_display() {
        assert_eq!(ControllerType::Basler.to_string(), "Basler");
        assert_eq!(ControllerType::IDS.to_string(), "IDS");
        assert_eq!(ControllerType::RPi.to_string(), "RPi");
        assert_eq!(ControllerType::GigE.to_string(), "GigE");
    }

    #[test]
    fn test_device_info_serialization() {
        let device = DeviceInfo::new(
            "dev_004".to_string(),
            "Test Camera".to_string(),
            "TEST_SN".to_string(),
            ControllerType::GigE,
        );

        let json = serde_json::to_string(&device).expect("serialization failed");
        let deserialized: DeviceInfo = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.device_id, device.device_id);
        assert_eq!(deserialized.model_name, device.model_name);
    }
}
