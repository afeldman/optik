use crate::{Camera, Frame, Result, OptikError};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use tokio::task::JoinHandle;
use std::collections::HashMap;

/// Configuration for multi-camera capture
#[derive(Debug, Clone)]
pub struct MultiCameraConfig {
    pub num_cameras: usize,
    pub timeout_ms: u64,
    pub max_queue_size: usize,
    pub frame_rate_hz: u32,
}

impl Default for MultiCameraConfig {
    fn default() -> Self {
        MultiCameraConfig {
            num_cameras: 4,
            timeout_ms: 5000,
            max_queue_size: 30,
            frame_rate_hz: 30,
        }
    }
}

/// Frame queue item with camera metadata
#[derive(Clone)]
pub struct QueuedFrame {
    pub camera_id: u32,
    pub frame: Arc<Frame>,
    pub captured_at: std::time::Instant,
}

/// Multi-camera async handler using Tokio
pub struct MultiCameraHandler {
    cameras: Arc<StdMutex<HashMap<u32, Arc<StdMutex<Box<dyn Camera>>>>>>,
    frame_queue: Arc<tokio::sync::Mutex<Vec<QueuedFrame>>>,
    config: MultiCameraConfig,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl MultiCameraHandler {
    /// Create a new multi-camera handler
    pub fn new(config: MultiCameraConfig) -> Self {
        MultiCameraHandler {
            cameras: Arc::new(StdMutex::new(HashMap::new())),
            frame_queue: Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(config.max_queue_size))),
            config,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Register a camera with given ID
    pub fn register_camera(&self, id: u32, camera: Arc<StdMutex<Box<dyn Camera>>>) -> Result<()> {
        let mut cameras = self.cameras.lock()
            .map_err(|_| OptikError::LockError("Failed to acquire camera registry lock".to_string()))?;
        
        if cameras.contains_key(&id) {
            return Err(OptikError::ConfigError(format!("Camera {} already registered", id)));
        }
        
        cameras.insert(id, camera);
        tracing::info!("Registered camera {}", id);
        Ok(())
    }

    /// Unregister a camera
    pub fn unregister_camera(&self, id: u32) -> Result<()> {
        let mut cameras = self.cameras.lock()
            .map_err(|_| OptikError::LockError("Failed to acquire camera registry lock".to_string()))?;
        
        cameras.remove(&id);
        tracing::info!("Unregistered camera {}", id);
        Ok(())
    }

    /// Start async capture for all registered cameras
    pub fn start_capture(&self) -> Result<Vec<JoinHandle<Result<()>>>> {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        
        let cameras = self.cameras.lock()
            .map_err(|_| OptikError::LockError("Failed to acquire camera registry lock".to_string()))?;
        
        let mut handles = Vec::new();
        let interval = Duration::from_millis(1000 / self.config.frame_rate_hz as u64);

        for (&camera_id, camera) in cameras.iter() {
            let camera_clone = Arc::clone(camera);
            let queue_clone = Arc::clone(&self.frame_queue);
            let running_clone = Arc::clone(&self.running);
            let max_queue = self.config.max_queue_size;
            let timeout = Duration::from_millis(self.config.timeout_ms);

            let handle = tokio::spawn(async move {
                while running_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    let start = std::time::Instant::now();

                    // Try to grab frame with timeout
                    let frame_result = {
                        let mut cam = camera_clone.lock()
                            .map_err(|_| OptikError::LockError(
                                format!("Camera {} lock poisoned", camera_id)
                            ))?;
                        cam.grab_frame()
                    };

                    match frame_result {
                        Ok(frame) => {
                            // Add to queue
                            let queued = QueuedFrame {
                                camera_id,
                                frame: Arc::new(frame),
                                captured_at: std::time::Instant::now(),
                            };

                            let mut queue = queue_clone.lock().await;
                            if queue.len() >= max_queue {
                                queue.remove(0);  // Drop oldest frame
                                tracing::warn!("Frame queue full for camera {}, dropping oldest", camera_id);
                            }
                            queue.push(queued);
                        }
                        Err(e) => {
                            tracing::error!("Camera {} frame grab failed: {}", camera_id, e);
                        }
                    }

                    // Rate limiting
                    let elapsed = start.elapsed();
                    if elapsed < interval {
                        tokio::time::sleep(interval - elapsed).await;
                    }
                }
                Ok(())
            });

            handles.push(handle);
        }

        Ok(handles)
    }

    /// Stop all capture tasks
    pub fn stop_capture(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("Stopping all capture tasks");
    }

    /// Get next frame from queue (non-blocking)
    pub async fn get_frame(&self) -> Option<QueuedFrame> {
        let mut queue = self.frame_queue.lock().await;
        if queue.is_empty() {
            return None;
        }
        Some(queue.remove(0))
    }

    /// Get all pending frames (drains queue)
    pub async fn get_all_frames(&self) -> Vec<QueuedFrame> {
        let mut queue = self.frame_queue.lock().await;
        std::mem::take(&mut *queue)
    }

    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.frame_queue.lock().await.len()
    }

    /// Get number of registered cameras
    pub fn camera_count(&self) -> Result<usize> {
        let cameras = self.cameras.lock()
            .map_err(|_| OptikError::LockError("Failed to acquire camera registry lock".to_string()))?;
        Ok(cameras.len())
    }

    /// Check if capture is running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::RpiCamera;

    #[test]
    fn test_multi_camera_config_default() {
        let config = MultiCameraConfig::default();
        assert_eq!(config.num_cameras, 4);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_register_unregister_camera() {
        let handler = MultiCameraHandler::new(MultiCameraConfig::default());
        let cam = Arc::new(StdMutex::new(Box::new(RpiCamera::new(0)) as Box<dyn Camera>));
        
        assert!(handler.register_camera(0, cam).is_ok());
        assert_eq!(handler.camera_count().unwrap(), 1);
        
        assert!(handler.unregister_camera(0).is_ok());
        assert_eq!(handler.camera_count().unwrap(), 0);
    }

    #[test]
    fn test_duplicate_registration() {
        let handler = MultiCameraHandler::new(MultiCameraConfig::default());
        let cam = Arc::new(StdMutex::new(Box::new(RpiCamera::new(0)) as Box<dyn Camera>));
        
        assert!(handler.register_camera(0, Arc::clone(&cam)).is_ok());
        assert!(handler.register_camera(0, cam).is_err());
    }

    #[tokio::test]
    async fn test_handler_start_stop() {
        let handler = MultiCameraHandler::new(MultiCameraConfig {
            num_cameras: 1,
            timeout_ms: 1000,
            max_queue_size: 10,
            frame_rate_hz: 10,
        });
        
        let cam = Arc::new(StdMutex::new(Box::new(RpiCamera::new(0)) as Box<dyn Camera>));
        let _ = handler.register_camera(0, cam);
        
        let handles = handler.start_capture().unwrap();
        assert!(handler.is_running());
        
        handler.stop_capture();
        assert!(!handler.is_running());
    }

    #[tokio::test]
    async fn test_queue_management() {
        let handler = MultiCameraHandler::new(MultiCameraConfig {
            num_cameras: 1,
            timeout_ms: 1000,
            max_queue_size: 5,
            frame_rate_hz: 30,
        });
        
        assert_eq!(handler.queue_size().await, 0);
        
        let frames = handler.get_all_frames().await;
        assert!(frames.is_empty());
    }
}
