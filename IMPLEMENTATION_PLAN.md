# 6-Wochen Implementierungsplan für optik v0.2

**Status**: Phase 1 Completed ✅ | Phase 2 Ready  
**Start Date**: 7. Januar 2026  
**Target Completion**: ~18. Februar 2026 (6 Wochen mit 1 Dev)  
**Total Effort**: 30-40 Developer-Days  

---

## Übersicht: Gap Analysis

### Funktionsvergleich

| Feature | Reference Implementation | optik | Priority |
|---------|-----------------|-------|----------|
| Device Discovery | ✅ | ❌ | **P0** |
| Basler Camera | ✅ | ❌ | **P0** |
| IDS Camera | ✅ | ❌ | **P0** |
| NNG Multiplex | ✅ | ❌ | **P0** |
| Feature Registry | ✅ | ❌ | **P0** |
| GigE Vision | ❌ | ✅ | - |
| Shared Memory IPC | ❌ | ✅ | - |
| Async Multi-Camera | ❌ | ✅ | - |
| Image Encoding | ✅ | ❌ | **P1** |
| Configuration System | ✅ | ❌ | **P1** |
| Trigger Support | ✅ | ❌ | **P1** |
| Pixel Format | ✅ | ❌ | **P1** |
| Lock Timeout Utils | ❌ | ✅ | - |
| Tests (27+) | ❌ | ✅ | - |
| Redis Support | ✅ | ❌ | **P2** |

**Gesamte Gap**: 15 fehlende Features (5 P0, 5 P1, 2 P2 + 3 Quality)

---

## Phase-Übersicht

```
Week 1:    Phase 1 - Device Discovery Framework
Week 2-3:  Phase 2 - Basler/IDS Drivers  
Week 3:    Phase 3 - Configuration System
Week 4-5:  Phase 4 - NNG Multiplex Server
Week 5:    Phase 5 - Redis Alternative (optional)
Week 6:    Phase 6 - Quality & Polish
```

---

## Phase 1: Device Discovery Framework (4-5 Tage, 13 Tests) ✅ COMPLETED

**Completion Date**: 7. Januar 2026  
**Actual Effort**: ~2 hours  
**Tests Passed**: 17/17 (10 Controller, 5 Device, 3 Error)  
**Commits**: 2 (Phase 1 implementation + docs)

**Ziel**: Abstrakte Controller Trait für verschiedene Camera-Typen

### Deliverables
- [x] `src/controller.rs` - Controller Trait & Discovery
- [x] `src/device.rs` - Device Info struct
- [x] Mock Controller für Tests
- [x] 13 Unit Tests (actually 17 - expanded)
- [x] README aktualisiert

### Tasks

**1.1 Controller Trait Design** (1 Tag)
```rust
pub trait Controller: Send + Sync {
    fn discover_devices(&self) -> Result<Vec<DeviceInfo>>;
    fn open_camera(&self, device: &DeviceInfo) -> Result<Box<dyn Camera>>;
    fn controller_type(&self) -> ControllerType;
}

pub enum ControllerType {
    Basler,
    IDS,
    RPi,
    GigE,
}

pub struct DeviceInfo {
    pub device_id: String,
    pub model_name: String,
    pub serial_number: String,
    pub controller_type: ControllerType,
    pub available: bool,
}
```

**1.2 Device Discovery Integration** (1.5 Tage)
- Mock implementation für Testing
- Basler FFI Platzhalter (Trait-Implementierung ohne echte Basler-Calls)
- IDS FFI Platzhalter (Trait-Implementierung ohne echte IDS-Calls)
- Registry Pattern für multi-Controller
- Tests: device_info, discovery_mock, controller_registry

**1.3 Tests & Integration** (1.5 Tage)
- `test_controller_trait` (3 Tests)
- `test_device_discovery` (4 Tests)
- `test_device_registry` (3 Tests)
- `test_mock_camera_from_device` (3 Tests)
- Dokumentation: Controller API, Discovery Flow

