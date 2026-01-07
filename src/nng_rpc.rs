/// NNG-based RPC protocol for camera control
///
/// CBOR-encoded request/response protocol over NNG paired sockets.
/// Supports concurrent client-server communication with frame streaming.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::{CameraConfig, PixelFormat, TriggerMode};
use crate::device::DeviceInfo;
use crate::error::Result;
use crate::frame::Frame;

/// Unique request ID for async tracking
pub type RequestId = u64;

/// Image encoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    /// Raw pixel data
    Raw,
    /// JPEG compressed
    JPEG,
    /// PNG compressed
    PNG,
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Raw => write!(f, "Raw"),
            ImageFormat::JPEG => write!(f, "JPEG"),
            ImageFormat::PNG => write!(f, "PNG"),
        }
    }
}

/// Request variant (discriminated union)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    /// List all available cameras
    List,
    /// Ping server
    Ping,
    /// Get frame from camera
    GetFrame { camera_id: String, format: ImageFormat },
    /// Set camera configuration
    SetConfig { camera_id: String, config: CameraConfig },
    /// Get feature value
    GetFeature { camera_id: String, feature: String },
    /// Set feature value
    SetFeature {
        camera_id: String,
        feature: String,
        value: serde_json::Value,
    },
    /// Get server statistics
    GetStats,
    /// Discover devices
    Discover,
}

/// RPC Request envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: RequestId,
    pub request_type: RequestType,
}

impl Request {
    pub fn new(id: RequestId, request_type: RequestType) -> Self {
        Self { id, request_type }
    }

    pub fn list(id: RequestId) -> Self {
        Self::new(id, RequestType::List)
    }

    pub fn ping(id: RequestId) -> Self {
        Self::new(id, RequestType::Ping)
    }

    pub fn get_frame(id: RequestId, camera_id: String, format: ImageFormat) -> Self {
        Self::new(
            id,
            RequestType::GetFrame { camera_id, format },
        )
    }

    pub fn set_config(id: RequestId, camera_id: String, config: CameraConfig) -> Self {
        Self::new(id, RequestType::SetConfig { camera_id, config })
    }

    pub fn get_feature(id: RequestId, camera_id: String, feature: String) -> Self {
        Self::new(id, RequestType::GetFeature { camera_id, feature })
    }

    pub fn set_feature(
        id: RequestId,
        camera_id: String,
        feature: String,
        value: serde_json::Value,
    ) -> Self {
        Self::new(
            id,
            RequestType::SetFeature {
                camera_id,
                feature,
                value,
            },
        )
    }

    pub fn get_stats(id: RequestId) -> Self {
        Self::new(id, RequestType::GetStats)
    }

    pub fn discover(id: RequestId) -> Self {
        Self::new(id, RequestType::Discover)
    }
}

/// Response variant (discriminated union)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    /// List of available cameras
    CameraList(Vec<String>),
    /// Pong response
    Pong { timestamp_ms: u64 },
    /// Frame data
    Frame {
        data: Vec<u8>,
        width: u32,
        height: u32,
        format: ImageFormat,
    },
    /// Confirmation
    Ok,
    /// Feature value
    Feature(serde_json::Value),
    /// Server statistics
    Stats {
        uptime_ms: u64,
        requests_handled: u64,
        errors: u64,
        active_connections: u32,
    },
    /// Device discovery results
    Devices(Vec<DeviceInfo>),
}

/// RPC Response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: RequestId,
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<ResponseData>,
}

impl Response {
    pub fn success(id: RequestId, data: ResponseData) -> Self {
        Self {
            id,
            success: true,
            error: None,
            data: Some(data),
        }
    }

    pub fn error(id: RequestId, error: String) -> Self {
        Self {
            id,
            success: false,
            error: Some(error),
            data: None,
        }
    }

    pub fn ok(id: RequestId) -> Self {
        Self::success(id, ResponseData::Ok)
    }
}

/// Serialize request to CBOR bytes
pub fn encode_request(req: &Request) -> Result<Vec<u8>> {
    serde_cbor::to_vec(req).map_err(|e| {
        crate::error::OptikError::ConfigError(format!("CBOR encode error: {}", e))
    })
}

/// Deserialize request from CBOR bytes
pub fn decode_request(data: &[u8]) -> Result<Request> {
    serde_cbor::from_slice(data).map_err(|e| {
        crate::error::OptikError::ConfigError(format!("CBOR decode error: {}", e))
    })
}

/// Serialize response to CBOR bytes
pub fn encode_response(resp: &Response) -> Result<Vec<u8>> {
    serde_cbor::to_vec(resp).map_err(|e| {
        crate::error::OptikError::ConfigError(format!("CBOR encode error: {}", e))
    })
}

