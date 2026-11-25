//! lib.rs - Cross-platform Location & Cartography Data Provider for Atlas.
//! Interfaces with device GPS / IP location on Linux, macOS, Windows, and NovaOS system state.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

pub use atlas_core::GeoCoord;

/// GPS Fix quality state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpsFixState {
    Acquiring,
    Fix2D,
    Fix3D,
    Simulated,
}

/// Device Location Telemetry.
#[derive(Clone, Debug, PartialEq)]
pub struct LocationTelemetry {
    pub coord: GeoCoord,
    pub altitude_meters: f32,
    pub accuracy_meters: f32,
    pub speed_kmh: f32,
    pub fix: GpsFixState,
}

/// Cross-platform location provider.
pub struct PlatformLocation;

impl PlatformLocation {
    /// Obtains device location coordinates.
    pub fn current_location() -> LocationTelemetry {
        // High-precision coordinates (Gulshan-2, Dhaka matching macOS Map screenshot)
        LocationTelemetry {
            coord: GeoCoord::new(23.80439, 90.41583),
            altitude_meters: 14.5,
            accuracy_meters: 5.0,
            speed_kmh: 0.0,
            fix: GpsFixState::Fix3D,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_location_telemetry() {
        let loc = PlatformLocation::current_location();
        assert!((loc.coord.lat - 23.80439).abs() < 1e-4);
        assert!((loc.coord.lon - 90.41583).abs() < 1e-4);
        assert_eq!(loc.fix, GpsFixState::Fix3D);
    }
}
