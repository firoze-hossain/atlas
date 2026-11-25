//! lib.rs - Atlas macOS-Synthesized User Interface Components.
//! Provides floating acrylic sidebar, search drawer, Marked Location card,
//! setup modals, right-side map controls, and weather widget.

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
    string::ToString,
};

pub use atlas_core::{
    GeoCoord, InteractionMode, MapDatabase, MapLayerStyle, MapState, Poi, PoiCategory,
};
pub use atlas_render::{CanvasSurface, Color, Rect, VectorMapRenderer};

/// Interactive hit-test targets across Atlas UI.
#[derive(Clone, Debug, PartialEq)]
pub enum MapUiHitTarget {
    SearchInput,
    SearchClear,
    PlacesHome,
    PlacesWork,
    PlacesAdd,
    FavoritesCard,
    ActionShareLocation,
    ActionMarkLocation,
    ActionReportIssue,
    NearbyCategory(PoiCategory),
    CloseMarkedLocation,
    RemoveMarkedLocation,
    ModalDone,
    ModalClose,
    ControlLayerSwitch,
    ControlLocateUser,
    ControlZoomIn,
    ControlZoomOut,
    ControlToggle3D,
    ControlCompass,
    WeatherWidget,
    MapCanvasClick(i32, i32),
}

/// UI Shell Renderer synthesizing macOS Maps.
pub struct MapUiRenderer;

impl MapUiRenderer {
    /// Renders the complete application frame (vector map + overlaid macOS UI).
    pub fn render(surface: &mut CanvasSurface, state: &MapState) {
        // 1. Render base vector cartography layer
        VectorMapRenderer::render(surface, state);

        // 2. Render Left Floating Panel (Sidebar, Search Drawer, or Detail Card)
        match state.mode {
            InteractionMode::Explore => {
                Self::render_floating_sidebar(surface, state);
            }
            InteractionMode::SearchDrawer => {
                Self::render_nearby_drawer(surface, state);
            }
            InteractionMode::MarkedLocationDetail => {
                Self::render_marked_location_card(surface, state);
            }
            InteractionMode::SetupHomeModal => {
                Self::render_floating_sidebar(surface, state);
                Self::render_setup_modal(surface, "Set up Home", state);
            }
            InteractionMode::SetupWorkModal => {
                Self::render_floating_sidebar(surface, state);
                Self::render_setup_modal(surface, "Set up Work", state);
            }
            InteractionMode::AddPinModal => {
                Self::render_floating_sidebar(surface, state);
                Self::render_setup_modal(surface, "Add Pin", state);
            }
            InteractionMode::ShareSheetPopover => {
                Self::render_floating_sidebar(surface, state);
                Self::render_share_popover(surface, state);
            }
            InteractionMode::TurnByTurnRouting => {
                Self::render_routing_panel(surface, state);
            }
        }

        // 3. Render Right-Side Floating Map Controls
        Self::render_right_controls(surface, state);

        // 4. Render Bottom-Right Weather Widget
        Self::render_weather_widget(surface, state);
    }

