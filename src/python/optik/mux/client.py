"""
CBOR Multiplexer Client
"""

from typing import Dict, Optional, Any
import io

import pynng
import cbor2
import numpy as np
from loguru import logger

from ..exceptions import OptikError


class MuxClient:
    """Client for communicating with MuxServer"""

    def __init__(self, host: str = "127.0.0.1", port: int = 5555):
        self.endpoint = f"tcp://{host}:{port}"
        self._socket = None

    def __enter__(self):
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def connect(self) -> None:
        """Connect to server"""
        self._socket = pynng.Req0(dial=self.endpoint, recv_timeout=5000)
        logger.info(f"Connected to {self.endpoint}")

    def close(self) -> None:
        """Close connection"""
        if self._socket:
            self._socket.close()

    def list_cameras(self) -> Dict[str, Any]:
        """List available cameras"""
        response = self._send_request({"cmd": "list"})
        return response

    def ping(self, index: Optional[int] = None) -> Dict[str, Any]:
        """Ping camera or server"""
        response = self._send_request({"cmd": "ping", "index": index})
        return response

    def get_frame(self, index: int) -> Optional[np.ndarray]:
        """Get frame from camera"""
        response = self._send_request({"cmd": "get", "index": index})

        if "error" in response:
            raise OptikError(response["error"])

        # Reconstruct image from bytes
        data = response.get("data")
        width = response.get("width")
        height = response.get("height")
        channels = response.get("channels")

        if data is None or width is None or height is None:
            raise OptikError("Invalid frame response")

        if isinstance(data, (bytes, bytearray)):
            if channels == 1:
                frame = np.frombuffer(data, dtype=np.uint8).reshape(height, width)
            else:
                frame = (
                    np.frombuffer(data, dtype=np.uint8)
                    .reshape(height, width, channels)
                )
            return frame
        else:
            raise OptikError("Invalid frame data")

    def set_exposure(self, index: int, exposure_us: float) -> None:
        """Set camera exposure"""
        response = self._send_request(
            {"cmd": "set", "index": index, "param": "exposure", "value": exposure_us}
        )

        if "error" in response:
            raise OptikError(response["error"])

    def set_gain(self, index: int, gain: float) -> None:
        """Set camera gain"""
        response = self._send_request(
            {"cmd": "set", "index": index, "param": "gain", "value": gain}
        )

        if "error" in response:
            raise OptikError(response["error"])

    def _send_request(self, request: Dict) -> Dict:
        """Send request and get response"""
        if self._socket is None:
            raise OptikError("Not connected")

        try:
            msg = cbor2.dumps(request)
            self._socket.send(msg)
            response = self._socket.recv()
            return cbor2.loads(response)

        except pynng.Timeout:
            raise OptikError("Request timeout")
        except Exception as e:
            raise OptikError(f"Request failed: {e}")
