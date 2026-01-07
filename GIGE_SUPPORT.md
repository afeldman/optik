# GigE Vision Support für RPi Kameras

## Warum GigE für RPi?

**Deine Frage**: "was darf man auch einen gige kannal für die rpi kamera aufmachen?"

GigE (Gigabit Ethernet) ist eine standardisierte Schnittstelle für professionelle Kameras. Für die RPi bietet GigE Support mehrere Vorteile:

### 1. **Remote Camera Access**
```
Local Network:
┌────────────┐          ┌──────────────────┐
│ Optik Core │ UDP 3956 │ Remote System    │
│ (GigE Srv) │────────→ │ (GigE Client)    │
└────────────┘          └──────────────────┘
```

- **Dezentralisierte Verarbeitung**: RPi kann lokal Bilder erfassen
- **Remote Monitoring**: Andere Systeme empfangen über Netzwerk
- **Bandbreite-Optimierung**: Nur interessante Frames senden (mit lokalem Processing)

### 2. **Multi-Kamera Setups**
```
Local Network:
┌────────────┐
│ RPi 1      │
│ (Camera 0) │  GigE Port 3956
│ GigE Srv   │──────────────────┐
└────────────┘                  │
                                │
┌────────────┐                  ├──→ ┌──────────────────┐
│ RPi 2      │  GigE Port 3957  │    │ Central Host    │
│ (Camera 1) │──────────────────┤    │ (Inference)     │
│ GigE Srv   │                  │    └──────────────────┘
└────────────┘                  │
                                │
┌────────────┐                  │
│ RPi 3      │  GigE Port 3958  │
│ (Camera 2) │──────────────────┘
│ GigE Srv   │
└────────────┘
```

### 3. **Standardisierung**
- GigE Vision ist ein **EMVA Standard 1288** (European Machine Vision Association)
- Kompatibel mit Industrial Vision Software
- Bekannte Port & Protokoll

---

## Implementierung in Optik

### Server-Modus (RPi sendet Frames)

```rust
// In gige.rs - GigeServer
pub struct GigeServer {
    config: GigeConfig,
    socket: Option<UdpSocket>,
}

impl GigeServer {
    pub fn bind(&mut self) -> Result<()> {
        // Bindet auf 0.0.0.0:3956
        // Empfängt GigE GVCP Commands
    }
    
    pub fn send_frame(&self, frame_data: &[u8], remote_addr: &str) -> Result<()> {
        // Sendet Frames über UDP an Remote-Client
    }
}
```

**Python Usage**:
```python
from optik.gige import GigeServer
from optik._core import PyCamera

server = GigeServer()
server.bind()  # Port 3956

cam = PyCamera("rpi", 0)
cam.open()

while True:
    frame = cam.grab_frame()
    # Sende an Remote-Client
    server.send_frame(frame.data(), "192.168.1.100")
```

### Client-Modus (Remote empfängt Frames)

```python
from optik.gige import GigeClient

client = GigeClient("192.168.1.50", 3956)  # RPi Camera
client.connect()

# Receive frames
buffer = bytearray(4056 * 3040 * 3)  # 12MP * 3 bytes
n = client.receive_frame(buffer)
```

### Discovery (Geräte-Scanning)

```python
from optik.gige import GigeDiscovery

discovery = GigeDiscovery()
devices = discovery.discover()  
# Findet alle GigE Kameras im Netzwerk
# (Implementation mit GVCP Broadcast pending)
```

---

## Protokoll-Details

### GigE Vision Packet Format

```
┌─────────────────────────────────────────┐
│ IP Header (IPv4)                        │
├─────────────────────────────────────────┤
│ UDP Header                              │
│ Destination Port: 3956 (GVCP/IEVP)     │
├─────────────────────────────────────────┤
│ GigE Vision Header                      │
│ - Frame ID                              │
│ - Timestamp                             │
│ - Payload Type (Image Data)             │
├─────────────────────────────────────────┤
│ Image Data                              │
│ - Raw Bytes oder Compressed             │
├─────────────────────────────────────────┤
│ Trailer                                 │
│ - Checksum                              │
│ - Status                                │
└─────────────────────────────────────────┘
```