    /// Renders the primary macOS floating acrylic panel on the left.
    fn render_floating_sidebar(surface: &mut CanvasSurface, state: &MapState) {
        let panel_x = 18;
        let panel_y = 18;
        let panel_w = 320;
        let panel_h = (surface.height as i32) - 36;
        let panel_rect = Rect::new(panel_x, panel_y, panel_w, panel_h);

        // Frosted Glass Acrylic Background
        surface.fill_rounded_rect(panel_rect, 18, Color::rgba(255, 255, 255, 235));
        surface.draw_rect_outline(panel_rect, Color::rgba(0, 0, 0, 35));

        // Top Search Bar
        let search_rect = Rect::new(panel_x + 16, panel_y + 16, panel_w - 32, 40);
        surface.fill_rounded_rect(search_rect, 20, Color::rgba(240, 240, 245, 240));
        surface.draw_rect_outline(search_rect, Color::rgba(0, 0, 0, 25));

        // Magnifying glass icon & text
        surface.draw_char(search_rect.x + 12, search_rect.y + 12, 'Q', Color::rgb(142, 142, 147));
        let search_text = if state.search_query.is_empty() {
            "Search Maps"
        } else {
            &state.search_query
        };
        let text_col = if state.search_query.is_empty() {
            Color::rgb(142, 142, 147)
        } else {
            Color::rgb(28, 28, 30)
        };
        surface.draw_text(search_rect.x + 34, search_rect.y + 12, search_text, text_col);

        // Profile Avatar circle on the right of search bar
        let avatar_cx = search_rect.x + search_rect.width - 20;
        let avatar_cy = search_rect.y + 20;
        surface.fill_circle(avatar_cx, avatar_cy, 12, Color::rgb(210, 210, 215));
        surface.draw_circle_outline(avatar_cx, avatar_cy, 12, Color::rgb(160, 160, 165));
        surface.draw_char(avatar_cx - 4, avatar_cy - 8, '@', Color::rgb(100, 100, 105));

        // Places Section
        let mut cur_y = panel_y + 72;
        surface.draw_text(panel_x + 18, cur_y, "Places >", Color::rgb(60, 60, 67));

        cur_y += 24;
        let btn_w = (panel_w - 48) / 3;

        // Button 1: Home
        let home_rect = Rect::new(panel_x + 16, cur_y, btn_w, 64);
        Self::render_quick_place(surface, home_rect, 'H', "Home", "Add");

        // Button 2: Work
        let work_rect = Rect::new(panel_x + 16 + btn_w + 8, cur_y, btn_w, 64);
        Self::render_quick_place(surface, work_rect, 'W', "Work", "Add");

        // Button 3: Add
        let add_rect = Rect::new(panel_x + 16 + (btn_w + 8) * 2, cur_y, btn_w, 64);
        Self::render_quick_place(surface, add_rect, '+', "Add", "");

        // Your Guides / Favorites Section
        cur_y += 76;
        surface.draw_text(panel_x + 18, cur_y, "Your Guides >", Color::rgb(60, 60, 67));

        cur_y += 22;
        let fav_rect = Rect::new(panel_x + 16, cur_y, 100, 96);
        surface.fill_rounded_rect(fav_rect, 14, Color::rgba(242, 242, 247, 240));
        surface.draw_rect_outline(fav_rect, Color::rgba(0, 0, 0, 20));

        // Star 3D Badge icon
        surface.fill_circle(fav_rect.x + 50, fav_rect.y + 36, 18, Color::rgb(250, 204, 21));
        surface.draw_char(fav_rect.x + 46, fav_rect.y + 28, '*', Color::rgb(255, 255, 255));
        surface.draw_text(fav_rect.x + 16, fav_rect.y + 60, "Favorites", Color::rgb(28, 28, 30));
        surface.draw_text(fav_rect.x + 22, fav_rect.y + 76, "0 places", Color::rgb(142, 142, 147));

        // Recents Section
        cur_y += 112;
        surface.draw_text(panel_x + 18, cur_y, "Recents >", Color::rgb(60, 60, 67));

        cur_y += 22;
        let recent_rect = Rect::new(panel_x + 16, cur_y, panel_w - 32, 44);
        surface.fill_rounded_rect(recent_rect, 10, Color::rgba(245, 245, 247, 200));
        // Red pin dot
        surface.fill_circle(recent_rect.x + 16, recent_rect.y + 22, 6, Color::rgb(225, 29, 72));
        surface.draw_text(recent_rect.x + 32, recent_rect.y + 10, "Marked Location", Color::rgb(28, 28, 30));
        surface.draw_text(recent_rect.x + 32, recent_rect.y + 26, "Near United Hospital", Color::rgb(142, 142, 147));

        // Bottom Action Buttons
        let bottom_y = panel_y + panel_h - 120;
        let actions = [
            ("^", "Share My Location"),
            ("P", "Mark My Location"),
            ("!", "Report an Issue"),
        ];

        for (i, (icon, label)) in actions.iter().enumerate() {
            let act_y = bottom_y + (i as i32 * 34);
            let act_rect = Rect::new(panel_x + 16, act_y, panel_w - 32, 30);
            surface.fill_rounded_rect(act_rect, 8, Color::rgba(240, 242, 248, 160));
            surface.draw_char(act_rect.x + 10, act_rect.y + 7, icon.chars().next().unwrap(), Color::rgb(0, 122, 255));
            surface.draw_text(act_rect.x + 32, act_rect.y + 7, label, Color::rgb(0, 122, 255));
        }

        // Terms link
        surface.draw_text(panel_x + 80, panel_y + panel_h - 18, "Terms & Conditions >", Color::rgb(174, 174, 178));
    }

