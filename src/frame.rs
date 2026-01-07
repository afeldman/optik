/// Frame structure containing image data and metadata
#[derive(Clone, Debug)]
pub struct Frame {
    pub timestamp: u64,      // Microseconds since UNIX_EPOCH
    pub sequence: u64,       // Frame sequence number
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub exposure_us: f32,    // Exposure in microseconds
    pub gain: f32,           // Gain in dB
    pub data: Vec<u8>,       // Raw image data (RGB or Mono)
}

impl Frame {
    pub fn new(
        width: u32,
        height: u32,
        channels: u8,
        data: Vec<u8>,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        Frame {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            sequence: 0,
            width,
            height,
            channels,
            exposure_us: 0.0,
            gain: 0.0,
            data,
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn bytes_per_pixel(&self) -> usize {
        self.channels as usize
    }

    pub fn bytes_per_line(&self) -> usize {
        self.width as usize * self.bytes_per_pixel()
    }

    pub fn pixel_at(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let offset = (y * self.width + x) as usize * self.bytes_per_pixel();
        let end = offset + self.bytes_per_pixel();

        if end <= self.data.len() {
            Some(&self.data[offset..end])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let data = vec![0u8; 640 * 480 * 3];
        let frame = Frame::new(640, 480, 3, data);
        
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
        assert_eq!(frame.channels, 3);
        assert_eq!(frame.size(), 640 * 480 * 3);
    }

    #[test]
    fn test_frame_pixel_access() {
        let data = vec![255u8; 640 * 480 * 3];
        let frame = Frame::new(640, 480, 3, data);
        
        let pixel = frame.pixel_at(0, 0);
        assert!(pixel.is_some());
        assert_eq!(pixel.unwrap()[0], 255);
    }

    #[test]
    fn test_frame_out_of_bounds() {
        let data = vec![0u8; 640 * 480 * 3];
        let frame = Frame::new(640, 480, 3, data);
        
        assert!(frame.pixel_at(640, 0).is_none());
        assert!(frame.pixel_at(0, 480).is_none());
    }
}
