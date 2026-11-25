//! lib.rs - Atlas Core Cartography Engine.
//! Provides Mercator projections, distance calculations, spatial road graphs,
//! Points of Interest (POIs), routing calculations, and viewport state.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Earth radius in kilometers (WGS-84 mean).
pub const EARTH_RADIUS_KM: f64 = 6371.0;
pub const PI: f64 = 3.141592653589793;

/// Geographic coordinates (Latitude, Longitude in degrees).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoCoord {
    pub lat: f64,
    pub lon: f64,
}

impl GeoCoord {
    pub const fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// Formats coordinate string like `23.80439° N, 90.41583° E`.
    pub fn format_dms(&self) -> String {
        let lat_dir = if self.lat >= 0.0 { 'N' } else { 'S' };
        let lon_dir = if self.lon >= 0.0 { 'E' } else { 'W' };
        format!(
            "{:.5}° {}, {:.5}° {}",
            self.lat.abs(),
            lat_dir,
            self.lon.abs(),
            lon_dir
        )
    }

    /// Calculates great-circle distance between two coordinates in kilometers using Haversine formula.
    pub fn distance_to(&self, other: &GeoCoord) -> f64 {
        let d_lat = (other.lat - self.lat) * (PI / 180.0);
        let d_lon = (other.lon - self.lon) * (PI / 180.0);
        let lat1 = self.lat * (PI / 180.0);
        let lat2 = other.lat * (PI / 180.0);

        let sin_dlat_2 = (d_lat / 2.0).sin();
        let sin_dlon_2 = (d_lon / 2.0).sin();

        let a = sin_dlat_2 * sin_dlat_2 + lat1.cos() * lat2.cos() * sin_dlon_2 * sin_dlon_2;
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        EARTH_RADIUS_KM * c
    }
}

/// 2D Screen or World point in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

impl ScreenPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Spherical Web Mercator Projection (EPSG:3857) engine.
pub struct MercatorProjection;

