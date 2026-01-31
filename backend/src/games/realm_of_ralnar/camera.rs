//! Camera system for Realm of Ralnar
//!
//! Manages the viewport for determining which portion of the map is visible.

use super::map::{Map, MapType, Tile};

/// Camera for determining visible map area
#[derive(Debug, Clone)]
pub struct Camera {
    /// Center X position (can be fractional for smooth scrolling)
    pub center_x: f32,
    /// Center Y position (can be fractional for smooth scrolling)
    pub center_y: f32,
    /// Viewport width in tiles
    pub view_width: u32,
    /// Viewport height in tiles
    pub view_height: u32,
}

impl Camera {
    /// Create a new camera with the given viewport size
    pub fn new(view_width: u32, view_height: u32) -> Self {
        Camera {
            center_x: 0.0,
            center_y: 0.0,
            view_width,
            view_height,
        }
    }

    /// Create a camera for typical BBS terminal (80x24)
    /// Using a portion for the map view (leaves room for status)
    pub fn for_bbs() -> Self {
        // Typical BBS layout: 60 columns for map, 18 rows for map
        Camera::new(30, 9)
    }

    /// Update camera to follow a player position
    pub fn follow_player(&mut self, player_x: u32, player_y: u32) {
        self.center_x = player_x as f32;
        self.center_y = player_y as f32;
    }

    /// Smoothly pan towards a target position
    pub fn pan_towards(&mut self, target_x: f32, target_y: f32, speed: f32) {
        let dx = target_x - self.center_x;
        let dy = target_y - self.center_y;

        self.center_x += dx * speed;
        self.center_y += dy * speed;
    }

    /// Get the top-left corner of the visible area
    pub fn get_viewport_origin(&self) -> (i32, i32) {
        let half_w = self.view_width as i32 / 2;
        let half_h = self.view_height as i32 / 2;
        (self.center_x as i32 - half_w, self.center_y as i32 - half_h)
    }

    /// Get all visible tiles with their screen-relative positions
    pub fn get_visible_tiles<'a>(&self, map: &'a Map) -> Vec<VisibleTile<'a>> {
        let half_w = self.view_width as i32 / 2;
        let half_h = self.view_height as i32 / 2;
        let cx = self.center_x as i32;
        let cy = self.center_y as i32;

        let mut visible = Vec::new();

        for dy in -half_h..=half_h {
            for dx in -half_w..=half_w {
                let world_x = cx + dx;
                let world_y = cy + dy;
                let screen_x = (dx + half_w) as u32;
                let screen_y = (dy + half_h) as u32;

                if let Some(tile) = map.get_tile(world_x, world_y) {
                    visible.push(VisibleTile {
                        world_x,
                        world_y,
                        screen_x,
                        screen_y,
                        tile,
                    });
                }
            }
        }

        visible
    }

    /// Get visible tile coordinates as a grid for rendering
    pub fn get_visible_grid(&self, map: &Map) -> Vec<Vec<Option<VisibleTileInfo>>> {
        let half_w = self.view_width as i32 / 2;
        let half_h = self.view_height as i32 / 2;
        let cx = self.center_x as i32;
        let cy = self.center_y as i32;

        let height = (half_h * 2 + 1) as usize;
        let width = (half_w * 2 + 1) as usize;

        let mut grid = vec![vec![None; width]; height];

        for (row_idx, dy) in (-half_h..=half_h).enumerate() {
            for (col_idx, dx) in (-half_w..=half_w).enumerate() {
                let world_x = cx + dx;
                let world_y = cy + dy;

                if let Some(tile) = map.get_tile(world_x, world_y) {
                    // Calculate wrapped world coordinates for overworld
                    let wrapped_x = if map.map_type == MapType::Overworld {
                        ((world_x % map.width as i32) + map.width as i32) as u32 % map.width
                    } else {
                        world_x as u32
                    };
                    let wrapped_y = if map.map_type == MapType::Overworld {
                        ((world_y % map.height as i32) + map.height as i32) as u32 % map.height
                    } else {
                        world_y as u32
                    };

                    grid[row_idx][col_idx] = Some(VisibleTileInfo {
                        world_x: wrapped_x,
                        world_y: wrapped_y,
                        base_id: tile.base_id,
                        overlay_id: tile.overlay_id,
                        passable: tile.attributes.passable,
                        is_water: tile.attributes.water,
                    });
                }
            }
        }

        grid
    }

    /// Check if a world position is currently visible
    pub fn is_visible(&self, world_x: i32, world_y: i32) -> bool {
        let half_w = self.view_width as i32 / 2;
        let half_h = self.view_height as i32 / 2;
        let cx = self.center_x as i32;
        let cy = self.center_y as i32;

        world_x >= cx - half_w
            && world_x <= cx + half_w
            && world_y >= cy - half_h
            && world_y <= cy + half_h
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_x: i32, world_y: i32) -> Option<(u32, u32)> {
        if !self.is_visible(world_x, world_y) {
            return None;
        }

        let half_w = self.view_width as i32 / 2;
        let half_h = self.view_height as i32 / 2;
        let cx = self.center_x as i32;
        let cy = self.center_y as i32;

        let screen_x = (world_x - cx + half_w) as u32;
        let screen_y = (world_y - cy + half_h) as u32;

        Some((screen_x, screen_y))
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_x: u32, screen_y: u32) -> (i32, i32) {
        let half_w = self.view_width as i32 / 2;
        let half_h = self.view_height as i32 / 2;
        let cx = self.center_x as i32;
        let cy = self.center_y as i32;

        let world_x = screen_x as i32 + cx - half_w;
        let world_y = screen_y as i32 + cy - half_h;

        (world_x, world_y)
    }

    /// Clamp camera to map bounds (for non-wrapping maps)
    pub fn clamp_to_map(&mut self, map: &Map) {
        if map.map_type == MapType::Overworld {
            return; // Overworld wraps, no clamping needed
        }

        let half_w = self.view_width as f32 / 2.0;
        let half_h = self.view_height as f32 / 2.0;

        self.center_x = self
            .center_x
            .max(half_w)
            .min(map.width as f32 - half_w - 1.0);
        self.center_y = self
            .center_y
            .max(half_h)
            .min(map.height as f32 - half_h - 1.0);
    }
}

