# optik Development Guide

## Project Structure

```
optik/
├── src/
│   ├── lib.rs              # Rust core (Maturin)
│   └── python/
│       └── optik/
│           ├── __init__.py
│           ├── exceptions.py
│           ├── camera.py      # BaslerCamera, IDSCamera
│           ├── controller.py   # Controllers for discovery
│           ├── config.py       # Configuration
│           ├── cli.py          # CLI
│           └── mux/
│               ├── server.py   # CBOR Mux Server
│               └── client.py   # Mux Client
├── tests/
├── Cargo.toml             # Rust dependencies
├── pyproject.toml         # Python project metadata
└── Makefile              # Build targets
```

## Architecture Overview

### Camera Abstraction

- **Camera (ABC)**: Abstract base class with common interface
  - `open()`, `close()`, `grab_frame()`
  - `set_exposure()`, `set_gain()`, `set_pixel_format()`
  - Thread-safe operations with `safe_get_image()`

- **BaslerCamera**: Basler (pypylon) implementation
- **IDSCamera**: IDS (ids_peak) implementation

### Controller Pattern

- **Controller (ABC)**: Manages multiple cameras
  - `discover()`: Find available devices
  - `open_device(index)`: Open specific camera
  - Context manager support

- **BaslerController**: Basler camera discovery
- **IDSController**: IDS camera discovery
- **MultiController**: Multi-vendor coordination

### Multiplexer Architecture

- **MuxServer**: CBOR-encoded messages over NNG
  - REQ/REP pattern (request/reply)
  - Single TCP port for all cameras
  - Commands: list, ping, get, set

- **MuxClient**: Python client for MuxServer
  - Simple async interface
  - Automatic frame reconstruction

### Rust Core (Maturin)

- `FrameBuffer`: Optimized frame buffer management
- `FrameMetadata`: Frame information
- Future: GPU acceleration, advanced image processing

## Development Workflow

### 1. Setting up Development Environment

```bash
# Clone and install
git clone https://github.com/your-org/optik
cd optik

# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install in dev mode
pip install -e ".[dev]"

# Install Rust development tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Running Tests

```bash
# All tests (without hardware)
pytest

# Specific test file
pytest tests/test_camera.py -v

# With coverage
pytest --cov=optik tests/
```

### 3. Code Quality

```bash
# Format code
black src/python tests
ruff check --fix src/python tests

# Type checking
mypy src/python

# Lint
ruff check src/python tests
```

### 4. Building Rust Extension

```bash
# Development build
maturin develop

# Release build
maturin build --release
```

## Adding New Features

### Adding a New Camera Vendor

1. Create new camera class in `camera.py`:

```python
class NewVendorCamera(Camera):
    def __init__(self, serial: str, index: int = 0):
        super().__init__(serial, "NewVendor")
        self.index = index
    
    def open(self) -> None:
        # Implementation
        pass
    
    # Implement other abstract methods...
```

2. Create controller in `controller.py`:

```python
class NewVendorController(Controller):
    def discover(self) -> List[DeviceInfo]:
        # Discovery implementation
        pass
    
    def open_device(self, index: int) -> Camera:
        # Open implementation
        pass
```

3. Integrate into `MultiController`:

```python
class MultiController(Controller):
    def __init__(self):
        super().__init__()
        self._new_vendor_ctrl = NewVendorController()
    
    def discover(self) -> List[Camera]:
        # Add to discovery loop
        pass
```

4. Add tests in `tests/test_camera.py`

### Adding Rust Optimization

1. Add Rust function in `src/lib.rs`:

```rust
#[pyfunction]
fn optimized_image_processing(image: &[u8]) -> Vec<u8> {
    // Rust implementation
}
```

2. Export from `__init__.py`:

```python
from . import _core
_core.optimized_image_processing
```

## Performance Considerations

### Frame Grabbing

- Use `safe_get_image()` for thread-safe access
- Frame buffers are managed by Rust for efficiency
- Mux server handles concurrent requests

### Memory Management

- Numpy arrays are shared with Python's reference counting
- Rust buffers are freed on image release
- Consider pre-allocated buffers for high-frequency capture

### Network (Mux)

- NNG handles efficient message passing
- CBOR encoding is compact and fast
- Single-threaded loop can handle ~1000 req/sec

## Debugging

### Enable Logging

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

### Inspect Camera State

```bash
optik info --camera 0
optik ping --camera 0
```

### Debug Mux Server

```bash
# Start with verbose output
optik mux-server --port 5555  # Logs to stderr
```

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Make changes and add tests
4. Run tests and linting: `make test lint`
5. Commit: `git commit -m 'Add amazing feature'`
6. Push: `git push origin feature/amazing-feature`
7. Open pull request

## License

Apache License 2.0 - See [LICENSE](../LICENSE)
