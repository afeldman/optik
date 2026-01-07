"""
Configuration management for optik
"""

from pathlib import Path
from typing import Any, Dict, Optional

import toml
from pydantic import BaseModel, Field
from loguru import logger


class CameraConfig(BaseModel):
    """Camera configuration"""
    enabled: bool = True
    pixel_format: str = "RGB8"
    exposure_us: float = 10000.0
    gain_db: float = 5.0


class MuxConfig(BaseModel):
    """Multiplexer configuration"""
    host: str = "0.0.0.0"
    port: int = 5555
    buffer_size: int = 10


class LoggingConfig(BaseModel):
    """Logging configuration"""
    level: str = "INFO"
    format: str = (
        "{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | {name}:{function}:{line} - {message}"
    )


class Config(BaseModel):
    """Main optik configuration"""
    cameras: Dict[str, CameraConfig] = Field(default_factory=dict)
    mux: MuxConfig = Field(default_factory=MuxConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)

    @classmethod
    def from_file(cls, path: Path) -> "Config":
        """Load configuration from TOML file"""
        if not path.exists():
            logger.warning(f"Config file not found: {path}")
            return cls()

        try:
            data = toml.load(path)
            return cls(**data)
        except Exception as e:
            logger.error(f"Failed to load config from {path}: {e}")
            raise

    def to_file(self, path: Path) -> None:
        """Save configuration to TOML file"""
        try:
            with open(path, "w") as f:
                toml.dump(self.dict(), f)
            logger.info(f"Config saved to {path}")
        except Exception as e:
            logger.error(f"Failed to save config to {path}: {e}")
            raise

    def get_camera_config(self, vendor: str) -> CameraConfig:
        """Get camera configuration by vendor"""
        return self.cameras.get(vendor, CameraConfig())
