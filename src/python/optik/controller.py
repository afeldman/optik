"""
Camera controller implementations for Raspberry Pi
"""

from abc import ABC, abstractmethod
from typing import List, Dict, Optional
from dataclasses import dataclass
import threading

from loguru import logger

from .camera import Camera, RPiCamera, IDSCamera
from .exceptions import OptikError, CameraNotFoundError


@dataclass
class DeviceInfo:
    """Information about a discovered device"""
    serial: str
    vendor: str
    name: str
    model: str
    index: int


class Controller(ABC):
    """Abstract base class for camera controllers"""

    def __init__(self):
        self._devices: List[Camera] = []
        self._device_map: Dict[str, Camera] = {}
        self._lock = threading.RLock()

    @abstractmethod
    def discover(self) -> List[DeviceInfo]:
        """Discover available devices"""
        pass

    @abstractmethod
    def open_device(self, index: int) -> Camera:
        """Open device by index"""
        pass

    def open_device_by_serial(self, serial: str) -> Camera:
        """Open device by serial number"""
        with self._lock:
            if serial in self._device_map:
                return self._device_map[serial]
            raise CameraNotFoundError(f"Camera with serial {serial} not found")

    def close_all(self) -> None:
        """Close all open devices"""
        with self._lock:
            for device in self._devices:
                if device.is_open():
                    device.close()
            self._devices.clear()
            self._device_map.clear()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close_all()


class RPiController(Controller):
    """Raspberry Pi camera controller"""

    def discover(self) -> List[DeviceInfo]:
        """Discover RPi cameras"""
        try:
            from picamera2 import Picamera2

            device_infos = []
            
            # Try to enumerate available cameras
            try:
                for i in range(4):  # Support up to 4 cameras
                    try:
                        cam = Picamera2(i)
                        # Get camera properties
                        props = cam.camera_properties
                        model = props.get("Model", f"RPi Camera {i}")
                        
                        info = DeviceInfo(
                            serial=f"rpi_camera_{i}",
                            vendor="Raspberry Pi",
                            name=model,
                            model=model,
                            index=i,
                        )
                        device_infos.append(info)
                        logger.debug(f"Found RPi camera: {info.serial}")
                        cam.close()
                    except Exception:
                        # Camera not available at this index
                        break
            except Exception as e:
                logger.warning(f"Error enumerating cameras: {e}")

            if not device_infos:
                logger.warning("No RPi cameras found")

            return device_infos

        except ImportError:
            logger.error("picamera2 not installed")
            return []
        except Exception as e:
            logger.error(f"RPi discovery failed: {e}")
            return []

    def open_device(self, index: int) -> Camera:
        """Open RPi camera by index"""
        with self._lock:
            camera = RPiCamera(f"rpi_camera_{index}", index)
            camera.open()
            self._devices.append(camera)
            self._device_map[camera.serial] = camera
            return camera


class BaslerController(Controller):
    """Legacy Basler controller (deprecated)"""

    def discover(self) -> List[DeviceInfo]:
        """Basler not supported anymore - use RPi instead"""
        logger.warning("Basler support deprecated. Use RPi Camera instead.")
        return []

    def open_device(self, index: int) -> Camera:
        """Not supported"""
        raise OptikError("Basler cameras not supported. Use RPi Camera instead.")


class IDSController(Controller):
    """Legacy IDS controller (deprecated)"""

    def discover(self) -> List[DeviceInfo]:
        """IDS not supported anymore - use RPi instead"""
        logger.warning("IDS support deprecated. Use RPi Camera instead.")
        return []

    def open_device(self, index: int) -> Camera:
        """Not supported"""
        raise OptikError("IDS cameras not supported. Use RPi Camera instead.")


class MultiController(Controller):
    """Multi-vendor camera controller (RPi focused)"""

    def __init__(self):
        super().__init__()
        self._rpi_ctrl = RPiController()

    def discover(self) -> List[Camera]:
        """Discover all available cameras and return opened instances"""
        all_devices = []

        # Discover RPi cameras
        rpi_devices = self._rpi_ctrl.discover()
        for device in rpi_devices:
            try:
                camera = self._rpi_ctrl.open_device(device.index)
                all_devices.append(camera)
                logger.info(f"Opened RPi camera: {device.serial}")
            except Exception as e:
                logger.error(f"Failed to open RPi camera {device.index}: {e}")

        self._devices.extend(all_devices)
        for camera in all_devices:
            self._device_map[camera.serial] = camera

        return all_devices

    def open_device(self, index: int) -> Camera:
        """Not implemented for MultiController - use discover()"""
        raise NotImplementedError("Use discover() for MultiController")

    def close_all(self) -> None:
        """Close all open devices"""
        super().close_all()
        self._rpi_ctrl.close_all()
