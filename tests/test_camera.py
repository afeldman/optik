"""
Tests for camera implementations
"""

import pytest

from optik.exceptions import OptikError
from optik.camera import RPiCamera, IDSCamera, FrameInfo


class TestRPiCamera:
    """Test RPi camera implementation"""

    def test_rpi_camera_init(self):
        """Test camera initialization"""
        camera = RPiCamera("rpi_camera_0", index=0)
        assert camera.serial == "rpi_camera_0"
        assert camera.vendor == "RPi"
        assert not camera.is_open()

    def test_rpi_camera_not_open(self):
        """Test operations on closed camera"""
        camera = RPiCamera("rpi_camera_0", index=0)
        with pytest.raises(OptikError):
            camera.grab_frame()


class TestIDSCamera:
    """Test legacy IDS camera implementation"""

    def test_ids_camera_init(self):
        """Test camera initialization"""
        camera = IDSCamera("ids_legacy_0", index=0)
        assert camera.serial == "ids_legacy_0"
        assert camera.vendor == "IDS-Legacy"
        assert not camera.is_open()

    def test_ids_not_supported(self):
        """Test that IDS is deprecated"""
        camera = IDSCamera("ids_legacy_0", index=0)
        with pytest.raises(Exception):
            camera.open()


class TestFrameInfo:
    """Test FrameInfo dataclass"""

    def test_frame_info_creation(self):
        """Test creating frame info"""
        info = FrameInfo(
            timestamp=1000,
            sequence=1,
            width=640,
            height=480,
            pixel_format="RGB8",
            exposure_us=10000,
            gain=5.0,
        )
        assert info.width == 640
        assert info.height == 480
        assert info.pixel_format == "RGB8"
