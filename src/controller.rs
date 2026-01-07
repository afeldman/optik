/// Camera controller abstraction and device discovery trait.
///
/// Defines the common interface for all camera controller types (Basler, IDS, RPi, GigE).
/// Controllers are responsible for discovering available devices and creating Camera instances.

use crate::camera::Camera;
use crate::device::{ControllerType, DeviceInfo};
use crate::error::OptikError;
use std::sync::Arc;

/// Result type for controller operations.
pub type ControllerResult<T> = Result<T, OptikError>;

/// Abstract interface for camera controllers.
///
/// A `Controller` is responsible for:
/// - Discovering available camera devices
/// - Opening and creating Camera instances from devices
/// - Managing device metadata
///
/// All implementations must be thread-safe (Send + Sync).
pub trait Controller: Send + Sync {
    /// Discover all available camera devices of this controller type.
    ///
    /// This may involve network scanning (for GigE), USB enumeration (for Basler/IDS),
    /// or checking local resources (for RPi).
    ///
    /// # Returns
    /// A vector of available DeviceInfo structs. May be empty if no devices found.
    ///
    /// # Errors
    /// Returns an error if discovery fails (e.g., driver not loaded, hardware issue).
    fn discover_devices(&self) -> ControllerResult<Vec<DeviceInfo>>;

    /// Create a Camera instance from a DeviceInfo.
    ///
    /// This opens the device and prepares it for frame capture.
    /// The returned Camera is thread-safe and can be used with MultiCameraHandler.
    ///
    /// # Arguments
    /// * `device` - DeviceInfo obtained from discover_devices()
    ///
    /// # Returns
    /// A boxed Camera instance ready for use
    ///
    /// # Errors
    /// Returns an error if the device cannot be opened or is already in use.
    fn open_camera(&self, device: &DeviceInfo) -> ControllerResult<Box<dyn Camera>>;

    /// Get the type of this controller.
    fn controller_type(&self) -> ControllerType;

    /// Get a human-readable name for this controller.
    ///
    /// Default implementation returns the controller type name.
    fn name(&self) -> String {
        self.controller_type().to_string()
    }
}

/// Registry managing multiple camera controllers.
///
/// Allows discovery and access to cameras from all available controller types.
/// This is the primary entry point for device discovery in optik.
pub struct ControllerRegistry {
    controllers: Vec<Arc<dyn Controller>>,
}

impl ControllerRegistry {
    /// Create a new empty controller registry.
    pub fn new() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    /// Register a new controller.
    ///
    /// # Arguments
    /// * `controller` - An Arc-wrapped controller implementation
    pub fn register(&mut self, controller: Arc<dyn Controller>) {
        self.controllers.push(controller);
    }

    /// Discover all devices from all registered controllers.
    ///
    /// # Returns
    /// A vector of all available devices across all controllers
    pub fn discover_all(&self) -> ControllerResult<Vec<DeviceInfo>> {
        let mut all_devices = Vec::new();

        for controller in &self.controllers {
            match controller.discover_devices() {
                Ok(devices) => all_devices.extend(devices),
                Err(e) => {
                    // Log the error but continue discovering from other controllers
                    eprintln!("Warning: {} discovery failed: {}", controller.name(), e);
                }
            }
        }

        Ok(all_devices)
    }

    /// Discover devices from a specific controller type.
    ///
    /// # Arguments
    /// * `controller_type` - The type of controller to query
    ///
    /// # Returns
    /// Devices from that controller type, or empty vec if no devices found
    pub fn discover_by_type(&self, controller_type: ControllerType) -> ControllerResult<Vec<DeviceInfo>> {
        for controller in &self.controllers {
            if controller.controller_type() == controller_type {
                return controller.discover_devices();
            }
        }

        Ok(Vec::new()) // No controller of this type registered
    }

    /// Get a controller by type.
    ///
    /// # Arguments
    /// * `controller_type` - The type of controller to retrieve
    ///
    /// # Returns
    /// Reference to the controller, or None if not found
    pub fn get_controller(&self, controller_type: ControllerType) -> Option<&Arc<dyn Controller>> {
        self.controllers
            .iter()
            .find(|c| c.controller_type() == controller_type)
    }

    /// Open a camera from its DeviceInfo.
    ///
    /// This will find the appropriate controller and open the camera.
    ///
    /// # Arguments
    /// * `device` - DeviceInfo for the camera to open
    ///
    /// # Returns
    /// A boxed Camera instance
    pub fn open_camera(&self, device: &DeviceInfo) -> ControllerResult<Box<dyn Camera>> {
        for controller in &self.controllers {
            if controller.controller_type() == device.controller_type {
                return controller.open_camera(device);
            }
        }

        Err(OptikError::ConfigError(format!(
            "No controller found for device type: {}",
            device.controller_type
        )))
    }

    /// Get the number of registered controllers.
    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }
}

impl Default for ControllerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Mock Controller for Testing
// ============================================================================

/// Mock controller for testing device discovery and camera opening.
pub struct MockController {
    devices: Vec<DeviceInfo>,
}

