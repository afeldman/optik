# optik 🎥

Hochperformanter Kamera-Manager für Raspberry Pi und kompatible Kameras mit Rust/Maturin- und Python-Bindings.

## 🚀 Features

- **Raspberry Pi Support**: Native Unterstützung für RPi Camera Module (12MP Autofocus, etc.)
- **High-Performance Core**: Rust mit Maturin für Buffer-Management und Frame-Verarbeitung
- **Pythonic API**: Einfache Python-Schnittstelle für Integration in bestehende Systeme
- **Netzwerk-Multiplexer**: CBOR über NNG - ein Port für alle Kameras
- **Thread-Safe Operations**: Sichere gleichzeitige Zugriffe auf Kameras
- **Feature-Registry**: Einheitliche Steuerung von Exposure, Gain, PixelFormat
- **Konfiguration**: TOML-basierte Konfigurationen
- **Async Support**: Tokio-Integration für moderne asynchrone Operationen

## 📋 Anforderungen

- Python 3.10+ (Raspberry Pi OS oder Debian)
- Rust 1.70+ (für Development)
- RPi Camera Module oder kompatible Kamera
- picamera2 (automatisch installiert)

## 🔧 Installation

```bash
# Von PyPI
pip install optik

# Von Quelle mit Development-Tools
git clone https://github.com/your-org/optik
cd optik
pip install -e ".[dev]"
```

### Auf Raspberry Pi:

```bash
# Stellen Sie sicher, dass die Kamera aktiviert ist
sudo raspi-config
# → Interfacing Options → Camera → Ja

# Installieren Sie optik
pip install optik
```

## 📖 Verwendung

### Python API

```python
from optik import MultiController

# Alle Kameras finden und öffnen
with MultiController() as ctrl:
    devices = ctrl.discover()
    print(f"Found {len(devices)} devices")
    
    # Auf Kameras zugreifen
    for device in devices:
        device.set_exposure(10000)  # 10ms
        device.set_gain(5)
        
        # Frame auslesen (thread-safe)
        frame = device.safe_get_image()
        if frame is not None:
            print(f"Got frame: {frame.shape}")
```

### CLI

```bash
# Verfügbare Kameras auflisten
optik list

# Frame von Kamera auslesen
optik grab --camera 0 --output frame.png

# Multiplexer-Server starten
optik mux-server --port 5555
```

## 🏗️ Architektur

```
optik/
├── src/
│   ├── python/          # Python-Quellcode
│   │   └── optik/
│   │       ├── camera/        # RPi Camera Implementierung
│   │       ├── controller/     # Discovery & Geräte-Verwaltung
│   │       ├── mux/           # CBOR-Multiplexer
│   │       └── cli.py         # Fire-basierte CLI
│   └── lib.rs           # Rust Core (Maturin)
├── tests/               # Tests (pytest, auch ohne Hardware)
├── Cargo.toml          # Rust-Abhängigkeiten
└── pyproject.toml      # Python-Abhängigkeiten
```

## 🧪 Tests

```bash
# Alle Tests ausführen
pytest

# Mit Coverage
pytest --cov=optik
```

## 📝 Lizenz

Apache License 2.0 - siehe [LICENSE](LICENSE)

## 🤝 Beitragen

Contributions sind willkommen! Bitte:
1. Ein Issue für Features/Bugs erstellen
2. Feature-Branch (`git checkout -b feature/AmazingFeature`)
3. Changes committen (`git commit -m 'Add AmazingFeature'`)
4. Auf Branch pushen (`git push origin feature/AmazingFeature`)
5. Pull Request öffnen
