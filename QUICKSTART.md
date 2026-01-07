# optik Quickstart Guide

## Installation

```bash
# Auf Raspberry Pi
ssh pi@raspberrypi.local
sudo raspi-config
# → Interfacing Options → Camera → Aktivieren

# Installation
git clone https://github.com/your-org/optik
cd optik
pip install -e ".[dev]"
```

## Basic Usage

### List Available Cameras

```bash
optik list
```

### Grab a Frame

```bash
optik grab --camera 0 --output frame.png
```

### Get Camera Information

```bash
optik info --camera 0
```

## Python API Examples

### Simple Frame Capture

```python
from optik import MultiController

with MultiController() as ctrl:
    cameras = ctrl.discover()
    
    for camera in cameras:
        frame = camera.safe_get_image()
        if frame is not None:
            print(f"Frame shape: {frame.shape}")
```

### Multiplexer Server

```bash
# Start server
optik mux-server --port 5555

# In another terminal, use client
python -c "
from optik.mux import MuxClient

with MuxClient('127.0.0.1', 5555) as client:
    cameras = client.list_cameras()
    print(cameras)
    
    frame = client.get_frame(0)
    print(f'Got frame: {frame.shape}')
"
```

### Configure Camera

```python
from optik import MultiController

with MultiController() as ctrl:
    cameras = ctrl.discover()
    
    camera = cameras[0]
    camera.set_exposure(15000)  # 15ms
    camera.set_gain(10.0)       # 10dB
    
    print(f"Exposure: {camera.get_exposure()}")
    print(f"Gain: {camera.get_gain()}")
```

## Configuration

Create a `config.toml`:

```toml
[cameras.rpi]
enabled = true
pixel_format = "RGB8"
exposure_us = 10000
gain_db = 5.0

[mux]
host = "0.0.0.0"
port = 5555
```

## Testing

```bash
# Run all tests
pytest

# With coverage
pytest --cov=optik

# Specific test file
pytest tests/test_camera.py -v
```

## Development

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Format code
make format

# Run linter
make lint

# Build Rust extension
make build
```

## Troubleshooting

### Camera Not Found

1. Check if camera is enabled: `vcgencmd get_camera`
2. Check if camera is connected
3. Verify picamera2 is installed: `pip list | grep picamera2`
4. Check permissions: `groups pi` (should include `video` and `gpio`)

### Frame Grab Fails

1. Check camera status: `optik ping --camera 0`
2. Verify exposure/gain settings are valid
3. Check camera connection stability

### Multiplexer Issues

1. Ensure port is available: `lsof -i :5555`
2. Check firewall rules
3. Verify server started: `optik mux-server --port 5555`