impl MercatorProjection {
    /// Projects a `GeoCoord` to normalized world coordinates [0.0..1.0, 0.0..1.0].
    pub fn geo_to_normalized(coord: &GeoCoord) -> (f64, f64) {
        let x = (coord.lon + 180.0) / 360.0;
        let lat_rad = (coord.lat.clamp(-85.05112878, 85.05112878)) * (PI / 180.0);
        let y = (1.0 - ((lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI)) / 2.0;
        (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
    }

    /// Converts normalized world coordinates [0.0..1.0, 0.0..1.0] back to `GeoCoord`.
    pub fn normalized_to_geo(x: f64, y: f64) -> GeoCoord {
        let lon = x * 360.0 - 180.0;
        let n = PI - 2.0 * PI * y;
        let lat_rad = (0.5 * (n.exp() - (-n).exp())).atan();
        let lat = lat_rad * (180.0 / PI);
        GeoCoord::new(lat, lon)
    }

    /// Projects `GeoCoord` directly to screen pixels relative to a center coordinate, zoom, and viewport size.
    pub fn geo_to_screen(
        coord: &GeoCoord,
        center: &GeoCoord,
        zoom: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) -> ScreenPoint {
        let scale = 256.0 * (2.0f64).powf(zoom as f64);
        let (cx, cy) = Self::geo_to_normalized(center);
        let (px, py) = Self::geo_to_normalized(coord);

        let dx = (px - cx) * scale;
        let dy = (py - cy) * scale;

        let screen_x = (viewport_w / 2.0) as f64 + dx;
        let screen_y = (viewport_h / 2.0) as f64 + dy;

        ScreenPoint::new(screen_x as f32, screen_y as f32)
    }

    /// Unprojects screen pixels back to `GeoCoord`.
    pub fn screen_to_geo(
        screen: ScreenPoint,
        center: &GeoCoord,
        zoom: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) -> GeoCoord {
        let scale = 256.0 * (2.0f64).powf(zoom as f64);
        let (cx, cy) = Self::geo_to_normalized(center);

        let dx = (screen.x as f64) - (viewport_w as f64 / 2.0);
        let dy = (screen.y as f64) - (viewport_h as f64 / 2.0);

        let px = cx + (dx / scale);
        let py = cy + (dy / scale);

        Self::normalized_to_geo(px, py)
    }
}

/// Point of Interest (POI) categories matching macOS Maps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PoiCategory {
    Hospital,
    Hotel,
    Restaurant,
    FastFood,
    Cafe,
    School,
    Transit,
    Shopping,
    GasStation,
    Bank,
    Airport,
    Park,
    Pharmacy,
    Atm,
    ChargingStation,
    Grocery,
}

impl PoiCategory {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Hospital => "Hospitals & Clinics",
            Self::Hotel => "Hotels",
            Self::Restaurant => "Restaurants",
            Self::FastFood => "Fast Food",
            Self::Cafe => "Coffee Shops",
            Self::School => "Schools & Universities",
            Self::Transit => "Transit Stations",
            Self::Shopping => "Shopping Centers",
            Self::GasStation => "Gas Stations",
            Self::Bank => "Banks & ATMs",
            Self::Airport => "Airports",
            Self::Park => "Parks & Recreation",
            Self::Pharmacy => "Pharmacies",
            Self::Atm => "ATMs",
            Self::ChargingStation => "EV Charging",
            Self::Grocery => "Grocery Stores",
        }
    }

    /// Primary badge RGB color.
    pub const fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Hospital | Self::Pharmacy => (239, 68, 68),   // Crimson Red
            Self::Hotel => (168, 85, 247),                      // Purple Amethyst
            Self::Restaurant => (249, 115, 22),                 // Vibrant Orange
            Self::FastFood => (245, 158, 11),                   // Amber
            Self::Cafe => (217, 119, 6),                        // Warm Brown
            Self::School => (133, 77, 14),                      // Sienna
            Self::Transit | Self::Airport => (59, 130, 246),    // Royal Blue
            Self::Shopping | Self::Grocery => (234, 179, 8),    // Golden Yellow
            Self::GasStation => (6, 182, 212),                  // Cyan
            Self::Bank | Self::Atm => (16, 185, 129),           // Emerald Green
            Self::Park => (34, 197, 94),                        // Fresh Green
            Self::ChargingStation => (16, 185, 129),            // Green
        }
    }

    pub const fn icon_char(&self) -> char {
        match self {
            Self::Hospital | Self::Pharmacy => '+',
            Self::Hotel => 'H',
            Self::Restaurant => 'R',
            Self::FastFood => 'F',
            Self::Cafe => 'C',
            Self::School => 'U',
            Self::Transit => 'T',
            Self::Shopping => 'S',
            Self::GasStation => 'G',
            Self::Bank | Self::Atm => '$',
            Self::Airport => 'A',
            Self::Park => 'P',
            Self::ChargingStation => 'E',
            Self::Grocery => 'M',
        }
    }
}

/// Point of Interest descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct Poi {
    pub id: &'static str,
    pub name: &'static str,
    pub category: PoiCategory,
    pub coord: GeoCoord,
    pub address: &'static str,
    pub subtitle: &'static str,
    pub rating: f32,
}

/// Road hierarchy classifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoadKind {
    Expressway, // Elevated or limited-access high speed road
    Highway,    // National primary route
    Avenue,     // Major city arterial
    Street,     // Urban residential street
    Bridge,     // Elevated bridge spanning water
}

impl RoadKind {
    pub const fn stroke_width(&self, zoom: f32) -> f32 {
        let base = match self {
            Self::Expressway => 8.0,
            Self::Highway => 6.5,
            Self::Avenue => 5.0,
            Self::Bridge => 6.0,
            Self::Street => 3.5,
        };
        if zoom > 14.0 {
            base * (zoom / 14.0)
        } else {
            base.max(2.0)
        }
    }

    pub const fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Expressway => (255, 183, 77), // Warm amber highway
            Self::Highway => (254, 215, 102),    // Golden primary
            Self::Avenue => (255, 255, 255),     // Crisp white avenue
            Self::Bridge => (224, 224, 224),     // Reinforced bridge concrete
            Self::Street => (250, 250, 250),     // Clean street white
        }
    }

    pub const fn casing_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Expressway => (224, 138, 48),
            Self::Highway => (218, 172, 60),
            Self::Avenue | Self::Street => (208, 208, 214),
            Self::Bridge => (160, 160, 170),
        }
    }
}

/// Continuous road polyline segment.
#[derive(Clone, Debug)]
pub struct RoadSegment {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: RoadKind,
    pub points: Vec<GeoCoord>,
    pub one_way: bool,
    pub speed_limit_kmh: u32,
}

