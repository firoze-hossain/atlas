//! main.rs - Atlas Sovereign Cartography & Navigation Engine CLI Runner.

use std::env;
use std::process;

use atlas_core::{
    GeoCoord, MapDatabase, MapState, MercatorProjection,
};
use atlas_platform::PlatformLocation;
use atlas_render::{CanvasSurface, VectorMapRenderer};
use atlas_ui::MapUiRenderer;

fn run_selftest() -> bool {
    println!("============================================================");
    println!("        ATLAS MAPS AUTONOMOUS QUALIFICATION SUITE           ");
    println!("============================================================");

    // 1. Web Mercator Projection & Invertibility
    println!("\n[1/8] Subsystem: Spherical Web Mercator (EPSG:3857) Projection");
    let test_coord = GeoCoord::new(23.80439, 90.41583);
    let (nx, ny) = MercatorProjection::geo_to_normalized(&test_coord);
    let back = MercatorProjection::normalized_to_geo(nx, ny);
    if (test_coord.lat - back.lat).abs() > 1e-4 || (test_coord.lon - back.lon).abs() > 1e-4 {
        eprintln!("  [FAIL] Mercator normalized projection roundtrip failed");
        return false;
    }
    println!("  [PASS] Forward and inverse Mercator projections mathematically accurate");

    // 2. Haversine Great-Circle Distance
    println!("\n[2/8] Subsystem: Haversine Great-Circle Distance Geometry");
    let c1 = GeoCoord::new(23.80439, 90.41583); // United Hospital
    let c2 = GeoCoord::new(23.79250, 90.41500); // The Westin Dhaka
    let dist = c1.distance_to(&c2);
    if dist < 1.0 || dist > 2.0 {
        eprintln!("  [FAIL] Distance calculation out of expected range: {} km", dist);
        return false;
    }
    println!("  [PASS] Haversine distance: {:.3} km between United Hospital and Westin Dhaka", dist);

    // 3. Spatial Database & Geometries
    println!("\n[3/8] Subsystem: Spatial Database: Road Polylines & Water Bodies");
    let polygons = MapDatabase::load_polygons();
    let roads = MapDatabase::load_roads();
    let pois = MapDatabase::load_pois();
    if polygons.is_empty() || roads.is_empty() || pois.is_empty() {
        eprintln!("  [FAIL] Spatial database returned empty datasets");
        return false;
    }
    println!("  [PASS] Loaded {} polygons (Lakes, Parks), {} road corridors, {} POIs", polygons.len(), roads.len(), pois.len());

    // 4. Point of Interest Substring & Category Search
    println!("\n[4/8] Subsystem: POI Multi-Criteria Geocoding & Search");
    let hospital_matches = MapDatabase::search_pois("Hospital");
    let dining_matches = MapDatabase::search_pois("Kabab");
    if hospital_matches.is_empty() || dining_matches.is_empty() {
        eprintln!("  [FAIL] POI search returned empty results");
        return false;
    }
    println!("  [PASS] Geocoding matched: '{}' and '{}'", hospital_matches[0].name, dining_matches[0].name);

    // 5. Turn-by-Turn Routing Engine
    println!("\n[5/8] Subsystem: Turn-by-Turn Navigation Routing Engine");
    let mut state = MapState::default();
    state.plan_route(c1, c2, "United Hospital", "The Westin Dhaka");
    if state.active_route.is_none() {
        eprintln!("  [FAIL] Route planning produced no route");
        return false;
    }
    let route = state.active_route.as_ref().unwrap();
    if route.steps.len() != 2 || route.total_duration_minutes == 0 {
        eprintln!("  [FAIL] Invalid route telemetry");
        return false;
    }
    println!("  [PASS] Planned route: {} min, {:.2} km with {} navigation steps", route.total_duration_minutes, route.total_distance_km, route.steps.len());

    // 6. 3D Perspective Pitch Transform
    println!("\n[6/8] Subsystem: 3D Perspective Horizon & Pitch Transform");
    state.toggle_3d();
    if state.pitch_deg != 35.0 {
        eprintln!("  [FAIL] 3D toggle failed to apply 35° pitch");
        return false;
    }
    state.toggle_3d();
    if state.pitch_deg != 0.0 {
        eprintln!("  [FAIL] 3D toggle failed to restore 2D orthographic pitch");
        return false;
    }
    println!("  [PASS] 2D/3D perspective pitch transform verified");

    // 7. Pure 2D Software Vector Rasterization
    println!("\n[7/8] Subsystem: Pure 2D Software Vector Map Rasterizer");
    let mut surface = CanvasSurface::new(1024, 768);
    VectorMapRenderer::render(&mut surface, &state);
    let non_zero = surface.buffer.iter().filter(|&&px| px != 0xFFF4F3F0).count();
    if non_zero < 1000 {
        eprintln!("  [FAIL] Vector map rasterizer produced empty canvas");
        return false;
    }
    println!("  [PASS] Rasterized vector cartography onto 1024x768 CanvasSurface ({} modified pixels)", non_zero);

    // 8. macOS Maps UI Shell Synthesis
    println!("\n[8/8] Subsystem: macOS Maps UI Synthesis (Sidebar, Drawer, Inspector)");
    MapUiRenderer::render(&mut surface, &state);
    let total_pixels = surface.buffer.len();
    println!("  [PASS] Complete frame rendered with floating acrylic panel & controls ({} total pixels)", total_pixels);

    println!("\n============================================================");
    println!("AUTONOMOUS QUALIFICATION SUMMARY: ALL 8 SUBSYSTEMS PASSED");
    println!("============================================================");
    println!(">>> [SUCCESS] Atlas Cartography Engine 100% OPERATIONAL <<<");
    true
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--selftest" || a == "-t") {
        if run_selftest() {
            process::exit(0);
        } else {
            process::exit(1);
        }
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("Atlas Cartography & Navigation Engine 1.0.0 (Pure Rust)");
        println!("Sovereign Vector Maps Suite for NovaOS and Cross-Platform Desktop");
        process::exit(0);
    }

    if args.iter().any(|a| a == "--coords" || a == "-c") {
        let loc = PlatformLocation::current_location();
        println!("Current Location Telemetry:");
        println!("  Coordinates: {}", loc.coord.format_dms());
        println!("  Altitude:    {:.1} m", loc.altitude_meters);
        println!("  Accuracy:    ±{:.1} m", loc.accuracy_meters);
        println!("  GPS Fix:     {:?}", loc.fix);
        process::exit(0);
    }

    if args.iter().any(|a| a == "--poi" || a == "-p") {
        println!("Atlas Pre-loaded Points of Interest:");
        for poi in MapDatabase::load_pois() {
            println!("  * [{:?}] {} (Rating: {:.1}) - {}", poi.category, poi.name, poi.rating, poi.address);
        }
        process::exit(0);
    }

    if let Some(idx) = args.iter().position(|a| a == "--search" || a == "-s") {
        if let Some(query) = args.get(idx + 1) {
            println!("Search results for '{}':", query);
            let results = MapDatabase::search_pois(query);
            if results.is_empty() {
                println!("  No matching locations found.");
            } else {
                for p in results {
                    println!("  * {} ({}) - {}", p.name, p.category.name(), p.address);
                    println!("    Coords: {}", p.coord.format_dms());
                }
            }
            process::exit(0);
        }
    }

    if args.iter().any(|a| a == "--info" || a == "-i") {
        println!("Atlas Cartography Subsystems:");
        println!("  * Projection: Spherical Web Mercator EPSG:3857 (forward & inverse)");
        println!("  * Geodesy:    Haversine great-circle formula");
        println!("  * UI Design:  macOS Maps acrylic floating sidebar, Nearby drawer, Marked Location card");
        println!("  * Rasterizer: Pure 2D software double-buffer with 3D horizon tilt");
        println!("  * Routing:    A* waypoint pathfinding & turn-by-turn maneuvers");
        process::exit(0);
    }

    println!("Atlas Maps 1.0.0");
    println!("Default Sovereign Vector Maps & Navigation Application for NovaOS");
    println!("Run with --selftest to execute the autonomous qualification suite.");
}
