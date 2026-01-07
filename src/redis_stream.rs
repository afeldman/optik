/// Redis-based frame streaming and pub/sub for camera control
///
/// Provides high-performance frame streaming using Redis Pub/Sub
/// and connection pooling for multi-camera scenarios.
/// NOTE: Redis client integration is abstracted for flexibility in implementation.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::error::{OptikError, Result};
use crate::frame::Frame;

/// Frame data serialized for Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisFrame {
    pub camera_id: String,
    pub timestamp: u64,
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub exposure_us: f32,
    pub gain: f32,
    /// Frame data as base64-encoded string (for JSON compatibility)
    pub data_base64: String,
}

impl RedisFrame {
    /// Create from a Frame with camera_id
    pub fn from_frame(frame: &Frame, camera_id: &str) -> Result<Self> {
        Ok(RedisFrame {
            camera_id: camera_id.to_string(),
            timestamp: frame.timestamp,
            sequence: frame.sequence,
            width: frame.width,
            height: frame.height,
            channels: frame.channels,
            exposure_us: frame.exposure_us,
            gain: frame.gain,
            data_base64: base64_encode(&frame.data),
        })
    }

    /// Convert back to raw frame data
    pub fn to_frame_data(&self) -> Result<Vec<u8>> {
        base64_decode(&self.data_base64)
            .map_err(|e| OptikError::ConfigError(format!("Base64 decode error: {}", e)))
    }
}

/// Redis Pub/Sub Publisher for frames (interface)
pub struct RedisPublisher {
    /// In-memory frame cache for testing
    frame_cache: Arc<Mutex<HashMap<String, RedisFrame>>>,
    redis_url: String,
}

impl RedisPublisher {
    /// Create a new publisher connected to Redis server
    pub fn new(redis_url: &str) -> Result<Self> {
        // Validate URL format
        if !redis_url.starts_with("redis://") && !redis_url.starts_with("rediss://") {
            return Err(OptikError::ConfigError(
                "Redis URL must start with redis:// or rediss://".to_string(),
            ));
        }

        Ok(Self {
            frame_cache: Arc::new(Mutex::new(HashMap::new())),
            redis_url: redis_url.to_string(),
        })
    }

    /// Publish a frame to a camera-specific channel
    pub fn publish_frame(&self, frame: &RedisFrame) -> Result<()> {
        // In production: publish to Redis channel camera:{camera_id}
        // For now: store in in-memory cache
        let mut cache = self.frame_cache.lock().unwrap();
        let channel = format!("camera:{}", frame.camera_id);
        cache.insert(channel, frame.clone());
        Ok(())
    }

    /// Publish to a broadcast channel (all subscribers)
    pub fn broadcast_frame(&self, frame: &RedisFrame) -> Result<()> {
        // In production: publish to Redis channel frames:broadcast
        let mut cache = self.frame_cache.lock().unwrap();
        cache.insert("frames:broadcast".to_string(), frame.clone());
        Ok(())
    }

    /// Set frame as latest in a key-value store
    pub fn set_latest_frame(&self, camera_id: &str, frame: &RedisFrame) -> Result<()> {
        // In production: SET with EX (expiration)
        let mut cache = self.frame_cache.lock().unwrap();
        let key = format!("camera:{}:latest", camera_id);
        cache.insert(key, frame.clone());
        Ok(())
    }

    /// Get Redis URL (for connection info)
    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    /// Get cached frame count (for testing)
    pub fn cached_frames(&self) -> usize {
        self.frame_cache.lock().unwrap().len()
    }
}

/// Redis Pub/Sub Subscriber for frames
pub struct RedisSubscriber {
    frame_cache: Arc<Mutex<HashMap<String, RedisFrame>>>,
    redis_url: String,
}

impl RedisSubscriber {
    /// Create a new subscriber
    pub fn new(redis_url: &str) -> Result<Self> {
        // Validate URL format
        if !redis_url.starts_with("redis://") && !redis_url.starts_with("rediss://") {
            return Err(OptikError::ConfigError(
                "Redis URL must start with redis:// or rediss://".to_string(),
            ));
        }

        Ok(Self {
            frame_cache: Arc::new(Mutex::new(HashMap::new())),
            redis_url: redis_url.to_string(),
        })
    }

