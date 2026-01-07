"""
optik - High-performance camera manager for Raspberry Pi and compatible cameras

Python API for controlling RPi cameras with Rust performance backend.
"""

__version__ = "0.1.0"

from .camera import RPiCamera, IDSCamera
from .controller import RPiController, MultiController
from .exceptions import OptikError, CameraNotFoundError, FeatureNotAvailableError

__all__ = [
    "RPiCamera",
    "IDSCamera",
    "RPiController",
    "MultiController",
    "OptikError",
    "CameraNotFoundError",
    "FeatureNotAvailableError",
]

try:
    from . import _core
except ImportError:
    _core = None