/// A visible tile with its world and screen positions
#[derive(Debug, Clone)]
pub struct VisibleTile<'a> {
    /// World X coordinate
    pub world_x: i32,
    /// World Y coordinate
    pub world_y: i32,
    /// Screen X coordinate
    pub screen_x: u32,
    /// Screen Y coordinate
    pub screen_y: u32,
    /// Reference to the tile
    pub tile: &'a Tile,
}

/// Simplified tile info for rendering (without lifetime)
#[derive(Debug, Clone)]
pub struct VisibleTileInfo {
    /// World X coordinate
    pub world_x: u32,
    /// World Y coordinate
    pub world_y: u32,
    /// Base tile ID
    pub base_id: u16,
    /// Overlay tile ID
    pub overlay_id: Option<u16>,
    /// Whether tile is passable
    pub passable: bool,
    /// Whether tile is water
    pub is_water: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_new() {
        let camera = Camera::new(20, 15);
        assert_eq!(camera.view_width, 20);
        assert_eq!(camera.view_height, 15);
        assert_eq!(camera.center_x, 0.0);
        assert_eq!(camera.center_y, 0.0);
    }

    #[test]
    fn test_camera_for_bbs() {
        let camera = Camera::for_bbs();
        assert_eq!(camera.view_width, 30);
        assert_eq!(camera.view_height, 9);
    }

    #[test]
    fn test_follow_player() {
        let mut camera = Camera::new(10, 10);
        camera.follow_player(50, 100);
        assert_eq!(camera.center_x, 50.0);
        assert_eq!(camera.center_y, 100.0);
    }

    #[test]
    fn test_pan_towards() {
        let mut camera = Camera::new(10, 10);
        camera.center_x = 0.0;
        camera.center_y = 0.0;

        camera.pan_towards(10.0, 20.0, 0.5);

        // Should move halfway
        assert_eq!(camera.center_x, 5.0);
        assert_eq!(camera.center_y, 10.0);
    }

    #[test]
    fn test_get_viewport_origin() {
        let mut camera = Camera::new(10, 10);
        camera.center_x = 50.0;
        camera.center_y = 50.0;

        let (origin_x, origin_y) = camera.get_viewport_origin();
        assert_eq!(origin_x, 45); // 50 - 5
        assert_eq!(origin_y, 45); // 50 - 5
    }

    #[test]
    fn test_is_visible() {
        let mut camera = Camera::new(10, 10);
        camera.center_x = 50.0;
        camera.center_y = 50.0;

        // Center should be visible
        assert!(camera.is_visible(50, 50));

        // Edges should be visible
        assert!(camera.is_visible(45, 50)); // left edge
        assert!(camera.is_visible(55, 50)); // right edge
        assert!(camera.is_visible(50, 45)); // top edge
        assert!(camera.is_visible(50, 55)); // bottom edge

        // Outside should not be visible
        assert!(!camera.is_visible(44, 50));
        assert!(!camera.is_visible(56, 50));
        assert!(!camera.is_visible(50, 44));
        assert!(!camera.is_visible(50, 56));
    }

