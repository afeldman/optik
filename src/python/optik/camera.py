"""
Camera implementations for Raspberry Pi and compatible cameras
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional, Dict, Any
import threading
import time

import numpy as np
from loguru import logger

from .exceptions import (
    OptikError,
    FeatureNotAvailableError,
    CameraConnectionError,
    FrameGrabError,
)


@dataclass
class FrameInfo:
    """Information about a captured frame"""
    timestamp: int  # microseconds
    sequence: int
    width: int
    height: int
    pixel_format: str
    exposure_us: float
    gain: float
    data: Optional[np.ndarray] = None


class Camera(ABC):
    """Abstract base class for camera implementations"""

    def __init__(self, serial: str, vendor: str):
        self.serial = serial
        self.vendor = vendor
        self._is_open = False
        self._lock = threading.RLock()
        self._last_frame: Optional[FrameInfo] = None
        self._features: Dict[str, Any] = {}

    @abstractmethod
    def open(self) -> None:
        """Open connection to camera"""
        pass

    @abstractmethod
    def close(self) -> None:
        """Close connection to camera"""
        pass

    @abstractmethod
    def grab_frame(self) -> FrameInfo:
        """Grab a single frame from camera"""
        pass

    @abstractmethod
    def set_exposure(self, exposure_us: float) -> None:
        """Set exposure time in microseconds"""
        pass

    @abstractmethod
    def get_exposure(self) -> float:
        """Get current exposure time in microseconds"""
        pass

    @abstractmethod
    def set_gain(self, gain: float) -> None:
        """Set gain in dB"""
        pass

    @abstractmethod
    def get_gain(self) -> float:
        """Get current gain in dB"""
        pass

    @abstractmethod
    def set_pixel_format(self, format: str) -> None:
        """Set pixel format (e.g., 'RGB8', 'Mono8')"""
        pass

    @abstractmethod
    def get_pixel_format(self) -> str:
        """Get current pixel format"""
        pass

    def is_open(self) -> bool:
        """Check if camera is open"""
        with self._lock:
            return self._is_open

    def safe_get_image(self) -> Optional[np.ndarray]:
        """Thread-safe image grab"""
        with self._lock:
            try:
                frame = self.grab_frame()
                self._last_frame = frame
                return frame.data
            except Exception as e:
                logger.error(f"Failed to grab frame from {self.serial}: {e}")
                return None

    def __enter__(self):
        self.open()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.is_open():
            self.close()


class RPiCamera(Camera):
    """Raspberry Pi camera implementation using picamera2"""

    def __init__(self, serial: str, index: int = 0):
        super().__init__(serial, "RPi")
        self.index = index
        self._camera = None
        self._picam2 = None
        self._config = None

    def open(self) -> None:
        """Open RPi camera"""
        with self._lock:
            if self._is_open:
                return

            try:
                from picamera2 import Picamera2

                self._picam2 = Picamera2(self.index)
                config = self._picam2.create_preview_configuration()
                self._picam2.configure(config)
                self._picam2.start()
                
                self._is_open = True
                logger.info(f"Opened RPi camera: {self.serial}")

            except Exception as e:
                raise CameraConnectionError(f"Failed to open RPi camera: {e}")

    def close(self) -> None:
        """Close RPi camera"""
        with self._lock:
            if not self._is_open:
                return

            try:
                if self._picam2 is not None:
                    self._picam2.stop()
                    self._picam2.close()
                self._is_open = False
                logger.info(f"Closed RPi camera: {self.serial}")
            except Exception as e:
                logger.error(f"Error closing RPi camera: {e}")

    def grab_frame(self) -> FrameInfo:
        """Grab frame from RPi camera"""
        if not self._is_open:
            raise OptikError("Camera not open")

        try:
            request = self._picam2.capture_request()
            
            # Get frame data
            buffer = request.make_buffer(0)
            cfg = request.get_metadata("ScalerCrop")
            
            # Get image array
            img_array = np.asarray(buffer)
            
            # Get sensor metadata
            timestamp = request.get_metadata("SensorTimestamp", 0)
            exposure_us = request.get_metadata("ExposureTime", 0)
            analog_gain = request.get_metadata("AnalogueGain", 1.0)
            
            request.release()

            return FrameInfo(
                timestamp=int(timestamp),
                sequence=0,
                width=self._picam2.camera_properties["PixelArraySize"][0],
                height=self._picam2.camera_properties["PixelArraySize"][1],
                pixel_format="RGB8",
                exposure_us=float(exposure_us),
                gain=20 * np.log10(analog_gain),  # Convert to dB
                data=img_array,
            )

        except Exception as e:
            raise FrameGrabError(f"RPi frame grab failed: {e}")

    def set_exposure(self, exposure_us: float) -> None:
        """Set exposure in microseconds"""
        with self._lock:
            try:
                ctrls = self._picam2.camera_controls
                ctrls["ExposureTime"] = int(exposure_us)
                self._picam2.set_controls(ctrls)
            except Exception as e:
                raise FeatureNotAvailableError(f"Cannot set exposure: {e}")

    def get_exposure(self) -> float:
        """Get current exposure"""
        try:
            md = self._picam2.capture_metadata()
            return float(md.get("ExposureTime", 0))
        except Exception:
            return 0.0

    def set_gain(self, gain: float) -> None:
        """Set gain in dB"""
        with self._lock:
            try:
                # Convert dB to linear gain
                linear_gain = 10 ** (gain / 20)
                ctrls = self._picam2.camera_controls
                ctrls["AnalogueGain"] = linear_gain
                self._picam2.set_controls(ctrls)
            except Exception as e:
                raise FeatureNotAvailableError(f"Cannot set gain: {e}")

    def get_gain(self) -> float:
        """Get current gain"""
        try:
            md = self._picam2.capture_metadata()
            analog_gain = md.get("AnalogueGain", 1.0)
            return 20 * np.log10(analog_gain)
        except Exception:
            return 0.0

    def set_pixel_format(self, format: str) -> None:
        """Set pixel format"""
        # RPi camera format handling
        with self._lock:
            try:
                config = self._picam2.create_preview_configuration()
                # Format wird über config gesetzt
                self._picam2.configure(config)
            except Exception as e:
                raise FeatureNotAvailableError(f"Cannot set pixel format: {e}")

    def get_pixel_format(self) -> str:
        """Get pixel format"""
        try:
            cfg = self._picam2.camera_config
            fmt = cfg.get("format", "RGB8")
            return str(fmt)
        except Exception:
            return "RGB8"


class IDSCamera(Camera):
    """Legacy IDS camera (fallback for older systems)"""

    def __init__(self, serial: str, index: int = 0):
        super().__init__(serial, "IDS-Legacy")
        self.index = index
        self._camera = None

    def open(self) -> None:
        """IDS not supported in this version"""
        raise CameraConnectionError("IDS cameras not supported. Use RPi Camera instead.")