/// Water body and park polygon classifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonKind {
    Lake,
    River,
    Pond,
    Park,
    GolfClub,
    Stadium,
}

impl PolygonKind {
    pub const fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Lake | Self::River | Self::Pond => (163, 212, 251), // Calming Cyan-Blue Water
            Self::Park => (203, 231, 185),                           // Fresh Meadow Green
            Self::GolfClub => (188, 224, 168),                       // Manicured Green
            Self::Stadium => (220, 238, 210),                        // Sports Arena
        }
    }
}

/// Cartographic area polygon.
#[derive(Clone, Debug)]
pub struct GeoPolygon {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: PolygonKind,
    pub points: Vec<GeoCoord>,
}

/// Turn-by-turn navigation maneuver step.
#[derive(Clone, Debug, PartialEq)]
pub struct RouteStep {
    pub instruction: String,
    pub distance_meters: u32,
    pub duration_seconds: u32,
    pub waypoint: GeoCoord,
}

/// Complete planned navigation itinerary.
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationRoute {
    pub origin_name: String,
    pub destination_name: String,
    pub total_distance_km: f64,
    pub total_duration_minutes: u32,
    pub steps: Vec<RouteStep>,
}

/// Map rendering and layout modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapLayerStyle {
    Standard, // Clean vector streets and parks
    Satellite, // High-altitude aerial terrain
    Transit,  // Highlighted railway, metro & bus routes
}

/// Active interaction mode in the map view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionMode {
    Explore,
    SearchDrawer,
    MarkedLocationDetail,
    SetupHomeModal,
    SetupWorkModal,
    AddPinModal,
    ShareSheetPopover,
    TurnByTurnRouting,
}

/// Primary dynamic map state managing viewport, coordinates, and pins.
#[derive(Clone, Debug)]
pub struct MapState {
    pub center: GeoCoord,
    pub zoom: f32,
    pub pitch_deg: f32, // 0.0 (flat 2D) to 45.0 (3D perspective)
    pub heading_deg: f32, // Compass rotation (0 = North)
    pub user_location: GeoCoord,
    pub marked_location: Option<GeoCoord>,
    pub home_location: Option<GeoCoord>,
    pub work_location: Option<GeoCoord>,
    pub selected_poi: Option<Poi>,
    pub active_route: Option<NavigationRoute>,
    pub layer_style: MapLayerStyle,
    pub mode: InteractionMode,
    pub is_tracking_user: bool,
    pub search_query: String,
    pub temperature_celsius: i32,
    pub weather_symbol: &'static str,
}

impl Default for MapState {
    fn default() -> Self {
        // Default center: United Hospital & Gulshan Lake, Dhaka (matching macOS screenshot)
        let default_coord = GeoCoord::new(23.80439, 90.41583);
        Self {
            center: default_coord,
            zoom: 15.2,
            pitch_deg: 0.0,
            heading_deg: 0.0,
            user_location: default_coord,
            marked_location: Some(default_coord),
            home_location: None,
            work_location: None,
            selected_poi: None,
            active_route: None,
            layer_style: MapLayerStyle::Standard,
            mode: InteractionMode::Explore,
            is_tracking_user: true,
            search_query: String::new(),
            temperature_celsius: 25,
            weather_symbol: "🌤️",
        }
    }
}

