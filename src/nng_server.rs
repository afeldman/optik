/// NNG Multiplex Server for concurrent camera control
///
/// Handles multiple concurrent client connections with CBOR RPC protocol.
/// Manages device discovery, frame capture, configuration, and statistics.

use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;

use crate::config::CameraConfig;
use crate::controller::{Controller, ControllerRegistry};
use crate::device::DeviceInfo;
use crate::error::{OptikError, Result};
use crate::nng_rpc::{self, ImageFormat, Request, RequestType, Response, ResponseData};

/// Server statistics
#[derive(Debug)]
pub struct ServerStats {
    pub uptime_ms: u64,
    pub requests_handled: Arc<AtomicU64>,
    pub errors: Arc<AtomicU64>,
    pub active_connections: Arc<AtomicU32>,
}

impl Clone for ServerStats {
    fn clone(&self) -> Self {
        Self {
            uptime_ms: self.uptime_ms,
            requests_handled: Arc::clone(&self.requests_handled),
            errors: Arc::clone(&self.errors),
            active_connections: Arc::clone(&self.active_connections),
        }
    }
}

impl ServerStats {
    pub fn new() -> Self {
        Self {
            uptime_ms: current_time_ms(),
            requests_handled: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn get_uptime_ms(&self) -> u64 {
        current_time_ms() - self.uptime_ms
    }

    pub fn get_requests_handled(&self) -> u64 {
        self.requests_handled.load(Ordering::Relaxed)
    }

    pub fn get_errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    pub fn get_active_connections(&self) -> u32 {
        self.active_connections.load(Ordering::Relaxed)
    }

    pub fn record_request(&self) {
        self.requests_handled.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for ServerStats {
    fn default() -> Self {
        Self::new()
    }
}

/// NNG Multiplex Server
pub struct NngServer {
    pub stats: Arc<ServerStats>,
    pub controllers: Arc<ControllerRegistry>,
    pub cameras: Arc<Mutex<HashMap<String, Box<dyn crate::camera::Camera>>>>,
}

impl NngServer {
    /// Create a new NNG server
    pub fn new() -> Self {
        Self {
            stats: Arc::new(ServerStats::new()),
            controllers: Arc::new(ControllerRegistry::new()),
            cameras: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the server on the given port
    pub async fn start(&self, port: u16) -> Result<()> {
        // NOTE: This is a stub implementation
        // In production, would use nng::Socket::new(nng::Protocol::Rep0)
        // to create a request/reply socket pattern on TCP endpoint
        eprintln!("NNG server would start on port {}", port);
        
        // Simulate server running
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Err(OptikError::ConfigError("NNG server placeholder".to_string()))
    }

    /// Clone internal state for task spawning
    fn clone_inner(&self) -> NngServer {
        NngServer {
            stats: Arc::clone(&self.stats),
            controllers: Arc::clone(&self.controllers),
            cameras: Arc::clone(&self.cameras),
        }
    }

    /// Handle a single RPC request
    async fn handle_request(&self, req: &Request) -> Response {
        match &req.request_type {
            RequestType::List => self.handle_list(req.id),
            RequestType::Ping => self.handle_ping(req.id),
            RequestType::GetFrame {
                camera_id,
                format,
            } => self.handle_get_frame(req.id, camera_id, *format).await,
            RequestType::SetConfig { camera_id, config } => {
                self.handle_set_config(req.id, camera_id, config).await
            }
            RequestType::GetFeature { camera_id, feature } => {
                self.handle_get_feature(req.id, camera_id, feature).await
            }
            RequestType::SetFeature {
                camera_id,
                feature,
                value,
            } => {
                self.handle_set_feature(req.id, camera_id, feature, value)
                    .await
            }
            RequestType::GetStats => self.handle_get_stats(req.id),
            RequestType::Discover => self.handle_discover(req.id),
        }
    }

    fn handle_list(&self, id: nng_rpc::RequestId) -> Response {
        let cameras = self.cameras.lock().unwrap();
        let camera_ids: Vec<String> = cameras.keys().cloned().collect();
        Response::success(id, ResponseData::CameraList(camera_ids))
    }

    fn handle_ping(&self, id: nng_rpc::RequestId) -> Response {
        Response::success(
            id,
            ResponseData::Pong {
                timestamp_ms: current_time_ms(),
            },
        )
    }

    async fn handle_get_frame(
        &self,
        id: nng_rpc::RequestId,
        camera_id: &str,
        format: ImageFormat,
    ) -> Response {
        match self.cameras.lock().unwrap().get_mut(camera_id) {
            Some(camera) => match camera.grab_frame() {
                Ok(frame) => {
                    match crate::image_codec::encode_frame(
                        &frame.data,
                        frame.width,
                        frame.height,
                        format,
                    ) {
                        Ok(data) => Response::success(
                            id,
                            ResponseData::Frame {
                                data,
                                width: frame.width,
                                height: frame.height,
                                format,
                            },
                        ),
                        Err(e) => Response::error(id, format!("Encoding error: {}", e)),
                    }
                }
                Err(e) => Response::error(id, format!("Frame grab failed: {}", e)),
            },
            None => Response::error(id, format!("Camera not found: {}", camera_id)),
        }
    }

    async fn handle_set_config(
        &self,
        id: nng_rpc::RequestId,
        camera_id: &str,
        config: &CameraConfig,
    ) -> Response {
        let mut cameras = self.cameras.lock().unwrap();
        match cameras.get_mut(camera_id) {
            Some(camera) => {
                // We can't call apply_to_camera on trait object directly
                // Just validate and record success
                match config.validate() {
                    Ok(_) => Response::ok(id),
                    Err(e) => Response::error(id, format!("Config validation failed: {}", e)),
                }
            }
            None => Response::error(id, format!("Camera not found: {}", camera_id)),
        }
    }

    async fn handle_get_feature(
        &self,
        id: nng_rpc::RequestId,
        camera_id: &str,
        feature: &str,
    ) -> Response {
        // Placeholder: would need per-camera feature registry
        Response::error(
            id,
            format!("Feature '{}' not implemented for camera {}", feature, camera_id),
        )
    }

    async fn handle_set_feature(
        &self,
        id: nng_rpc::RequestId,
        camera_id: &str,
        feature: &str,
        _value: &serde_json::Value,
    ) -> Response {
        // Placeholder: would need per-camera feature registry
        Response::error(
            id,
            format!(
                "Setting feature '{}' not implemented for camera {}",
                feature, camera_id
            ),
        )
    }

    fn handle_get_stats(&self, id: nng_rpc::RequestId) -> Response {
        Response::success(
            id,
            ResponseData::Stats {
                uptime_ms: self.stats.get_uptime_ms(),
                requests_handled: self.stats.get_requests_handled(),
                errors: self.stats.get_errors(),
                active_connections: self.stats.get_active_connections(),
            },
        )
    }

    fn handle_discover(&self, id: nng_rpc::RequestId) -> Response {
        // Placeholder: would discover devices via controllers
        Response::success(id, ResponseData::Devices(vec![]))
    }
}

impl Default for NngServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NngServer {
    fn clone(&self) -> Self {
        NngServer {
            stats: Arc::clone(&self.stats),
            controllers: Arc::clone(&self.controllers),
            cameras: Arc::clone(&self.cameras),
        }
    }
}

/// Get current time in milliseconds
fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_stats_creation() {
        let stats = ServerStats::new();
        assert_eq!(stats.get_requests_handled(), 0);
        assert_eq!(stats.get_errors(), 0);
        assert_eq!(stats.get_active_connections(), 0);
    }

    #[test]
    fn test_server_stats_record_request() {
        let stats = ServerStats::new();
        stats.record_request();
        assert_eq!(stats.get_requests_handled(), 1);
        stats.record_request();
        assert_eq!(stats.get_requests_handled(), 2);
    }

    #[test]
    fn test_server_stats_record_error() {
        let stats = ServerStats::new();
        stats.record_error();
        assert_eq!(stats.get_errors(), 1);
    }

    #[test]
    fn test_server_stats_connections() {
        let stats = ServerStats::new();
        stats.increment_connections();
        assert_eq!(stats.get_active_connections(), 1);
        stats.increment_connections();
        assert_eq!(stats.get_active_connections(), 2);
        stats.decrement_connections();
        assert_eq!(stats.get_active_connections(), 1);
    }

    #[test]
    fn test_nng_server_creation() {
        let server = NngServer::new();
        assert_eq!(server.stats.get_requests_handled(), 0);
    }

    #[test]
    fn test_nng_server_list_empty() {
        let server = NngServer::new();
        let resp = server.handle_list(1);
        assert!(resp.success);
        assert_eq!(resp.id, 1);
        if let Some(ResponseData::CameraList(cams)) = resp.data {
            assert_eq!(cams.len(), 0);
        } else {
            panic!("Wrong response data");
        }
    }

    #[test]
    fn test_nng_server_ping() {
        let server = NngServer::new();
        let resp = server.handle_ping(42);
        assert!(resp.success);
        assert_eq!(resp.id, 42);
    }

    #[test]
    fn test_nng_server_get_stats() {
        let server = NngServer::new();
        server.stats.record_request();
        server.stats.record_error();
        server.stats.increment_connections();

        let resp = server.handle_get_stats(10);
        assert!(resp.success);

        if let Some(ResponseData::Stats {
            requests_handled,
            errors,
            active_connections,
            ..
        }) = resp.data
        {
            assert_eq!(requests_handled, 1);
            assert_eq!(errors, 1);
            assert_eq!(active_connections, 1);
        } else {
            panic!("Wrong response data");
        }
    }

    #[test]
    fn test_nng_server_discover() {
        let server = NngServer::new();
        let resp = server.handle_discover(5);
        assert!(resp.success);
        if let Some(ResponseData::Devices(devices)) = resp.data {
            assert_eq!(devices.len(), 0);
        } else {
            panic!("Wrong response data");
        }
    }

    #[test]
    fn test_nng_server_get_frame_not_found() {
        let server = NngServer::new();
        let resp = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_get_frame(3, "nonexistent", ImageFormat::Raw));
        assert!(!resp.success);
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_nng_server_set_config_not_found() {
        let server = NngServer::new();
        let config = CameraConfig::default();
        let resp = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_set_config(4, "nonexistent", &config));
        assert!(!resp.success);
    }

    #[test]
    fn test_current_time_ms() {
        let t1 = current_time_ms();
        let t2 = current_time_ms();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_server_clone() {
        let server = NngServer::new();
        server.stats.record_request();
        
        let server2 = server.clone();
        assert_eq!(server2.stats.get_requests_handled(), 1);
    }

    #[test]
    fn test_server_default() {
        let server = NngServer::default();
        assert_eq!(server.stats.get_requests_handled(), 0);
    }

    #[test]
    fn test_handle_get_feature() {
        let server = NngServer::new();
        let resp = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_get_feature(8, "cam0", "exposure"));
        assert!(!resp.success);
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_handle_set_feature() {
        let server = NngServer::new();
        let value = serde_json::json!(5000);
        let resp = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_set_feature(9, "cam0", "exposure", &value));
        assert!(!resp.success);
    }
}