    #[test]
    fn test_world_to_screen() {
        let mut camera = Camera::new(10, 10);
        camera.center_x = 50.0;
        camera.center_y = 50.0;

        // Center of camera should map to center of screen
        let screen = camera.world_to_screen(50, 50);
        assert_eq!(screen, Some((5, 5)));

        // Top-left of viewport
        let screen = camera.world_to_screen(45, 45);
        assert_eq!(screen, Some((0, 0)));

        // Outside viewport
        let screen = camera.world_to_screen(0, 0);
        assert_eq!(screen, None);
    }

    #[test]
    fn test_screen_to_world() {
        let mut camera = Camera::new(10, 10);
        camera.center_x = 50.0;
        camera.center_y = 50.0;

        // Center of screen should map to center of camera
        let (world_x, world_y) = camera.screen_to_world(5, 5);
        assert_eq!(world_x, 50);
        assert_eq!(world_y, 50);

        // Top-left of screen
        let (world_x, world_y) = camera.screen_to_world(0, 0);
        assert_eq!(world_x, 45);
        assert_eq!(world_y, 45);
    }

    #[test]
    fn test_get_visible_tiles() {
        let map = Map::new("test".to_string(), 100, 100);
        let mut camera = Camera::new(5, 5);
        camera.center_x = 50.0;
        camera.center_y = 50.0;

        let visible = camera.get_visible_tiles(&map);

        // Should have (2*2+1) * (2*2+1) = 25 tiles (5x5 viewport)
        // Actually with half_w=2, half_h=2, we get from -2 to +2 inclusive = 5x5 = 25
        assert!(!visible.is_empty());

        // Check that center tile is at correct position
        let center_tile = visible
            .iter()
            .find(|t| t.world_x == 50 && t.world_y == 50);
        assert!(center_tile.is_some());
    }

    #[test]
    fn test_get_visible_grid() {
        let map = Map::new("test".to_string(), 100, 100);
        let mut camera = Camera::new(5, 5);
        camera.center_x = 50.0;
        camera.center_y = 50.0;

        let grid = camera.get_visible_grid(&map);

        // Grid dimensions should match viewport
        assert_eq!(grid.len(), 5); // height
        assert_eq!(grid[0].len(), 5); // width

        // All tiles should be present
        for row in &grid {
            for tile in row {
                assert!(tile.is_some());
            }
        }
    }

    #[test]
    fn test_clamp_to_map() {
        let map = Map::new("test".to_string(), 100, 100);
        let mut camera = Camera::new(20, 20);

        // Try to go too far left/up
        camera.center_x = 0.0;
        camera.center_y = 0.0;
        camera.clamp_to_map(&map);
        assert!(camera.center_x >= 10.0); // half width
        assert!(camera.center_y >= 10.0); // half height

        // Try to go too far right/down
        camera.center_x = 99.0;
        camera.center_y = 99.0;
        camera.clamp_to_map(&map);
        assert!(camera.center_x <= 89.0); // width - half_width - 1
        assert!(camera.center_y <= 89.0);
    }

    #[test]
    fn test_clamp_not_applied_to_overworld() {
        let mut map = Map::new("world".to_string(), 100, 100);
        map.map_type = MapType::Overworld;

        let mut camera = Camera::new(20, 20);
        camera.center_x = 0.0;
        camera.center_y = 0.0;

        camera.clamp_to_map(&map);

        // Should not be clamped
        assert_eq!(camera.center_x, 0.0);
        assert_eq!(camera.center_y, 0.0);
    }

    #[test]
    fn test_visible_tiles_at_map_edge() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = MapType::Town;

        let mut camera = Camera::new(5, 5);
        camera.center_x = 0.0;
        camera.center_y = 0.0;

        let grid = camera.get_visible_grid(&map);

        // Some tiles should be None (outside map bounds)
        let has_none = grid.iter().any(|row| row.iter().any(|t| t.is_none()));
        assert!(has_none);
    }

    #[test]
    fn test_visible_tiles_overworld_wrap() {
        let mut map = Map::new("world".to_string(), 10, 10);
        map.map_type = MapType::Overworld;

        let mut camera = Camera::new(5, 5);
        camera.center_x = 0.0;
        camera.center_y = 0.0;

        let grid = camera.get_visible_grid(&map);

        // All tiles should be present (wrapped)
        for row in &grid {
            for tile in row {
                assert!(tile.is_some());
            }
        }
    }
}
