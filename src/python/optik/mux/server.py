"""
CBOR Multiplexer Server - single port for all cameras
"""

import time
import threading
from typing import Dict, List, Optional

import pynng
import cbor2
from loguru import logger

from ..controller import MultiController
from ..exceptions import OptikError


class MuxServer:
    """CBOR multiplexer server for multiple cameras"""

    def __init__(self, host: str = "127.0.0.1", port: int = 5555):
        self.host = host
        self.port = port
        self.endpoint = f"tcp://{host}:{port}"
        self._running = False
        self._socket = None
        self._controller = None
        self._devices = []

    def run(self) -> None:
        """Run the multiplexer server"""
        try:
            self._setup_socket()
            self._discover_cameras()
            self._server_loop()

        except Exception as e:
            logger.error(f"Server error: {e}")
            raise
        finally:
            self._cleanup()

    def _setup_socket(self) -> None:
        """Setup NNG socket"""
        self._socket = pynng.Rep0(listen=self.endpoint)
        logger.info(f"Server listening on {self.endpoint}")

    def _discover_cameras(self) -> None:
        """Discover all available cameras"""
        self._controller = MultiController()
        self._devices = self._controller.discover()
        logger.info(f"Discovered {len(self._devices)} cameras")

    def _server_loop(self) -> None:
        """Main server loop"""
        self._running = True
        while self._running:
            try:
                # Receive request (blocking)
                msg = self._socket.recv()
                request = cbor2.loads(msg)

                # Process request
                response = self._handle_request(request)

                # Send response
                reply = cbor2.dumps(response)
                self._socket.send(reply)

            except Exception as e:
                logger.error(f"Request error: {e}")
                response = {"error": str(e)}
                reply = cbor2.dumps(response)
                try:
                    self._socket.send(reply)
                except Exception:
                    pass

    def _handle_request(self, request: Dict) -> Dict:
        """Handle incoming request"""
        cmd = request.get("cmd")

        if cmd == "list":
            return self._cmd_list()
        elif cmd == "ping":
            return self._cmd_ping(request.get("index"))
        elif cmd == "get":
            return self._cmd_get(request.get("index"))
        elif cmd == "set":
            return self._cmd_set(request)
        else:
            return {"error": f"Unknown command: {cmd}"}

    def _cmd_list(self) -> Dict:
        """List available cameras"""
        cameras = []
        for i, device in enumerate(self._devices):
            cameras.append(
                {
                    "index": i,
                    "serial": device.serial,
                    "vendor": device.vendor,
                }
            )
        return {"cameras": cameras, "count": len(cameras)}

    def _cmd_ping(self, index: Optional[int]) -> Dict:
        """Ping camera"""
        if index is None:
            return {"status": "pong"}

        if index >= len(self._devices):
            return {"error": f"Camera {index} not found"}

        try:
            device = self._devices[index]
            frame = device.safe_get_image()
            return {
                "status": "pong",
                "camera": device.serial,
                "got_frame": frame is not None,
            }
        except Exception as e:
            return {"error": str(e)}

    def _cmd_get(self, index: Optional[int]) -> Dict:
        """Get frame from camera"""
        if index is None:
            return {"error": "index required"}

        if index >= len(self._devices):
            return {"error": f"Camera {index} not found"}

        try:
            device = self._devices[index]
            frame = device.safe_get_image()

            if frame is None:
                return {"error": "Failed to grab frame"}

            # Convert frame to bytes for CBOR encoding
            return {
                "success": True,
                "camera": device.serial,
                "width": frame.shape[1],
                "height": frame.shape[0],
                "channels": frame.shape[2] if len(frame.shape) > 2 else 1,
                "data": frame.tobytes(),
            }

        except Exception as e:
            return {"error": str(e)}

    def _cmd_set(self, request: Dict) -> Dict:
        """Set camera parameters"""
        index = request.get("index")
        param = request.get("param")
        value = request.get("value")

        if index is None or index >= len(self._devices):
            return {"error": "Invalid camera index"}

        try:
            device = self._devices[index]

            if param == "exposure":
                device.set_exposure(float(value))
                return {"success": True, "param": param, "value": value}

            elif param == "gain":
                device.set_gain(float(value))
                return {"success": True, "param": param, "value": value}

            else:
                return {"error": f"Unknown parameter: {param}"}

        except Exception as e:
            return {"error": str(e)}

    def _cleanup(self) -> None:
        """Cleanup resources"""
        self._running = False
        if self._socket:
            self._socket.close()
        if self._controller:
            self._controller.close_all()
        logger.info("Server stopped")