### Port Zuordnung

| Port | Protokoll | Zweck |
|------|-----------|-------|
| 3956 | UDP | GigE Vision Camera (GVCP) |
| 3957 | UDP | Alternate Camera (GigE 2) |
| 3958+ | UDP | Multi-Camera Setup |

---

## Performance-Optimierung

### 1. **Frame Streaming** (aktuell implementiert)
```rust
// UDP-basiert, connectionless
// Pro: Einfach, Low-Latency
// Con: Keine Fehlerkorrekktion
```

### 2. **Chunked Transfer** (future)
```rust
// Große Frames in mehrere Pakete splitten
// Mit Sequence-Nummern für Reassembly
```

### 3. **Lossless Compression** (future)
```rust
// H264 / H265 Encoding
// Nur Key-Frames auf Demand
// Bandwidth-Spar
```

---

## Netzwerk-Anforderungen

### Empfohlene Setup

```
RPi ────── Gigabit Switch ────── Central Host
   │                              │
   └── 100+ Mbps                 /
       (3956 UDP)                
                            
Empfohlen:
- Gigabit LAN (1000 Mbps)
- Dedizierte Network Interface für Video
- Jumbo Frames (MTU 9000) für große Bilder
```

### Bandbreite-Berechnung

```
RPi 12MP (4056x3040) @ 30 FPS @ 100% Quality

= 4056 × 3040 × 3 bytes/frame × 30 frames/sec
= 1,111,261,760 bytes/sec
= ~1.1 GB/sec

⚠️  Exceeds Gigabit limit!

Lösungen:
1. Reduziere FPS: 30 → 10 FPS = 370 MB/sec ✅
2. Kompression: H264 @ 30 FPS = 50-100 MB/sec ✅
3. Resolution: 4056 → 2048 @ 30 FPS = 280 MB/sec ✅
```

---

## Konfiguration (config.example.toml)

```toml
[gige]
enabled = true
server_port = 3956
client_timeout_ms = 5000
max_connections = 4

# Multi-camera setup
[[gige.cameras]]
index = 0
port = 3956
host = "0.0.0.0"

[[gige.cameras]]
index = 1
port = 3957
host = "0.0.0.0"
```

---

## Häufige Fragen

### F: Ist GigE besser als USB?
**A**: 
- **USB**: Lokal schneller, aber Kabel-begrenzt (5m)
- **GigE**: Über Netzwerk (100m), aber höhere Latenz

### F: Kann man mehrere RPi-Kameras kombinieren?
**A**: Ja! Mit GigE Server können mehrere RPis unterschiedliche Kameras hosten:
```python
# RPi 1 (Port 3956): Camera 0
# RPi 2 (Port 3957): Camera 1
# RPi 3 (Port 3958): Camera 2
```

### F: Funktioniert über Internet?
**A**: Theoretisch ja, praktisch nein:
- Latenz zu hoch für Live-Video
- Bandbreite limitiert
- Firewall-Probleme (UDP 3956)

Für WAN: Nutze komprimierte Streams (RTMP/HLS statt GigE)

---

## Weitere Ressourcen

- [EMVA GigE Vision Standard](https://www.emva.org/standards-technology/genicam/)
- [GigE Vision Spec Sheet](https://www.emva.org/wp-content/uploads/2022/06/EMVA1288.pdf)
- [Aravis (Open Source GigE Library)](https://github.com/AravisProject/aravis)

---

## Roadmap

- [ ] **v0.2.0**: Implementiere GVCP Discovery
- [ ] **v0.3.0**: UDP Chunked Transfer für >1GB/sec Bilder  
- [ ] **v0.4.0**: H264 Hardware-Encoding auf RPi
- [ ] **v0.5.0**: GigE Vision Protocol vollständig (GenICam)

