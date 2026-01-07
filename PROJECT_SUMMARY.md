# optik Project Summary

## Projektübersicht

**optik** ist ein Kamera-Management-System für Raspberry Pi und kompatible Kameras, speziell optimiert für die 12MP Autofocus Camera Module.

### 🎯 Hauptmerkmale

1. **Raspberry Pi Fokus**: Native Unterstützung für RPi Camera Modules
2. **Hybrid-Architektur**: Python API mit Rust/Maturin Core für Performance
3. **Netzwerk-Multiplexer**: CBOR über NNG - ein TCP-Port für alle Kameras
4. **Einheitliche API**: Exposure, Gain, PixelFormat-Steuerung
5. **Thread-Safe**: Sichere nebenläufige Zugriffe
6. **CLI & Python API**: Fire-basierte Kommandozeile + Python-Module
7. **Apache 2.0 Lizenz**: Für kommerzielle Nutzung geeignet

### 📁 Projektstruktur

```
optik/
├── src/
│   ├── lib.rs                    # Rust Core (Maturin)
│   └── python/optik/
│       ├── __init__.py
│       ├── exceptions.py         # Fehlerbehandlung
│       ├── camera.py             # RPiCamera Implementation
│       ├── controller.py         # Discovery & Management
│       ├── config.py             # TOML-Konfiguration
│       ├── cli.py                # CLI
│       └── mux/
│           ├── server.py         # CBOR MUX Server
│           └── client.py         # Mux Client
├── tests/
│   ├── test_camera.py
│   └── test_controller.py
├── examples_basic.py             # Einfaches Beispiel
├── examples_mux.py               # MUX-Beispiel
├── Cargo.toml                    # Rust-Abhängigkeiten
├── pyproject.toml                # Python-Projekt
├── pytest.ini                    # Test-Konfiguration
├── Makefile                      # Build-Targets
├── setup.sh                      # Quick-Setup
└── README.md, QUICKSTART.md, DEVELOPMENT.md
```

### 🔄 Migration von Basler/IDS zu RPi

Das Projekt wurde migr von Universal-Unterstützung (Basler, IDS) zu Raspberry Pi fokussiert:

**Removed:**
- ❌ BaslerCamera / BaslerController
- ❌ IDSCamera / IDSController
- ❌ pypylon, ids-peak Dependencies

**Added:**
- ✅ RPiCamera Implementation
- ✅ RPiController für Discovery
- ✅ picamera2 Integration
- ✅ Libcamera Support

### 🚀 Schnelleinstieg

```bash
# 1. RPi vorbereiten
ssh pi@raspberrypi.local
sudo raspi-config  # Camera aktivieren

# 2. Setup
git clone ...
cd optik
./setup.sh

# 3. Kameras listen
optik list

# 4. Frame grabben
optik grab --camera 0 --output frame.png

# 5. Python API nutzen
from optik import MultiController

with MultiController() as ctrl:
    cameras = ctrl.discover()
    frame = cameras[0].safe_get_image()
```

### 📊 Architektur-Highlights

1. **RPiCamera Abstraction:**
   - Wrapper um picamera2
   - Thread-safe Frame Grabbing
   - Exposure, Gain, PixelFormat Control

2. **Discovery Pattern:**
   - RPiController für Kamera-Enumeration
   - MultiController für Erweiterbarkeit
   - DeviceInfo für Metadaten

3. **Multiplexer:**
   - CBOR-kodierte Messages
   - NNG für Networking
   - Single TCP Port

4. **Rust Optimization:**
   - FrameBuffer für Buffer-Management
   - Potential für GPU-Integration

### 🛠️ Dependencies

**Python (auf RPi OS):**
- picamera2>=0.3.0
- libcamera-py>=0.1.0
- pynng, cbor2, pydantic
- loguru, fire
- opencv-python (optional)

**Rust:**
- pyo3 (Python FFI)
- tokio (Async)
- ndarray (Arrays)

### 📝 Nächste Schritte

- ✅ RPi Camera Support
- ⚙️ Hardware-Tests auf echtem RPi
- ⚙️ GPIO-Integration (für externe Trigger, LEDs, etc.)
- ⚙️ Video-Recording
- ⚙️ Histogram/Statistics
- ⚙️ PyPI Release
- ⚙️ Docker Images