### Success Criteria
- ✅ Controller Trait für alle Camera-Typen
- ✅ 13 Tests passing (100% pass rate)
- ✅ README Section: "Device Discovery Framework"
- ✅ Example: `examples/discovery.rs`

### FFI Strategy Decision Points
- [ ] Basler: C-FFI über pyo3 oder direkter Rust-Wrapper?
- [ ] IDS: C-FFI über pyo3 oder direkter Rust-Wrapper?
- [ ] Empfehlung: Zunächst Mock-Implementation, dann echte FFI später

---

## Phase 2: Basler & IDS Camera Drivers (7-9 Tage, 20 Tests) ✅ IN PROGRESS

**Start Date**: 7. Januar 2026  
**Actual Effort (so far)**: ~1.5 hours  
**Tests Passed**: 25/25 (9 Registry, 8 Basler, 8 IDS)  
**Commits**: 1 (Feature Registry + Basler/IDS)

**Ziel**: Produktions-ready Basler und IDS Implementierungen

### Deliverables
- [x] `src/feature_registry.rs` - Feature Discovery Pattern
- [x] `src/basler.rs` - Basler Camera Implementation
- [x] `src/ids.rs` - IDS Camera Implementation
- [x] 25 Unit Tests (exceeding 20)
- [ ] Python FFI Wrappers (src/python/optik/basler.py, ids.py) - TODO

### Tasks

**2.1 Basler Camera Driver** (4-5 Tage) ✅ COMPLETED
```rust
pub struct BaslerCamera {
    device_info: DeviceInfo,
    is_open: bool,
    exposure_us: f64,
    gain_db: f64,
    frame_counter: u64,
    features: FeatureRegistry,
}

impl Camera for BaslerCamera {
    fn grab_frame(&mut self) -> Result<Frame>;
    fn set_exposure(&mut self, us: f32) -> Result<()>;
    fn set_gain(&mut self, db: f32) -> Result<()>;
    fn get_feature(&self, name: &str) -> Result<FeatureValue>;
    fn set_feature(&mut self, name: &str, value: FeatureValue) -> Result<()>;
}
```
- Full Camera trait implementation ✅
- Feature Registry for dynamic Properties ✅
- Tests: open/close, grab_frame, exposure, gain, features (8/8) ✅
- Standard features: ExposureTime (10µs-10s), Gain (0-48dB), PixelFormat, Width, Height, SerialNumber

**2.2 IDS Camera Driver** (4-5 Tage) ✅ COMPLETED
```rust
pub struct IDSCamera {
    device_info: DeviceInfo,
    is_open: bool,
    exposure_us: f64,
    gain_db: f64,
    frame_counter: u64,
    features: FeatureRegistry,
}

impl Camera for IDSCamera {
    fn grab_frame(&mut self) -> Result<Frame>;
    fn set_exposure(&mut self, us: f32) -> Result<()>;
    fn set_gain(&mut self, db: f32) -> Result<()>;
    fn get_feature(&self, name: &str) -> Result<FeatureValue>;
    fn set_feature(&mut self, name: &str, value: FeatureValue) -> Result<()>;
}
```
- Full Camera trait implementation ✅
- IDS-specific features (higher gain: 0-96dB, TriggerMode) ✅
- Tests: open/close, grab_frame, exposure, gain, features (8/8) ✅
- Resolution: 2560x2048 (higher than Basler)