    fn render_quick_place(surface: &mut CanvasSurface, rect: Rect, icon: char, title: &str, subtitle: &str) {
        let cx = rect.x + rect.width / 2;
        surface.fill_circle(cx, rect.y + 22, 18, Color::rgb(224, 238, 254)); // Light blue circle
        surface.draw_char(cx - 4, rect.y + 14, icon, Color::rgb(0, 122, 255)); // Blue icon

        let title_x = cx - (title.len() as i32 * 8) / 2;
        surface.draw_text(title_x, rect.y + 44, title, Color::rgb(28, 28, 30));

        if !subtitle.is_empty() {
            let sub_x = cx - (subtitle.len() as i32 * 8) / 2;
            surface.draw_text(sub_x, rect.y + 56, subtitle, Color::rgb(0, 122, 255));
        }
    }

    /// Renders Find Nearby Category Drawer (Screenshot 11.07.55).
    fn render_nearby_drawer(surface: &mut CanvasSurface, state: &MapState) {
        let panel_x = 18;
        let panel_y = 18;
        let panel_w = 320;
        let panel_h = (surface.height as i32) - 36;
        let panel_rect = Rect::new(panel_x, panel_y, panel_w, panel_h);

        surface.fill_rounded_rect(panel_rect, 18, Color::rgba(255, 255, 255, 245));
        surface.draw_rect_outline(panel_rect, Color::rgba(0, 0, 0, 35));

        // Search Input with Close X button
        let search_rect = Rect::new(panel_x + 16, panel_y + 16, panel_w - 32, 40);
        surface.fill_rounded_rect(search_rect, 20, Color::rgba(240, 240, 245, 240));
        surface.draw_char(search_rect.x + 12, search_rect.y + 12, 'Q', Color::rgb(142, 142, 147));
        surface.draw_text(search_rect.x + 34, search_rect.y + 12, "Search Nearby", Color::rgb(28, 28, 30));
        surface.draw_char(search_rect.x + search_rect.width - 24, search_rect.y + 12, 'x', Color::rgb(142, 142, 147));

        surface.draw_text(panel_x + 18, panel_y + 70, "Find Nearby", Color::rgb(28, 28, 30));

        let categories = [
            (PoiCategory::Restaurant, "Restaurants", 'R'),
            (PoiCategory::FastFood, "Fast Food", 'F'),
            (PoiCategory::GasStation, "Gas Stations", 'G'),
            (PoiCategory::Cafe, "Coffee Shops", 'C'),
            (PoiCategory::Grocery, "Grocery Stores", 'M'),
            (PoiCategory::Hotel, "Hotels", 'H'),
            (PoiCategory::Shopping, "Shopping Centers", 'S'),
            (PoiCategory::Airport, "Airports", 'A'),
            (PoiCategory::Bank, "Banks and ATMs", '$'),
            (PoiCategory::ChargingStation, "Charging Stations", 'E'),
            (PoiCategory::Hospital, "Hospitals & Care", '+'),
        ];

        let mut item_y = panel_y + 96;
        for (cat, name, icon) in categories {
            let item_rect = Rect::new(panel_x + 16, item_y, panel_w - 32, 34);
            surface.draw_rect_outline(Rect::new(item_rect.x, item_rect.y + item_rect.height, item_rect.width, 1), Color::rgba(0, 0, 0, 15));

            let (r, g, b) = cat.color_rgb();
            surface.draw_char(item_rect.x + 8, item_rect.y + 9, icon, Color::rgb(r, g, b));
            surface.draw_text(item_rect.x + 32, item_rect.y + 9, name, Color::rgb(28, 28, 30));

            item_y += 36;
            if item_y > panel_y + panel_h - 40 {
                break;
            }
        }
    }

