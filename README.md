# Atlas — Sovereign Vector Cartography & Navigation Engine

Atlas is a pure-Rust, cross-platform, GPU-independent vector maps and navigation engine. Built from the ground up to synthesize the fluid, elegant aesthetics of **macOS Maps** (acrylic floating sidebar, Find Nearby drawer, Marked Location inspector, weather widget) with sovereign bare-metal portability for **NovaOS ring-3**, **Linux/Ubuntu**, **Windows**, and **macOS**.

---

## 🌟 Architecture & Features

- **Mathematical Web Mercator Projection (EPSG:3857)**:
  - Exact forward spherical geodetic projection: $(\text{lat}, \text{lon}) \to (x, y)$ in $[0, 1]^2$.
  - Exact inverse hyperbolic projection: $(x, y) \to (\text{lat}, \text{lon})$.
  - Viewport zoom, camera panning, and screen projection matrices.
- **Geodesic Geometry**:
  - Haversine great-circle distance computation between arbitrary geodetic points.
  - Heading (bearing) calculation and compass orienting.
- **Spatial Vector Database**:
  - Real-world polygon geometries (Lakes, Parks, Reservoirs, Diplomatic Enclaves).
  - Multi-tier road networks (Expressways, Major Arteries, Secondary Avenues, Local Streets) with double-pass casing and infill rendering.
  - Multi-criteria POI directory with categorized pins, addresses, ratings, and phone numbers.
- **Turn-by-Turn Navigation Routing**:
  - Waypoint path planning with distance and estimated time calculation.
  - Step-by-step guidance maneuvers (depart, turn, arrive).
- **3D Perspective Horizon**:
  - Dynamic 2D/3D perspective pitch tilt (35° horizon tilt) with ground-plane depth projection.
- **macOS Maps Acrylic Shell Synthesis**:
  - Floating translucent acrylic sidebar with Search, Saved Places (Home, Work, Add), Guides, and Recents.
  - Find Nearby drawer with 11+ categories (Restaurants, Fast Food, Gas, Coffee, Hotels, Shopping, etc.).
  - Marked Location inspector card with distance, formatted coordinates, address, and Pin/Remove buttons.
  - Real-time weather and AQI telemetry badge (`25° 🌤️`).
- **Zero External Dependencies**:
  - Pure 2D software double-buffered vector rasterizer (`CanvasSurface`).
  - No libc required for core engine; supports `#![no_std]` bare-metal environments.

---

## 📂 Repository Structure

```
atlas/
├── crates/
│   ├── atlas-core/      # Geodesy, Mercator EPSG:3857, routing, and spatial database
│   ├── atlas-render/    # Pure 2D software vector rasterizer and 3D pitch projection
│   ├── atlas-ui/        # macOS Maps UI shell (floating sidebar, nearby drawer, HUD)
│   ├── atlas-platform/  # Cross-platform location telemetry & GPS fix providers
│   └── atlas-app/       # Autonomous qualification runner & cross-platform CLI
├── novaos/              # NovaOS bare-metal ring-3 port (ELF32) & fixture generator
└── Cargo.toml           # Multi-crate workspace configuration
```

---

## 🚀 Building & Testing

### Cross-Platform Host (Linux / macOS / Windows)
```bash
cargo test
cargo run -p atlas-app -- --selftest
cargo run -p atlas-app -- --poi
cargo run -p atlas-app -- --coords
```

### NovaOS Bare-Metal Ring-3
```bash
chmod +x novaos/build.sh
./novaos/build.sh
```

---

## 📜 License
MIT / Apache-2.0.
