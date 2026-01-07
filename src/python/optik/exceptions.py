"""
optik exceptions and error handling
"""


class OptikError(Exception):
    """Base exception for optik errors"""
    pass


class CameraNotFoundError(OptikError):
    """Raised when a camera is not found"""
    pass


class FeatureNotAvailableError(OptikError):
    """Raised when a camera feature is not available"""
    pass


class CameraConnectionError(OptikError):
    """Raised when unable to connect to camera"""
    pass


class FrameGrabError(OptikError):
    """Raised when frame grabbing fails"""
    pass


class ConfigurationError(OptikError):
    """Raised when configuration is invalid"""
    pass