    /// Renders Marked Location Detail Card (Screenshot 11.07.20).
    fn render_marked_location_card(surface: &mut CanvasSurface, state: &MapState) {
        let card_x = 18;
        let card_y = 18;
        let card_w = 320;
        let card_h = 380;
        let card_rect = Rect::new(card_x, card_y, card_w, card_h);

        surface.fill_rounded_rect(card_rect, 18, Color::rgba(255, 255, 255, 245));
        surface.draw_rect_outline(card_rect, Color::rgba(0, 0, 0, 35));

        // Header controls (Share icon on left, Close X on right)
        surface.draw_char(card_x + 18, card_y + 16, '^', Color::rgb(0, 122, 255));
        surface.draw_char(card_x + card_w - 28, card_y + 16, 'x', Color::rgb(142, 142, 147));

        // Title and Distance
        let title = "Marked Location";
        let title_x = card_x + (card_w - (title.len() as i32 * 8)) / 2;
        surface.draw_text(title_x, card_y + 24, title, Color::rgb(28, 28, 30));
        surface.draw_text(title_x + 20, card_y + 44, "0 m away", Color::rgb(142, 142, 147));

        // Central Pink Pin Circle Badge
        let pin_cx = card_x + card_w / 2;
        let pin_cy = card_y + 90;
        surface.fill_circle(pin_cx, pin_cy, 24, Color::rgb(225, 29, 72));
        surface.draw_circle_outline(pin_cx, pin_cy, 24, Color::rgb(255, 255, 255));
        surface.draw_char(pin_cx - 4, pin_cy - 8, 'P', Color::rgb(255, 255, 255));

        // Details Section
        surface.draw_text(card_x + 18, card_y + 130, "Details", Color::rgb(28, 28, 30));
        surface.draw_text(card_x + 18, card_y + 152, "Address", Color::rgb(142, 142, 147));
        surface.draw_text(card_x + 90, card_y + 152, "United Hospital", Color::rgb(28, 28, 30));
        surface.draw_text(card_x + 90, card_y + 168, "Dhaka, Bangladesh", Color::rgb(142, 142, 147));

        surface.draw_text(card_x + 18, card_y + 192, "Coordinates", Color::rgb(142, 142, 147));
        let coord_str = if let Some(m) = state.marked_location {
            m.format_dms()
        } else {
            "23.80439° N, 90.41583° E".to_string()
        };
        surface.draw_text(card_x + 90, card_y + 192, &coord_str, Color::rgb(28, 28, 30));

        // Action Buttons
        let act1_rect = Rect::new(card_x + 16, card_y + 224, card_w - 32, 34);
        surface.fill_rounded_rect(act1_rect, 8, Color::rgba(240, 242, 248, 160));
        surface.draw_text(act1_rect.x + 32, act1_rect.y + 9, "Report Something Missing", Color::rgb(0, 122, 255));

        let act2_rect = Rect::new(card_x + 16, card_y + 266, card_w - 32, 34);
        surface.fill_rounded_rect(act2_rect, 8, Color::rgba(240, 242, 248, 160));
        surface.draw_text(act2_rect.x + 130, act2_rect.y + 9, "Pin", Color::rgb(0, 122, 255));

        let act3_rect = Rect::new(card_x + 16, card_y + 308, card_w - 32, 34);
        surface.fill_rounded_rect(act3_rect, 8, Color::rgba(254, 242, 242, 200));
        surface.draw_text(act3_rect.x + 120, act3_rect.y + 9, "Remove", Color::rgb(225, 29, 72));
    }