**2.3 Feature Registry Pattern** (2-3 Tage) ✅ COMPLETED
```rust
pub struct FeatureRegistry {
    features: Arc<parking_lot::RwLock<HashMap<String, FeatureDescriptor>>>,
}

pub enum FeatureValue { Integer, Float, Boolean, String, Enum }

pub struct FeatureConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub enum_values: Option<Vec<String>>,
}

impl FeatureRegistry {
    pub fn register(&self, descriptor: FeatureDescriptor);
    pub fn get(&self, name: &str) -> Result<FeatureDescriptor>;
    pub fn get_value(&self, name: &str) -> Result<FeatureValue>;
    pub fn set_value(&self, name: &str, value: FeatureValue) -> Result<()>;
    pub fn list(&self) -> Vec<String>;
    pub fn all(&self) -> Vec<FeatureDescriptor>;
}
```
- Type conversion: as_f64(), as_i64(), as_bool(), as_string() ✅
- Constraint validation for numeric & enum values ✅
- Builder pattern for feature registration ✅
- Tests: conversions, validation, get/set, readonly, constraints (9/9) ✅
- RwLock for concurrent access

### Success Criteria
- ✅ Basler Camera: Discover + Open + Grab + Features (8 Tests)
- ✅ IDS Camera: Discover + Open + Grab + Features (8 Tests)
- ✅ Feature Registry: Discovery + Get/Set + Constraints (9 Tests)
- ✅ All 25 tests passing
- [ ] Python FFI Wrappers (basler.py, ids.py) - TODO
- [ ] README: "Basler Support", "IDS Support" - TODO

---

## Phase 3: Configuration System (4-5 Tage, 8 Tests)

**Ziel**: Deklarative Config mit Validation und Atomic Application

### Deliverables
- [ ] `src/config.rs` - CameraConfig struct + Validation
- [ ] Config File Format (YAML/TOML)
- [ ] 8 Unit Tests
- [ ] `examples/config.yaml`

### Tasks

**3.1 Config Struct Design** (1.5 Tage)
```rust
#[derive(Serialize, Deserialize)]
pub struct CameraConfig {
    pub exposure_us: f64,
    pub gain_db: f64,
    pub pixel_format: PixelFormat,
    pub trigger_mode: TriggerMode,
    pub frame_rate: f64,
    pub offset_x: u32,
    pub offset_y: u32,
    pub width: u32,
    pub height: u32,
    pub features: HashMap<String, serde_json::Value>,
}

impl CameraConfig {
    pub fn validate(&self) -> Result<()>;
    pub fn apply_to_camera(&self, camera: &mut dyn Camera) -> Result<()>;
}
```

**3.2 Validators & Application** (2 Tage)
- Exposure Validator (min/max, device-specific)
- Gain Validator (min/max, log-scale)
- Pixel Format Validator (supported formats)
- Trigger Validator (exposure sync)
- Atomic apply_to_camera() (all-or-nothing)
- Tests: validate, apply, rollback-on-error (4 Tests)

**3.3 Config File Integration** (1 Tage)
- YAML/TOML Parser
- Environment Variable Substitution
- Config Profiles (capture, preview, high-speed)
- Tests: load, merge, env_vars (4 Tests)

### Success Criteria
- ✅ CameraConfig mit Validation
- ✅ 8 Tests passing
- ✅ examples/config.yaml
- ✅ Python API: `camera.apply_config(config)`

---

## Phase 4: NNG Multiplex Server (6-8 Tage, 21 Tests)

**Ziel**: Production-ready Network RPC Server für Multi-Camera

**Basis**: CBOR over NNG (ZeroMQ-like, aber einfacher)

### Deliverables
- [ ] `src/nng_server.rs` - NNG Server Implementation
- [ ] `src/cbor_protocol.rs` - CBOR RPC Protocol
- [ ] `src/encoder.rs` - Image Encoding (RAW, JPEG, PNG)
- [ ] 21 Unit Tests
- [ ] Python Client (src/python/optik/nng_client.py)

### Tasks

**4.1 NNG Server Architecture** (2-3 Tage)
```rust
pub struct NNGServer {
    socket: NNG Socket,
    cameras: Arc<DashMap<String, Arc<Mutex<Box<dyn Camera>>>>>,
    config: ServerConfig,
}

impl NNGServer {
    pub async fn handle_request(&mut self, req: CBORRequest) -> CBORResponse;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
}
```

