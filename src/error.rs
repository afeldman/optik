/// Error types and handling for optik.

use crate::camera::CameraError;
use thiserror::Error;

/// Main error type for optik operations.
#[derive(Error, Debug)]
pub enum OptikError {
    /// Camera-specific errors (from CameraError)
    #[error("Camera error: {0}")]
    CameraError(#[from] CameraError),

    /// Frame processing errors
    #[error("Frame error: {0}")]
    FrameError(String),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// I/O errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Mutex/synchronization lock errors
    #[error("Lock error: {0}")]
    LockError(String),

    /// Lock acquisition timeout
    #[error("Lock timeout: {0}")]
    LockTimeout(String),

    /// Frame queue or channel errors
    #[error("Frame queue error: {0}")]
    QueueError(String),

    /// Shared memory operation errors
    #[error("Shared memory error: {0}")]
    ShmemError(String),

    /// Device discovery or controller errors
    #[error("Device error: {0}")]
    DeviceError(String),
}

/// Result type for optik operations.
pub type Result<T> = std::result::Result<T, OptikError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OptikError::ConfigError("test config error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test config error");
    }

    #[test]
    fn test_error_device() {
        let err = OptikError::DeviceError("device not found".to_string());
        assert_eq!(err.to_string(), "Device error: device not found");
    }

    #[test]
    fn test_result_type() {
        let _result: Result<()> = Err(OptikError::LockTimeout("test".to_string()));
        assert!(_result.is_err());
    }
}