impl MapState {
    pub fn pan(&mut self, dx_pixels: f32, dy_pixels: f32, viewport_w: f32, viewport_h: f32) {
        let center_screen = ScreenPoint::new(viewport_w / 2.0 - dx_pixels, viewport_h / 2.0 - dy_pixels);
        self.center = MercatorProjection::screen_to_geo(center_screen, &self.center, self.zoom, viewport_w, viewport_h);
        self.is_tracking_user = false;
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.5).min(18.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.5).max(1.0);
    }

    pub fn toggle_3d(&mut self) {
        if self.pitch_deg > 0.0 {
            self.pitch_deg = 0.0;
        } else {
            self.pitch_deg = 35.0;
        }
    }

    pub fn center_on_user(&mut self) {
        self.center = self.user_location;
        self.is_tracking_user = true;
    }

    pub fn mark_current_location(&mut self) {
        self.marked_location = Some(self.user_location);
        self.mode = InteractionMode::MarkedLocationDetail;
    }

    pub fn clear_marked_location(&mut self) {
        self.marked_location = None;
        if self.mode == InteractionMode::MarkedLocationDetail {
            self.mode = InteractionMode::Explore;
        }
    }

    /// Plans a navigation route between two points.
    pub fn plan_route(&mut self, from: GeoCoord, to: GeoCoord, from_name: &str, to_name: &str) {
        let dist = from.distance_to(&to);
        // Estimate: 30 km/h average speed in city
        let duration = ((dist / 30.0) * 60.0).round().max(3.0) as u32;

        let step1 = RouteStep {
            instruction: format!("Depart toward {}", to_name),
            distance_meters: (dist * 400.0) as u32,
            duration_seconds: (duration * 20),
            waypoint: from,
        };
        let step2 = RouteStep {
            instruction: format!("Arrive at destination: {}", to_name),
            distance_meters: (dist * 600.0) as u32,
            duration_seconds: (duration * 40),
            waypoint: to,
        };

        self.active_route = Some(NavigationRoute {
            origin_name: from_name.to_string(),
            destination_name: to_name.to_string(),
            total_distance_km: dist,
            total_duration_minutes: duration,
            steps: vec![step1, step2],
        });
        self.mode = InteractionMode::TurnByTurnRouting;
    }
}

/// Cartographic database containing high-fidelity roads, lakes, parks, and POIs.
pub struct MapDatabase;

impl MapDatabase {
    /// Loads all pre-loaded vector polygons (lakes, rivers, parks, stadiums).
    pub fn load_polygons() -> Vec<GeoPolygon> {
        vec![
            // Gulshan-Badda Lake
            GeoPolygon {
                id: "gulshan_lake",
                name: "Gulshan-Badda Lake",
                kind: PolygonKind::Lake,
                points: vec![
                    GeoCoord::new(23.8150, 90.4140),
                    GeoCoord::new(23.8120, 90.4150),
                    GeoCoord::new(23.8050, 90.4160),
                    GeoCoord::new(23.7980, 90.4170),
                    GeoCoord::new(23.7910, 90.4150),
                    GeoCoord::new(23.7850, 90.4120),
                    GeoCoord::new(23.7870, 90.4160),
                    GeoCoord::new(23.7950, 90.4200),
                    GeoCoord::new(23.8040, 90.4210),
                    GeoCoord::new(23.8120, 90.4180),
                ],
            },
            // Baridhara Lakeside Rajuk Park
            GeoPolygon {
                id: "baridhara_park",
                name: "Baridhara Lakeside Rajuk Park",
                kind: PolygonKind::Park,
                points: vec![
                    GeoCoord::new(23.8035, 90.4165),
                    GeoCoord::new(23.8010, 90.4172),
                    GeoCoord::new(23.7990, 90.4168),
                    GeoCoord::new(23.8015, 90.4155),
                ],
            },
            // Justice Shahabuddin Ahmed Park
            GeoPolygon {
                id: "shahabuddin_park",
                name: "Justice Shahabuddin Ahmed Park",
                kind: PolygonKind::Park,
                points: vec![
                    GeoCoord::new(23.7960, 90.4135),
                    GeoCoord::new(23.7940, 90.4145),
                    GeoCoord::new(23.7925, 90.4130),
                    GeoCoord::new(23.7945, 90.4120),
                ],
            },
            // Army Golf Club
            GeoPolygon {
                id: "army_golf_club",
                name: "Army Golf Club",
                kind: PolygonKind::GolfClub,
                points: vec![
                    GeoCoord::new(23.8220, 90.4080),
                    GeoCoord::new(23.8250, 90.4150),
                    GeoCoord::new(23.8180, 90.4170),
                    GeoCoord::new(23.8150, 90.4100),
                ],
            },
            // Bangladesh Army Stadium
            GeoPolygon {
                id: "army_stadium",
                name: "Bangladesh Army Stadium",
                kind: PolygonKind::Stadium,
                points: vec![
                    GeoCoord::new(23.8110, 90.4020),
                    GeoCoord::new(23.8130, 90.4060),
                    GeoCoord::new(23.8080, 90.4070),
                    GeoCoord::new(23.8060, 90.4030),
                ],
            },
        ]
    }

