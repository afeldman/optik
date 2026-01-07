use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use crate::Result;

/// GigE-Vision client for network cameras
/// This allows connecting to GigE cameras or GigE gateways
/// For RPi: Can stream via GigE to remote systems or act as GigE server

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigeConfig {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
}

impl Default for GigeConfig {
    fn default() -> Self {
        GigeConfig {
            host: "0.0.0.0".to_string(),
            port: 3956,  // Standard GigE-Vision port
            timeout_ms: 5000,
        }
    }
}

pub struct GigeServer {
    config: GigeConfig,
    socket: Option<UdpSocket>,
}

impl GigeServer {
    pub fn new(config: GigeConfig) -> Self {
        GigeServer {
            config,
            socket: None,
        }
    }

    pub fn bind(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let socket = UdpSocket::bind(&addr)
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        socket.set_read_timeout(Some(std::time::Duration::from_millis(self.config.timeout_ms)))
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        self.socket = Some(socket);
        tracing::info!("GigE server bound to {}", addr);
        Ok(())
    }

    pub fn send_frame(&self, frame_data: &[u8], remote_addr: &str) -> Result<()> {
        let socket = self.socket.as_ref()
            .ok_or(crate::OptikError::ConfigError("Socket not bound".to_string()))?;
        
        socket.send_to(frame_data, remote_addr)
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        Ok(())
    }
}

pub struct GigeClient {
    config: GigeConfig,
    socket: Option<UdpSocket>,
}

impl GigeClient {
    pub fn new(host: &str, port: u16) -> Self {
        GigeClient {
            config: GigeConfig {
                host: host.to_string(),
                port,
                timeout_ms: 5000,
            },
            socket: None,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        socket.set_read_timeout(Some(std::time::Duration::from_millis(self.config.timeout_ms)))
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        let remote_addr = format!("{}:{}", self.config.host, self.config.port);
        socket.connect(&remote_addr)
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        self.socket = Some(socket);
        tracing::info!("GigE client connected to {}", remote_addr);
        Ok(())
    }

    pub fn receive_frame(&self, buffer: &mut [u8]) -> Result<usize> {
        let socket = self.socket.as_ref()
            .ok_or(crate::OptikError::ConfigError("Socket not connected".to_string()))?;
        
        let n = socket.recv(buffer)
            .map_err(|e| crate::OptikError::IoError(e))?;
        
        Ok(n)
    }
}

/// GigE Discovery using broadcast
pub struct GigeDiscovery {
    discovery_port: u16,
}

impl GigeDiscovery {
    pub fn new() -> Self {
        GigeDiscovery {
            discovery_port: 3956,
        }
    }

    pub fn discover(&self) -> Result<Vec<String>> {
        // Simplified discovery - would normally send GVCP discovery packets
        // Returns list of discovered device IPs
        tracing::info!("Starting GigE device discovery on port {}", self.discovery_port);
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gige_config_default() {
        let config = GigeConfig::default();
        assert_eq!(config.port, 3956);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_gige_server_creation() {
        let config = GigeConfig::default();
        let _server = GigeServer::new(config);
    }

    #[test]
    fn test_gige_client_creation() {
        let mut _client = GigeClient::new("192.168.1.100", 3956);
    }

    #[test]
    fn test_gige_discovery() {
        let discovery = GigeDiscovery::new();
        let devices = discovery.discover();
        assert!(devices.is_ok());
    }
}