    /// Renders Setup Modal Dialog (Screenshots 11.06.32, 11.06.44, 11.06.54).
    fn render_setup_modal(surface: &mut CanvasSurface, title: &str, _state: &MapState) {
        let modal_w = 340;
        let modal_h = 420;
        let modal_x = (surface.width as i32 - modal_w) / 2;
        let modal_y = (surface.height as i32 - modal_h) / 2;
        let modal_rect = Rect::new(modal_x, modal_y, modal_w, modal_h);

        // Modal backdrop shadow
        surface.fill_rounded_rect(Rect::new(modal_x + 4, modal_y + 6, modal_w, modal_h), 18, Color::rgba(0, 0, 0, 45));
        surface.fill_rounded_rect(modal_rect, 18, Color::rgb(255, 255, 255));
        surface.draw_rect_outline(modal_rect, Color::rgba(0, 0, 0, 30));

        // Title
        let title_x = modal_x + (modal_w - (title.len() as i32 * 8)) / 2;
        surface.draw_text(title_x, modal_y + 20, title, Color::rgb(28, 28, 30));

        // Search Input in Modal
        let search_rect = Rect::new(modal_x + 20, modal_y + 54, modal_w - 40, 36);
        surface.fill_rounded_rect(search_rect, 18, Color::rgba(240, 240, 245, 240));
        surface.draw_char(search_rect.x + 12, search_rect.y + 10, 'Q', Color::rgb(142, 142, 147));
        surface.draw_text(search_rect.x + 32, search_rect.y + 10, "Search Maps", Color::rgb(142, 142, 147));

        surface.draw_text(modal_x + 20, modal_y + 106, "Siri Suggestions", Color::rgb(110, 110, 115));

        // Suggestions List
        let suggestions = [
            ("Nearby Transit", "Options for all nearby departures", PoiCategory::Transit),
            ("United Hospital", "Plot 15, Road 71, Gulshan-2", PoiCategory::Hospital),
            ("Road 16", "Residential, Gulshan", PoiCategory::Hotel),
        ];

        let mut item_y = modal_y + 130;
        for (name, sub, cat) in suggestions {
            let (r, g, b) = cat.color_rgb();
            surface.fill_circle(modal_x + 34, item_y + 16, 12, Color::rgb(r, g, b));
            surface.draw_char(modal_x + 30, item_y + 8, cat.icon_char(), Color::rgb(255, 255, 255));

            surface.draw_text(modal_x + 58, item_y + 6, name, Color::rgb(28, 28, 30));
            surface.draw_text(modal_x + 58, item_y + 22, sub, Color::rgb(142, 142, 147));

            // Pin icon on right
            surface.draw_circle_outline(modal_x + modal_w - 36, item_y + 16, 10, Color::rgb(0, 122, 255));
            surface.draw_char(modal_x + modal_w - 40, item_y + 8, 'P', Color::rgb(0, 122, 255));

            item_y += 46;
        }

        // Blue Done Button at bottom
        let done_rect = Rect::new(modal_x + modal_w - 90, modal_y + modal_h - 48, 70, 32);
        surface.fill_rounded_rect(done_rect, 8, Color::rgb(0, 122, 255));
        surface.draw_text(done_rect.x + 18, done_rect.y + 8, "Done", Color::rgb(255, 255, 255));
    }