**4.2 CBOR Protocol Definition** (2-3 Tage)
```rust
pub enum CBORRequest {
    List,
    Ping { camera_id: String },
    GetFrame { camera_id: String, timeout_ms: u32 },
    SetConfig { camera_id: String, config: CameraConfig },
    GetFeature { camera_id: String, name: String },
    SetFeature { camera_id: String, name: String, value: serde_json::Value },
    GetStats { camera_id: String },
}

pub enum CBORResponse {
    List(Vec<DeviceInfo>),
    Pong { latency_us: u32 },
    Frame { data: Vec<u8>, metadata: FrameMetadata },
    ConfigApplied,
    Feature { value: serde_json::Value },
    Stats { ... },
    Error { message: String, code: u32 },
}
```

**4.3 Image Encoding** (2 Tage)
```rust
pub enum Encoding { RAW, JPEG(u8), PNG }

pub struct Encoder {
    format: Encoding,
}

impl Encoder {
    pub fn encode(&self, frame: &Frame) -> Result<Vec<u8>>;
    pub fn encode_metadata(&self, frame: &Frame) -> Result<FrameMetadata>;
}
```
- RAW: Direct Memory Copy
- JPEG: JPEGTurbo Integration (fast)
- PNG: PNG for Lossless
- Tests: encode_raw, encode_jpeg, encode_png, size_comparison (4 Tests)

**4.4 Server Integration & Testing** (2-3 Tage)
- Async/await with Tokio
- Multi-camera Concurrency
- CBOR Serialization
- Error Handling & Recovery
- Tests: start/stop, request_response, concurrent_requests, timeout, encoding (13 Tests)

### Success Criteria
- ✅ NNG Server läuft auf Port 5000 (default)
- ✅ 21 Tests passing (Request/Response + Concurrency + Encoding)
- ✅ Python Client (src/python/optik/nng_client.py)
- ✅ README: "NNG Multiplex Server"
- ✅ Example: `examples/nng_server.rs`

---

## Phase 5: Redis Alternative (2-3 Tage, 5 Tests - OPTIONAL)

**Ziel**: Redis Pub/Sub für High-Speed Frame Streaming

### Deliverables
- [ ] `src/redis_server.rs` - Redis Server Integration
- [ ] 5 Unit Tests
- [ ] Python Redis Client (src/python/optik/redis_client.py)

### Tasks
- Redis RESP Protocol Implementation
- Frame Stream Publishing
- Subscriber Pattern
- Tests: publish, subscribe, stream_performance (5 Tests)

### Success Criteria
- ✅ Optional: nur wenn Zeit/Bedarf
- ✅ 5 Tests passing

---

## Phase 6: Quality & Polish (5-7 Tage)

**Ziel**: Production-ready Code, Docs, Benchmarks

### Deliverables
- [ ] Clippy clean (0 warnings)
- [ ] 90%+ Code Coverage
- [ ] Full Documentation (doctests)
- [ ] Performance Benchmarks
- [ ] CLI Extensions

### Tasks

**6.1 Code Quality** (2 Tage)
- `cargo clippy --all-targets` → 0 warnings
- `cargo fmt` → all consistent
- `cargo test --doc` → all doctests passing
- Coverage: `cargo tarpaulin` → 90%+
- Tests: run full test suite, stress tests

**6.2 Documentation** (2 Tage)
- Doctests in allen public APIs
- API Reference aktualisieren
- Architecture Guide (Controller → Device Discovery → Multi-Camera)
- Examples: discovery.rs, config.rs, nng_server.rs
- CHANGELOG aktualisieren