impl MockController {
    /// Create a new mock controller with default test devices.
    pub fn new() -> Self {
        Self {
            devices: vec![DeviceInfo::new(
                "mock_basler_001".to_string(),
                "Basler ace2 Pro".to_string(),
                "MOCK_BASLER_SN001".to_string(),
                ControllerType::Basler,
            )],
        }
    }

    /// Create a mock controller with custom devices.
    pub fn with_devices(devices: Vec<DeviceInfo>) -> Self {
        Self { devices }
    }
}

impl Default for MockController {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller for MockController {
    fn discover_devices(&self) -> ControllerResult<Vec<DeviceInfo>> {
        Ok(self.devices.clone())
    }

    fn open_camera(&self, device: &DeviceInfo) -> ControllerResult<Box<dyn Camera>> {
        // For mock, we return an RpiCamera (the simplest implementation)
        // In a real test, this would check if the device exists in our device list
        if !self.devices.iter().any(|d| d.device_id == device.device_id) {
            return Err(OptikError::DeviceError(format!(
                "Device not found in mock controller: {}",
                device.device_id
            )));
        }

        // Return a mock/dummy camera - create a new RpiCamera instance
        let mut cam = crate::camera::RpiCamera::new(0);
        cam.open()?;
        Ok(Box::new(cam) as Box<dyn Camera>)
    }

    fn controller_type(&self) -> ControllerType {
        ControllerType::Basler // Mock represents Basler for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_type_display() {
        let ctrl = MockController::new();
        assert_eq!(ctrl.controller_type(), ControllerType::Basler);
    }

    #[test]
    fn test_mock_controller_discover() {
        let ctrl = MockController::new();
        let devices = ctrl.discover_devices().expect("discovery failed");

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "mock_basler_001");
    }

    #[test]
    fn test_mock_controller_custom_devices() {
        let custom_device = DeviceInfo::new(
            "custom_device".to_string(),
            "Custom Camera".to_string(),
            "CUSTOM_SN".to_string(),
            ControllerType::GigE,
        );

        let ctrl = MockController::with_devices(vec![custom_device.clone()]);
        let devices = ctrl.discover_devices().expect("discovery failed");

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "custom_device");
    }

    #[test]
    fn test_controller_registry_register() {
        let mut registry = ControllerRegistry::new();
        assert_eq!(registry.controller_count(), 0);

        let mock_ctrl = Arc::new(MockController::new());
        registry.register(mock_ctrl);

        assert_eq!(registry.controller_count(), 1);
    }

    #[test]
    fn test_controller_registry_discover_all() {
        let mut registry = ControllerRegistry::new();

        let mock_ctrl = Arc::new(MockController::new());
        registry.register(mock_ctrl);

        let devices = registry.discover_all().expect("discovery failed");
        assert_eq!(devices.len(), 1);
    }

    #[test]
    fn test_controller_registry_discover_by_type() {
        let mut registry = ControllerRegistry::new();

        let mock_ctrl = Arc::new(MockController::new());
        registry.register(mock_ctrl);

        let basler_devices = registry
            .discover_by_type(ControllerType::Basler)
            .expect("discovery failed");

        assert_eq!(basler_devices.len(), 1);
        assert_eq!(basler_devices[0].device_id, "mock_basler_001");
    }

    #[test]
    fn test_controller_registry_open_camera() {
        let mut registry = ControllerRegistry::new();

        let mock_ctrl = Arc::new(MockController::new());
        registry.register(mock_ctrl);

        let devices = registry.discover_all().expect("discovery failed");
        let camera_result = registry.open_camera(&devices[0]);

        // Should succeed (opens an RpiCamera)
        assert!(camera_result.is_ok());
    }

    #[test]
    fn test_controller_registry_open_nonexistent() {
        let mut registry = ControllerRegistry::new();
        let mock_ctrl = Arc::new(MockController::new());
        registry.register(mock_ctrl);

        let fake_device = DeviceInfo::new(
            "nonexistent".to_string(),
            "Fake Camera".to_string(),
            "FAKE_SN".to_string(),
            ControllerType::Basler,
        );

        let result = registry.open_camera(&fake_device);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_controller_types() {
        let mut registry = ControllerRegistry::new();

        let devices = vec![
            DeviceInfo::new(
                "basler_dev".to_string(),
                "Basler Camera".to_string(),
                "BASLER_SN".to_string(),
                ControllerType::Basler,
            ),
            DeviceInfo::new(
                "ids_dev".to_string(),
                "IDS Camera".to_string(),
                "IDS_SN".to_string(),
                ControllerType::IDS,
            ),
        ];

        let mock_ctrl = Arc::new(MockController::with_devices(devices.clone()));
        registry.register(mock_ctrl);

        let all_devices = registry.discover_all().expect("discovery failed");
        assert_eq!(all_devices.len(), 2);

        // Verify we can identify each device type
        let basler_count = all_devices
            .iter()
            .filter(|d| d.controller_type == ControllerType::Basler)
            .count();
        assert_eq!(basler_count, 1);
    }
}
