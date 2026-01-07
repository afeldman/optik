"""
Command-line interface for optik
"""

import sys
import json
from pathlib import Path
from typing import Optional

import fire
from loguru import logger

from .controller import RPiController, MultiController
from .exceptions import OptikError


class OptikCLI:
    """CLI for optik camera management"""

    def __init__(self, verbose: bool = False):
        """Initialize CLI"""
        if verbose:
            logger.enable("optik")
        else:
            logger.disable("optik")

    def list(self, vendor: Optional[str] = None) -> None:
        """List available cameras"""
        try:
            if vendor == "rpi":
                ctrl = RPiController()
            else:
                ctrl = MultiController()

            devices = ctrl.discover() if vendor else ctrl.discover()
            
            if isinstance(devices, list) and devices and hasattr(devices[0], 'serial'):
                # discover() returned DeviceInfo objects
                for device in devices:
                    print(
                        f"  {device.index}: {device.vendor:15} - "
                        f"{device.model:20} ({device.serial})"
                    )
            else:
                # MultiController discover() returned Camera objects
                for i, camera in enumerate(devices):
                    print(f"  {i}: {camera.vendor:15} - {camera.serial}")

            print(f"\nTotal: {len(devices)} cameras found")

        except Exception as e:
            logger.error(f"Failed to list cameras: {e}")
            sys.exit(1)

    def grab(
        self,
        camera: int = 0,
        output: str = "frame.png",
        vendor: Optional[str] = None,
    ) -> None:
        """Grab frame from camera"""
        try:
            import cv2

            if vendor == "rpi":
                ctrl = RPiController()
            else:
                ctrl = MultiController()

            with ctrl:
                if vendor:
                    cam = ctrl.open_device(camera)
                else:
                    devices = ctrl.discover()
                    if camera >= len(devices):
                        raise OptikError(f"Camera {camera} not found")
                    cam = devices[camera]

                frame = cam.safe_get_image()
                if frame is None:
                    raise OptikError("Failed to grab frame")

                cv2.imwrite(output, frame)
                print(f"Frame saved to {output}")
                print(
                    f"  Size: {frame.shape[1]}x{frame.shape[0]}, "
                    f"Channels: {frame.shape[2] if len(frame.shape) > 2 else 1}"
                )

        except Exception as e:
            logger.error(f"Failed to grab frame: {e}")
            sys.exit(1)

    def info(self, camera: int = 0, vendor: Optional[str] = None) -> None:
        """Get camera information"""
        try:
            if vendor == "rpi":
                ctrl = RPiController()
            else:
                ctrl = MultiController()

            with ctrl:
                if vendor:
                    cam = ctrl.open_device(camera)
                else:
                    devices = ctrl.discover()
                    if camera >= len(devices):
                        raise OptikError(f"Camera {camera} not found")
                    cam = devices[camera]

                print(f"Camera: {cam.serial}")
                print(f"Vendor: {cam.vendor}")
                print(f"Exposure: {cam.get_exposure():.2f} µs")
                print(f"Gain: {cam.get_gain():.2f} dB")
                print(f"Pixel Format: {cam.get_pixel_format()}")

        except Exception as e:
            logger.error(f"Failed to get camera info: {e}")
            sys.exit(1)

    def mux_server(self, port: int = 5555, bind: str = "127.0.0.1") -> None:
        """Start CBOR multiplexer server"""
        try:
            from .mux import MuxServer

            server = MuxServer(host=bind, port=port)
            print(f"Starting multiplexer on {bind}:{port}")
            server.run()

        except ImportError:
            logger.error("Mux module not available")
            sys.exit(1)
        except Exception as e:
            logger.error(f"Failed to start mux server: {e}")
            sys.exit(1)


def main():
    """Main entry point"""
    fire.Fire(OptikCLI)


if __name__ == "__main__":
    main()