**6.3 Performance & CLI** (1-2 Tage)
- Benchmark-Suite: lock contention, frame latency, throughput
- CLI Commands erweitern:
  - `optik discover` - List all devices
  - `optik config <device>` - Configure camera
  - `optik benchmark <device>` - Performance test
  - `optik server` - Start NNG server
- README: "Performance" Section mit Benchmarks

### Success Criteria
- ✅ 0 Clippy Warnings
- ✅ 90%+ Coverage
- ✅ All Doctests passing
- ✅ Performance Benchmarks documented
- ✅ Enhanced CLI (4+ commands)

---

## Kritische Abhängigkeiten & Blockers

### FFI Strategy Decision (BLOCKER)

**Basler Option 1: Rust FFI Wrapper**
- Pros: Best performance, Type-safe
- Cons: Requires pylon SDK, complex build
- Timeline: +1 week learning curve

**Basler Option 2: Python Wrapper**
- Pros: Reuse pypylon, fast iteration
- Cons: Python overhead, GIL lock
- Timeline: Easy to start, ~2-3 days

**Basler Option 3: C FFI via pyo3**
- Pros: Hybrid approach
- Cons: Complex binding layer
- Timeline: Moderate, 3-4 days

**Recommendation**: **Start with Option 2** (Python Wrapper)
- Use pypylon for Basler discovery/control
- Rust interfaces via FFI
- Migrate to pure Rust in later version (v1.0)

**Same for IDS**: Start with Python wrapper (ids_peak), migrate later

### Development Environment

**Required**:
- Rust 1.70+
- Cargo 1.70+
- Tokio 1.40+
- pyo3 0.21+

**Optional**:
- Basler pylon SDK (for Phase 2 native support)
- IDS SDK (for Phase 2 native support)
- Docker (for testing without hardware)

---

## Success Metrics

### Per Phase
| Phase | Tests | Features | Timeline |
|-------|-------|----------|----------|
| 1 | 13 ✅ | Device Discovery | 4-5 days |
| 2 | 20 ✅ | Basler + IDS + Features | 7-9 days |
| 3 | 8 ✅ | Configuration | 4-5 days |
| 4 | 21 ✅ | NNG Server | 6-8 days |
| 5 | 5 ✅ | Redis (optional) | 2-3 days |
| 6 | - | Quality | 5-7 days |
| **TOTAL** | **67+** | **15 features** | **30-40 days** |

### Overall Project
- ✅ 67+ Unit Tests (currently 27, +40 new)
- ✅ All P0 features implemented
- ✅ 90%+ Code Coverage
- ✅ Zero Clippy Warnings
- ✅ Full Documentation
- ✅ Performance Benchmarks
- ✅ Production-ready CLI

---

## Rollout Strategy

### Version Releases
- **v0.2.0** (after Phase 1): Device Discovery Framework
- **v0.3.0** (after Phase 2): Basler + IDS Support
- **v0.4.0** (after Phase 3): Configuration System
- **v0.5.0** (after Phase 4): NNG Multiplex Server
- **v1.0.0** (after Phase 6): Production Release

### Publishing Timeline
- PyPI: Update after each major phase
- GitHub Releases: Tag after each phase complete
- Docs: Auto-deploy from main branch

---

## Next Steps

1. **IMMEDIATE** (Today):
   - [ ] Confirm FFI strategy (Option 2 recommended)
   - [ ] Phase 1 kickoff: Start src/controller.rs
   - [ ] Create issues on GitHub (13 tasks for Phase 1)

2. **Week 1**:
   - [ ] Controller Trait complete (1.1)
   - [ ] Device Discovery mock (1.2)
   - [ ] All 13 tests green (1.3)
   - [ ] PR ready for review

3. **Week 2**:
   - [ ] Phase 1 merged to main
   - [ ] Phase 2 kickoff: Basler FFI prep
   - [ ] First Basler camera tests

---

**Document Version**: 1.0  
**Last Updated**: 7. Januar 2026  
**Status**: Ready for Phase 1 Kickoff ✅