    /// Loads primary road network.
    pub fn load_roads() -> Vec<RoadSegment> {
        vec![
            // Dhaka Elevated Expressway
            RoadSegment {
                id: "elevated_exp",
                name: "Dhaka Elevated Expressway",
                kind: RoadKind::Expressway,
                points: vec![
                    GeoCoord::new(23.8300, 90.4060),
                    GeoCoord::new(23.8180, 90.4030),
                    GeoCoord::new(23.8050, 90.3990),
                    GeoCoord::new(23.7900, 90.3950),
                ],
                one_way: true,
                speed_limit_kmh: 80,
            },
            // Pragati Sarani
            RoadSegment {
                id: "pragati_sarani",
                name: "Pragati Sarani",
                kind: RoadKind::Highway,
                points: vec![
                    GeoCoord::new(23.8250, 90.4240),
                    GeoCoord::new(23.8150, 90.4220),
                    GeoCoord::new(23.8000, 90.4230),
                    GeoCoord::new(23.7850, 90.4250),
                ],
                one_way: false,
                speed_limit_kmh: 60,
            },
            // Kamal Ataturk Avenue
            RoadSegment {
                id: "kamal_ataturk",
                name: "Kamal Ataturk Avenue",
                kind: RoadKind::Avenue,
                points: vec![
                    GeoCoord::new(23.7930, 90.4000),
                    GeoCoord::new(23.7935, 90.4070),
                    GeoCoord::new(23.7940, 90.4150),
                    GeoCoord::new(23.7945, 90.4230),
                ],
                one_way: false,
                speed_limit_kmh: 50,
            },
            // Road 71
            RoadSegment {
                id: "road_71",
                name: "Road 71",
                kind: RoadKind::Street,
                points: vec![
                    GeoCoord::new(23.8060, 90.4120),
                    GeoCoord::new(23.8055, 90.4160),
                    GeoCoord::new(23.8050, 90.4190),
                ],
                one_way: false,
                speed_limit_kmh: 40,
            },
            // Road 62
            RoadSegment {
                id: "road_62",
                name: "Road 62",
                kind: RoadKind::Street,
                points: vec![
                    GeoCoord::new(23.8020, 90.4110),
                    GeoCoord::new(23.8015, 90.4155),
                    GeoCoord::new(23.8010, 90.4180),
                ],
                one_way: false,
                speed_limit_kmh: 40,
            },
            // Road 11
            RoadSegment {
                id: "road_11",
                name: "Road 11",
                kind: RoadKind::Avenue,
                points: vec![
                    GeoCoord::new(23.7890, 90.4030),
                    GeoCoord::new(23.7895, 90.4110),
                    GeoCoord::new(23.7900, 90.4160),
                ],
                one_way: false,
                speed_limit_kmh: 40,
            },
            // Banani-Gulshan Bridge
            RoadSegment {
                id: "banani_bridge",
                name: "Banani-Gulshan Bridge",
                kind: RoadKind::Bridge,
                points: vec![
                    GeoCoord::new(23.7915, 90.4090),
                    GeoCoord::new(23.7918, 90.4115),
                ],
                one_way: false,
                speed_limit_kmh: 40,
            },
        ]
    }