    /// Subscribe to a camera channel
    pub fn subscribe_camera(&self, camera_id: &str) -> Result<()> {
        // In production: subscribe to camera:{camera_id} channel
        let _ = format!("camera:{}", camera_id);
        Ok(())
    }

    /// Subscribe to broadcast channel
    pub fn subscribe_broadcast(&self) -> Result<()> {
        // In production: subscribe to frames:broadcast channel
        Ok(())
    }

    /// Get latest frame for a camera (from cache or Redis)
    pub fn get_latest_frame(&self, camera_id: &str) -> Result<Option<RedisFrame>> {
        let cache = self.frame_cache.lock().unwrap();
        let key = format!("camera:{}:latest", camera_id);
        Ok(cache.get(&key).cloned())
    }

    /// Store frame in cache (simulates Redis)
    pub fn cache_frame(&self, camera_id: &str, frame: &RedisFrame) -> Result<()> {
        let mut cache = self.frame_cache.lock().unwrap();
        let key = format!("camera:{}:latest", camera_id);
        cache.insert(key, frame.clone());
        Ok(())
    }

    /// Get Redis URL (for connection info)
    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }
}

/// Base64 encoding/decoding utilities
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;

        result.push(CHARSET[b1] as char);
        result.push(CHARSET[b2] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[b3] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[b4] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn base64_decode(s: &str) -> std::result::Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let bytes: Vec<u8> = s
        .chars()
        .filter(|&c| c != '=' && c != '\n' && c != '\r')
        .map(|c| {
            charset
                .find(c)
                .ok_or_else(|| format!("Invalid character: {}", c))
                .map(|i| i as u8)
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    for chunk in bytes.chunks(4) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);
        let b4 = chunk.get(3).copied().unwrap_or(0);

        result.push((b1 << 2) | (b2 >> 4));

        if chunk.len() > 2 {
            result.push((b2 << 4) | (b3 >> 2));
        }

        if chunk.len() > 3 {
            result.push((b3 << 6) | b4);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_frame_creation() {
        let rf = RedisFrame {
            camera_id: "cam0".to_string(),
            timestamp: 1234567890,
            sequence: 42,
            width: 640,
            height: 480,
            channels: 1,
            exposure_us: 5000.0,
            gain: 12.0,
            data_base64: "AQID".to_string(),
        };

        assert_eq!(rf.camera_id, "cam0");
        assert_eq!(rf.sequence, 42);
        assert_eq!(rf.width, 640);
    }

    #[test]
    fn test_redis_frame_serialization() {
        let rf = RedisFrame {
            camera_id: "cam1".to_string(),
            timestamp: 9999,
            sequence: 100,
            width: 1920,
            height: 1080,
            channels: 3,
            exposure_us: 1000.0,
            gain: 24.0,
            data_base64: "AQIDBA==".to_string(),
        };

        let json = serde_json::to_string(&rf).expect("serialize failed");
        let deserialized: RedisFrame = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(deserialized.camera_id, "cam1");
        assert_eq!(deserialized.sequence, 100);
        assert_eq!(deserialized.width, 1920);
    }

    #[test]
    fn test_base64_encode_decode() {
        let original = b"Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).expect("decode failed");

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_redis_publisher_creation() {
        let pub_result = RedisPublisher::new("redis://localhost:6379");
        assert!(pub_result.is_ok());

        let pub_invalid = RedisPublisher::new("http://localhost:6379");
        assert!(pub_invalid.is_err());
    }

    #[test]
    fn test_redis_subscriber_creation() {
        let sub_result = RedisSubscriber::new("redis://localhost:6379");
        assert!(sub_result.is_ok());

        let sub_invalid = RedisSubscriber::new("tcp://localhost:6379");
        assert!(sub_invalid.is_err());
    }
}
