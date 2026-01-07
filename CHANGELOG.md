# optik Changelog

## [0.1.0] - 2024-01-07

### Added
- Initial release
- Multi-vendor camera support (Basler, IDS)
- BaslerCamera and IDSCamera implementations
- Unified camera control API
  - Exposure, Gain, PixelFormat management
  - Thread-safe frame grabbing
- Multi-vendor controller (MultiController, BaslerController, IDSController)
- CBOR-based multiplexer server
  - Single TCP port for all cameras
  - REQ/REP protocol using NNG
  - Concurrent camera access
- MuxClient for remote camera control
- Python CLI (Fire-based)
- Rust/Maturin core for performance
  - FrameBuffer optimization
  - FrameMetadata structures
- Comprehensive Python API
- Full test suite (pytest)
- Documentation (README, QUICKSTART, DEVELOPMENT)
- Examples (basic discovery, mux client)

### Features
- 🎥 Multi-vendor camera support
- 🚀 High-performance Rust backend with Python bindings
- 🔌 Single network port for all cameras
- 🧵 Thread-safe operations
- ⚙️ Unified feature control API
- 📊 Frame buffer management
- 🔍 Device discovery
- 🌐 Network multiplexer

### Dependencies
- Python 3.10+
- pypylon (Basler)
- ids-peak (IDS)
- pynng (networking)
- cbor2 (serialization)
- loguru (logging)
- pydantic (config validation)
- fire (CLI)
- Rust 1.70+ (for development)