    /// Loads points of interest.
    pub fn load_pois() -> Vec<Poi> {
        vec![
            Poi {
                id: "united_hospital",
                name: "United Hospital",
                category: PoiCategory::Hospital,
                coord: GeoCoord::new(23.80439, 90.41583),
                address: "Plot 15, Road 71, Gulshan-2, Dhaka 1212",
                subtitle: "Multispecialty Hospital & Emergency",
                rating: 4.8,
            },
            Poi {
                id: "lake_view_clinic",
                name: "Lake View Clinic",
                category: PoiCategory::Hospital,
                coord: GeoCoord::new(23.80250, 90.41550),
                address: "Road 70, Gulshan, Dhaka",
                subtitle: "Specialized Healthcare Clinic",
                rating: 4.5,
            },
            Poi {
                id: "westin_dhaka",
                name: "The Westin Dhaka",
                category: PoiCategory::Hotel,
                coord: GeoCoord::new(23.79250, 90.41500),
                address: "Main Gulshan Avenue, Dhaka",
                subtitle: "5-Star Luxury Hotel",
                rating: 4.7,
            },
            Poi {
                id: "radisson_blu",
                name: "Radisson Blu Water Garden",
                category: PoiCategory::Hotel,
                coord: GeoCoord::new(23.81800, 90.41200),
                address: "Airport Road, Dhaka Cantonment",
                subtitle: "Premium Resort & Hotel",
                rating: 4.6,
            },
            Poi {
                id: "sultans_dine",
                name: "Sultan's Dine, Gulshan",
                category: PoiCategory::Restaurant,
                coord: GeoCoord::new(23.79420, 90.41380),
                address: "Kamal Ataturk Avenue, Gulshan-2",
                subtitle: "Authentic Kacchi Biryani & Kebabs",
                rating: 4.9,
            },
            Poi {
                id: "star_kabab",
                name: "Star Kabab & Restaurant",
                category: PoiCategory::Restaurant,
                coord: GeoCoord::new(23.79300, 90.40400),
                address: "Banani 11, Dhaka",
                subtitle: "Traditional Mughal Cuisine",
                rating: 4.7,
            },
            Poi {
                id: "tarka",
                name: "Tarka Indian Dining",
                category: PoiCategory::Restaurant,
                coord: GeoCoord::new(23.78850, 90.41050),
                address: "Road 11, Banani, Dhaka",
                subtitle: "North Indian Gourmet Specialties",
                rating: 4.6,
            },
            Poi {
                id: "jamuna_future_park",
                name: "Jamuna Future Park",
                category: PoiCategory::Shopping,
                coord: GeoCoord::new(23.81300, 90.42400),
                address: "Pragati Sarani, Baridhara",
                subtitle: "Premier Shopping & Entertainment Complex",
                rating: 4.8,
            },
            Poi {
                id: "dhaka_cantonment_stn",
                name: "Dhaka Cantonment Railway Station",
                category: PoiCategory::Transit,
                coord: GeoCoord::new(23.81850, 90.40350),
                address: "Dhaka Cantonment Railway Track",
                subtitle: "Suburban Rail & Intercity Terminal",
                rating: 4.4,
            },
            Poi {
                id: "independent_university",
                name: "Independent University, Bangladesh (IUB)",
                category: PoiCategory::School,
                coord: GeoCoord::new(23.81600, 90.42800),
                address: "Plot 16, Block B, Bashundhara R/A",
                subtitle: "Leading Higher Education Institution",
                rating: 4.7,
            },
        ]
    }

    /// Searches POIs matching substring `query` (case-insensitive).
    pub fn search_pois(query: &str) -> Vec<Poi> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let all = Self::load_pois();
        all.into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q)
                    || p.address.to_lowercase().contains(&q)
                    || p.category.name().to_lowercase().contains(&q)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let dhaka = GeoCoord::new(23.8103, 90.4125);
        let chittagong = GeoCoord::new(22.3569, 91.7832);
        let dist = dhaka.distance_to(&chittagong);
        // Approx 210-220 km
        assert!(dist > 200.0 && dist < 240.0, "Distance was {}", dist);
    }

    #[test]
    fn test_mercator_roundtrip() {
        let orig = GeoCoord::new(23.80439, 90.41583);
        let (x, y) = MercatorProjection::geo_to_normalized(&orig);
        let back = MercatorProjection::normalized_to_geo(x, y);

        assert!((orig.lat - back.lat).abs() < 1e-4);
        assert!((orig.lon - back.lon).abs() < 1e-4);
    }

    #[test]
    fn test_screen_projection_center() {
        let center = GeoCoord::new(23.80439, 90.41583);
        let screen = MercatorProjection::geo_to_screen(&center, &center, 15.0, 1024.0, 768.0);
        assert!((screen.x - 512.0).abs() < 0.1);
        assert!((screen.y - 384.0).abs() < 0.1);
    }

    #[test]
    fn test_poi_search() {
        let results = MapDatabase::search_pois("Hospital");
        assert!(results.len() >= 2);
        assert!(results.iter().any(|p| p.name == "United Hospital"));
    }

    #[test]
    fn test_route_planning() {
        let mut state = MapState::default();
        let from = GeoCoord::new(23.80439, 90.41583);
        let to = GeoCoord::new(23.79250, 90.41500);
        state.plan_route(from, to, "United Hospital", "The Westin Dhaka");

        assert!(state.active_route.is_some());
        let route = state.active_route.as_ref().unwrap();
        assert!(route.total_distance_km > 0.5);
        assert!(route.total_duration_minutes > 0);
        assert_eq!(route.steps.len(), 2);
    }
}
