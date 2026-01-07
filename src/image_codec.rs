/// Image encoding utilities for different formats
///
/// Supports RAW, JPEG, and PNG encoding from frame data.
/// For simplicity, JPEG and PNG are stored as-is (in production would use real encoders).

use crate::error::{OptikError, Result};

/// Encode frame data to requested format
pub fn encode_frame(
    data: &[u8],
    _width: u32,
    _height: u32,
    format: crate::nng_rpc::ImageFormat,
) -> Result<Vec<u8>> {
    match format {
        crate::nng_rpc::ImageFormat::Raw => Ok(data.to_vec()),
        crate::nng_rpc::ImageFormat::JPEG => {
            // In production: use mozjpeg to encode
            // For now: return raw data with JPEG magic header
            let mut result = vec![0xFF, 0xD8]; // JPEG SOI
            result.extend_from_slice(data);
            result.extend_from_slice(&[0xFF, 0xD9]); // JPEG EOI
            Ok(result)
        }
        crate::nng_rpc::ImageFormat::PNG => {
            // In production: use image crate to encode
            // For now: return raw data with PNG magic header
            let mut result = vec![0x89, 0x50, 0x4E, 0x47]; // PNG signature
            result.extend_from_slice(data);
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_raw() {
        let data = vec![128u8; 1024];
        let encoded = encode_frame(&data, 32, 32, crate::nng_rpc::ImageFormat::Raw)
            .expect("raw encode failed");
        assert_eq!(encoded, data);
    }

    #[test]
    fn test_encode_jpeg_has_header() {
        let data = vec![100u8; 512];
        let encoded = encode_frame(&data, 32, 32, crate::nng_rpc::ImageFormat::JPEG)
            .expect("jpeg encode failed");
        // Should have JPEG SOI header
        assert_eq!(encoded[0], 0xFF);
        assert_eq!(encoded[1], 0xD8);
    }

    #[test]
    fn test_encode_png_has_header() {
        let data = vec![200u8; 512];
        let encoded = encode_frame(&data, 32, 32, crate::nng_rpc::ImageFormat::PNG)
            .expect("png encode failed");
        // Should have PNG signature
        assert_eq!(encoded[0], 0x89);
        assert_eq!(encoded[1], 0x50);
    }

    #[test]
    fn test_encode_preserves_raw() {
        let original = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let encoded = encode_frame(&original, 2, 4, crate::nng_rpc::ImageFormat::Raw)
            .expect("encode failed");
        assert_eq!(encoded, original);
    }

    #[test]
    fn test_encode_different_formats() {
        let data = vec![50u8; 256];
        
        let raw = encode_frame(&data, 16, 16, crate::nng_rpc::ImageFormat::Raw)
            .expect("raw failed");
        let jpeg = encode_frame(&data, 16, 16, crate::nng_rpc::ImageFormat::JPEG)
            .expect("jpeg failed");
        let png = encode_frame(&data, 16, 16, crate::nng_rpc::ImageFormat::PNG)
            .expect("png failed");

        // All should return something
        assert!(!raw.is_empty());
        assert!(!jpeg.is_empty());
        assert!(!png.is_empty());
        
        // Raw should be smallest (no header)
        assert_eq!(raw.len(), data.len());
    }
}