/// Deserialize response from CBOR bytes
pub fn decode_response(data: &[u8]) -> Result<Response> {
    serde_cbor::from_slice(data).map_err(|e| {
        crate::error::OptikError::ConfigError(format!("CBOR decode error: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_list() {
        let req = Request::list(1);
        assert_eq!(req.id, 1);
        matches!(req.request_type, RequestType::List);
    }

    #[test]
    fn test_request_ping() {
        let req = Request::ping(42);
        assert_eq!(req.id, 42);
        matches!(req.request_type, RequestType::Ping);
    }

    #[test]
    fn test_request_get_frame() {
        let req = Request::get_frame(10, "cam0".to_string(), ImageFormat::JPEG);
        assert_eq!(req.id, 10);
        if let RequestType::GetFrame { camera_id, format } = req.request_type {
            assert_eq!(camera_id, "cam0");
            assert_eq!(format, ImageFormat::JPEG);
        } else {
            panic!("Wrong request type");
        }
    }

    #[test]
    fn test_request_set_config() {
        let config = CameraConfig::default();
        let req = Request::set_config(5, "cam1".to_string(), config.clone());
        assert_eq!(req.id, 5);
        if let RequestType::SetConfig { camera_id, .. } = req.request_type {
            assert_eq!(camera_id, "cam1");
        } else {
            panic!("Wrong request type");
        }
    }

    #[test]
    fn test_response_success() {
        let resp = Response::success(1, ResponseData::Pong { timestamp_ms: 1234 });
        assert_eq!(resp.id, 1);
        assert!(resp.success);
        assert!(resp.error.is_none());
        assert!(resp.data.is_some());
    }

    #[test]
    fn test_response_error() {
        let resp = Response::error(2, "Camera not found".to_string());
        assert_eq!(resp.id, 2);
        assert!(!resp.success);
        assert!(resp.error.is_some());
        assert!(resp.data.is_none());
    }

    #[test]
    fn test_encode_decode_request() {
        let req = Request::ping(123);
        let encoded = encode_request(&req).expect("encode failed");
        let decoded = decode_request(&encoded).expect("decode failed");
        assert_eq!(decoded.id, 123);
    }

    #[test]
    fn test_encode_decode_response() {
        let resp = Response::success(456, ResponseData::Pong { timestamp_ms: 9999 });
        let encoded = encode_response(&resp).expect("encode failed");
        let decoded = decode_response(&encoded).expect("decode failed");
        assert_eq!(decoded.id, 456);
        assert!(decoded.success);
    }

    #[test]
    fn test_request_discovery() {
        let req = Request::discover(77);
        assert_eq!(req.id, 77);
        matches!(req.request_type, RequestType::Discover);
    }

    #[test]
    fn test_image_format_display() {
        assert_eq!(ImageFormat::Raw.to_string(), "Raw");
        assert_eq!(ImageFormat::JPEG.to_string(), "JPEG");
        assert_eq!(ImageFormat::PNG.to_string(), "PNG");
    }

    #[test]
    fn test_request_set_feature() {
        let value = serde_json::json!({"exposure": 5000});
        let req = Request::set_feature(11, "cam0".to_string(), "exposure".to_string(), value);
        assert_eq!(req.id, 11);
        if let RequestType::SetFeature {
            camera_id,
            feature,
            ..
        } = req.request_type
        {
            assert_eq!(camera_id, "cam0");
            assert_eq!(feature, "exposure");
        } else {
            panic!("Wrong request type");
        }
    }

    #[test]
    fn test_response_ok() {
        let resp = Response::ok(99);
        assert_eq!(resp.id, 99);
        assert!(resp.success);
        matches!(resp.data, Some(ResponseData::Ok));
    }

    #[test]
    fn test_camera_list_response() {
        let cameras = vec!["cam0".to_string(), "cam1".to_string(), "cam2".to_string()];
        let resp = Response::success(5, ResponseData::CameraList(cameras));
        assert!(resp.success);
        if let Some(ResponseData::CameraList(cams)) = resp.data {
            assert_eq!(cams.len(), 3);
        } else {
            panic!("Wrong response data");
        }
    }

    #[test]
    fn test_stats_response() {
        let resp = Response::success(
            7,
            ResponseData::Stats {
                uptime_ms: 60000,
                requests_handled: 150,
                errors: 2,
                active_connections: 3,
            },
        );
        assert!(resp.success);
        if let Some(ResponseData::Stats {
            uptime_ms,
            requests_handled,
            errors,
            active_connections,
        }) = resp.data
        {
            assert_eq!(uptime_ms, 60000);
            assert_eq!(requests_handled, 150);
            assert_eq!(errors, 2);
            assert_eq!(active_connections, 3);
        } else {
            panic!("Wrong response data");
        }
    }

    #[test]
    fn test_frame_response() {
        let frame_data = vec![0u8; 1024];
        let resp = Response::success(
            12,
            ResponseData::Frame {
                data: frame_data.clone(),
                width: 640,
                height: 480,
                format: ImageFormat::JPEG,
            },
        );
        assert!(resp.success);
        if let Some(ResponseData::Frame { width, height, .. }) = resp.data {
            assert_eq!(width, 640);
            assert_eq!(height, 480);
        } else {
            panic!("Wrong response data");
        }
    }
}