    /// Renders Share Location Popover (Screenshot 11.07.08).
    fn render_share_popover(surface: &mut CanvasSurface, _state: &MapState) {
        let pop_w = 220;
        let pop_h = 240;
        let pop_x = 360;
        let pop_y = (surface.height as i32) - 260;
        let pop_rect = Rect::new(pop_x, pop_y, pop_w, pop_h);

        surface.fill_rounded_rect(pop_rect, 14, Color::rgba(255, 255, 255, 245));
        surface.draw_rect_outline(pop_rect, Color::rgba(0, 0, 0, 30));

        // Header: Shared Location
        surface.fill_circle(pop_x + 28, pop_y + 26, 12, Color::rgb(225, 29, 72));
        surface.draw_text(pop_x + 48, pop_y + 16, "Shared Location", Color::rgb(28, 28, 30));
        surface.draw_text(pop_x + 48, pop_y + 30, "Atlas Maps", Color::rgb(142, 142, 147));

        let share_items = ["AirDrop", "Mail", "Messages", "Notes", "Copy"];
        for (i, item) in share_items.iter().enumerate() {
            let row_y = pop_y + 56 + (i as i32 * 28);
            surface.draw_text(pop_x + 24, row_y, item, Color::rgb(28, 28, 30));
        }
    }

    /// Renders Turn-by-Turn Routing summary card.
    fn render_routing_panel(surface: &mut CanvasSurface, state: &MapState) {
        let panel_x = 18;
        let panel_y = 18;
        let panel_w = 340;
        let panel_h = 260;
        let panel_rect = Rect::new(panel_x, panel_y, panel_w, panel_h);

        surface.fill_rounded_rect(panel_rect, 18, Color::rgba(255, 255, 255, 245));
        surface.draw_rect_outline(panel_rect, Color::rgba(0, 0, 0, 35));

        if let Some(route) = &state.active_route {
            surface.draw_text(panel_x + 20, panel_y + 16, "Directions", Color::rgb(142, 142, 147));
            let header = format!("{} min ({:.1} km)", route.total_duration_minutes, route.total_distance_km);
            surface.draw_text(panel_x + 20, panel_y + 36, &header, Color::rgb(0, 122, 255));

            let route_sub = format!("Fastest route via {}", route.destination_name);
            surface.draw_text(panel_x + 20, panel_y + 58, &route_sub, Color::rgb(46, 125, 50));

            for (i, step) in route.steps.iter().enumerate() {
                let step_y = panel_y + 92 + (i as i32 * 44);
                surface.fill_circle(panel_x + 30, step_y + 10, 8, Color::rgb(0, 122, 255));
                surface.draw_char(panel_x + 26, step_y + 2, '^', Color::rgb(255, 255, 255));
                surface.draw_text(panel_x + 48, step_y, &step.instruction, Color::rgb(28, 28, 30));
                let dist_str = format!("{} m", step.distance_meters);
                surface.draw_text(panel_x + 48, step_y + 16, &dist_str, Color::rgb(142, 142, 147));
            }
        }
    }

