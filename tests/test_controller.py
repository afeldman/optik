"""
Tests for controller implementations
"""

import pytest

from optik.controller import MultiController, RPiController
from optik.exceptions import CameraNotFoundError


class TestRPiController:
    """Test RPi controller"""

    def test_rpi_controller_init(self):
        """Test controller initialization"""
        ctrl = RPiController()
        assert ctrl is not None

    def test_rpi_discover(self):
        """Test camera discovery"""
        ctrl = RPiController()
        devices = ctrl.discover()
        assert isinstance(devices, list)


class TestMultiController:
    """Test Multi-vendor controller"""

    def test_multi_controller_init(self):
        """Test controller initialization"""
        ctrl = MultiController()
        assert ctrl is not None

    def test_multi_discover(self):
        """Test multi-vendor discovery"""
        ctrl = MultiController()
        with ctrl:
            devices = ctrl.discover()
            assert isinstance(devices, list)