    /// Renders Right Floating Controls (Map Style, Compass, Zoom, 3D).
    fn render_right_controls(surface: &mut CanvasSurface, state: &MapState) {
        let ctrl_x = (surface.width as i32) - 52;
        let mut cur_y = 18;

        // 1. Layer Switcher Button
        let layer_rect = Rect::new(ctrl_x, cur_y, 36, 36);
        surface.fill_rounded_rect(layer_rect, 8, Color::rgba(255, 255, 255, 230));
        surface.draw_rect_outline(layer_rect, Color::rgba(0, 0, 0, 25));
        surface.draw_char(ctrl_x + 14, cur_y + 10, 'L', Color::rgb(60, 60, 67));

        // 2. Locate User / Tracking Arrow
        cur_y += 44;
        let loc_rect = Rect::new(ctrl_x, cur_y, 36, 36);
        surface.fill_rounded_rect(loc_rect, 8, Color::rgba(255, 255, 255, 230));
        surface.draw_rect_outline(loc_rect, Color::rgba(0, 0, 0, 25));
        let loc_col = if state.is_tracking_user {
            Color::rgb(0, 122, 255)
        } else {
            Color::rgb(142, 142, 147)
        };
        surface.draw_char(ctrl_x + 14, cur_y + 10, '>', loc_col);

        // 3. Zoom Vertical Pill (+ / -)
        cur_y += 44;
        let zoom_rect = Rect::new(ctrl_x, cur_y, 36, 72);
        surface.fill_rounded_rect(zoom_rect, 8, Color::rgba(255, 255, 255, 230));
        surface.draw_rect_outline(zoom_rect, Color::rgba(0, 0, 0, 25));
        surface.draw_char(ctrl_x + 14, cur_y + 10, '+', Color::rgb(60, 60, 67));
        surface.draw_line(ctrl_x as f32 + 6.0, cur_y as f32 + 36.0, ctrl_x as f32 + 30.0, cur_y as f32 + 36.0, 1.0, Color::rgba(0, 0, 0, 25));
        surface.draw_char(ctrl_x + 14, cur_y + 46, '-', Color::rgb(60, 60, 67));

        // 4. 3D / 2D Switcher Pill
        cur_y += 80;
        let d3_rect = Rect::new(ctrl_x, cur_y, 36, 36);
        surface.fill_rounded_rect(d3_rect, 8, Color::rgba(255, 255, 255, 230));
        surface.draw_rect_outline(d3_rect, Color::rgba(0, 0, 0, 25));
        let d3_col = if state.pitch_deg > 0.0 {
            Color::rgb(0, 122, 255)
        } else {
            Color::rgb(60, 60, 67)
        };
        surface.draw_text(ctrl_x + 10, cur_y + 10, "3D", d3_col);

        // 5. Compass Dial (Points to North)
        cur_y += 44;
        let compass_rect = Rect::new(ctrl_x, cur_y, 36, 36);
        surface.fill_rounded_rect(compass_rect, 8, Color::rgba(255, 255, 255, 230));
        surface.draw_rect_outline(compass_rect, Color::rgba(0, 0, 0, 25));
        surface.draw_char(ctrl_x + 14, cur_y + 10, 'N', Color::rgb(225, 29, 72)); // Red North needle
    }

    /// Renders Bottom-Right Weather Badge (e.g. `25° 🌤️`).
    fn render_weather_widget(surface: &mut CanvasSurface, state: &MapState) {
        let badge_w = 64;
        let badge_h = 28;
        let badge_x = (surface.width as i32) - badge_w - 18;
        let badge_y = (surface.height as i32) - badge_h - 18;

        let rect = Rect::new(badge_x, badge_y, badge_w, badge_h);
        surface.fill_rounded_rect(rect, 8, Color::rgba(255, 255, 255, 220));
        surface.draw_rect_outline(rect, Color::rgba(0, 0, 0, 25));

        let weather_str = format!("{}°", state.temperature_celsius);
        surface.draw_text(badge_x + 8, badge_y + 6, &weather_str, Color::rgb(28, 28, 30));
        surface.draw_text(badge_x + 36, badge_y + 6, state.weather_symbol, Color::rgb(245, 158, 11));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_renderer_all_modes() {
        let mut surface = CanvasSurface::new(1024, 768);
        let mut state = MapState::default();

        // 1. Explore mode
        MapUiRenderer::render(&mut surface, &state);

        // 2. Search Drawer mode
        state.mode = InteractionMode::SearchDrawer;
        MapUiRenderer::render(&mut surface, &state);

        // 3. Marked Location Detail mode
        state.mode = InteractionMode::MarkedLocationDetail;
        MapUiRenderer::render(&mut surface, &state);

        // 4. Setup Home Modal mode
        state.mode = InteractionMode::SetupHomeModal;
        MapUiRenderer::render(&mut surface, &state);

        let non_zero = surface.buffer.iter().filter(|&&px| px != 0).count();
        assert!(non_zero > 1000, "Expected non-zero pixels across rendered map modes");
    }
}
