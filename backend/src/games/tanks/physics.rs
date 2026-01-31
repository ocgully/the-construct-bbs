//! Projectile physics simulation for Tanks
//!
//! Handles projectile trajectories with gravity and wind, collision detection,
//! and explosion mechanics.

use super::data::{GRAVITY, FIELD_WIDTH, FIELD_HEIGHT, get_weapon};
use super::terrain::Terrain;

/// Projectile state during simulation
#[derive(Debug, Clone)]
pub struct Projectile {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub weapon: String,
}

impl Projectile {
    /// Create a new projectile from firing parameters
    /// angle: 0 = right, 90 = up, 180 = left
    /// power: 0-100 percentage of max velocity
    pub fn new(start_x: f64, start_y: f64, angle: i32, power: u32, weapon: &str) -> Self {
        let angle_rad = (angle as f64).to_radians();
        let power_factor = power as f64 / 100.0;
        let max_velocity = 3.0; // Tuned for 80x20 field

        Self {
            x: start_x,
            y: start_y,
            vx: angle_rad.cos() * max_velocity * power_factor,
            vy: -angle_rad.sin() * max_velocity * power_factor, // Negative because y increases downward
            weapon: weapon.to_string(),
        }
    }

    /// Advance projectile one time step with wind
    pub fn step(&mut self, wind: i32) {
        let wind_factor = wind as f64 * 0.01;

        self.vx += wind_factor;
        self.vy += GRAVITY;

        self.x += self.vx;
        self.y += self.vy;
    }
}

/// Result of projectile impact
#[derive(Debug, Clone)]
pub struct Explosion {
    pub x: i32,
    pub y: i32,
    pub radius: u32,
    pub damage: u32,
    pub is_dirt_bomb: bool,
}

/// Full projectile simulation result
#[derive(Debug, Clone)]
pub struct ProjectileResult {
    /// Path the projectile took (for rendering trail)
    pub path: Vec<(i32, i32)>,
    /// Explosion that occurred (if any)
    pub explosion: Option<Explosion>,
    /// Tanks that were hit (tank_id, damage)
    pub hits: Vec<(u64, u32)>,
    /// Whether projectile went out of bounds
    pub out_of_bounds: bool,
}

/// Position of a tank on the field
#[derive(Debug, Clone)]
pub struct TankPosition {
    pub id: u64,
    pub x: usize,
    pub y: usize,
}

/// Simulate a projectile from firing to impact
pub fn simulate_projectile(
    start_x: f64,
    start_y: f64,
    angle: i32,
    power: u32,
    weapon: &str,
    wind: i32,
    terrain: &Terrain,
    tanks: &[TankPosition],
) -> ProjectileResult {
    let mut projectile = Projectile::new(start_x, start_y, angle, power, weapon);
    let mut path = Vec::new();

    let weapon_def = get_weapon(weapon).unwrap_or_else(|| get_weapon("standard").unwrap());
    let max_steps = 500; // Prevent infinite loops

    for _ in 0..max_steps {
        // Record position for trail
        let ix = projectile.x.round() as i32;
        let iy = projectile.y.round() as i32;

        // Check out of bounds
        if ix < 0 || ix >= FIELD_WIDTH as i32 || iy >= FIELD_HEIGHT as i32 {
            // Allow going above screen (y < 0) for high arcs
            if iy >= FIELD_HEIGHT as i32 || ix < 0 || ix >= FIELD_WIDTH as i32 {
                return ProjectileResult {
                    path,
                    explosion: None,
                    hits: Vec::new(),
                    out_of_bounds: true,
                };
            }
        }

        if iy >= 0 {
            path.push((ix, iy));
        }

        // Check terrain collision
        if ix >= 0 && iy >= 0 && (ix as usize) < terrain.width && (iy as usize) < terrain.height {
            if terrain.is_solid(ix as usize, iy as usize) {
                let explosion = Explosion {
                    x: ix,
                    y: iy,
                    radius: weapon_def.explosion_radius,
                    damage: weapon_def.damage,
                    is_dirt_bomb: weapon == "dirt_bomb",
                };

                // Calculate hits on tanks
                let hits = calculate_tank_hits(&explosion, tanks);

                return ProjectileResult {
                    path,
                    explosion: Some(explosion),
                    hits,
                    out_of_bounds: false,
                };
            }
        }

        // Check tank collision (direct hit)
        for tank in tanks {
            let dx = (tank.x as i32 - ix).abs();
            let dy = (tank.y as i32 - iy).abs();
            if dx <= 2 && dy <= 1 {
                let explosion = Explosion {
                    x: ix,
                    y: iy,
                    radius: weapon_def.explosion_radius,
                    damage: weapon_def.damage,
                    is_dirt_bomb: weapon == "dirt_bomb",
                };

                let hits = calculate_tank_hits(&explosion, tanks);

                return ProjectileResult {
                    path,
                    explosion: Some(explosion),
                    hits,
                    out_of_bounds: false,
                };
            }
        }

        // Advance simulation
        projectile.step(wind);
    }

    // Ran out of steps (shouldn't happen normally)
    ProjectileResult {
        path,
        explosion: None,
        hits: Vec::new(),
        out_of_bounds: true,
    }
}

/// Calculate damage to tanks from an explosion
fn calculate_tank_hits(explosion: &Explosion, tanks: &[TankPosition]) -> Vec<(u64, u32)> {
    let mut hits = Vec::new();

    for tank in tanks {
        let dx = (tank.x as i32 - explosion.x).abs();
        let dy = (tank.y as i32 - explosion.y).abs();
        let distance = ((dx * dx + dy * dy) as f64).sqrt();

        if distance <= explosion.radius as f64 {
            // Damage falls off with distance
            let damage_factor = 1.0 - (distance / (explosion.radius as f64 + 1.0));
            let damage = (explosion.damage as f64 * damage_factor) as u32;
            if damage > 0 {
                hits.push((tank.id, damage));
            }
        }
    }

    hits
}

/// Calculate aim assistance - shows where projectile would land
pub fn calculate_aim_preview(
    start_x: f64,
    start_y: f64,
    angle: i32,
    power: u32,
    wind: i32,
    max_points: usize,
) -> Vec<(i32, i32)> {
    let mut projectile = Projectile::new(start_x, start_y, angle, power, "standard");
    let mut points = Vec::new();
    let step_interval = 3; // Only record every Nth point

    for step in 0..(max_points * step_interval) {
        projectile.step(wind);

        if step % step_interval == 0 {
            let ix = projectile.x.round() as i32;
            let iy = projectile.y.round() as i32;

            // Stop if out of horizontal bounds
            if ix < 0 || ix >= FIELD_WIDTH as i32 || iy >= FIELD_HEIGHT as i32 {
                break;
            }

            if iy >= 0 {
                points.push((ix, iy));
            }
        }
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::terrain::TerrainType;

    #[test]
    fn test_projectile_creation() {
        let p = Projectile::new(40.0, 10.0, 45, 50, "standard");
        assert!(p.vx > 0.0); // Moving right
        assert!(p.vy < 0.0); // Moving up (negative y)
    }

    #[test]
    fn test_projectile_step() {
        let mut p = Projectile::new(40.0, 10.0, 45, 50, "standard");
        let initial_y = p.y;
        p.step(0);
        // Gravity should pull down (increase y)
        // But initial upward velocity means it might still go up first
        assert!(p.y != initial_y || p.vy != 0.0);
    }

    #[test]
    fn test_projectile_hits_ground() {
        let mut terrain = Terrain::new(80, 20);
        // Fill bottom half with dirt
        for y in 10..20 {
            for x in 0..80 {
                terrain.set(x, y, TerrainType::Dirt);
            }
        }

        let result = simulate_projectile(
            10.0, 5.0,  // Start position
            45, 50,     // Angle and power
            "standard",
            0,          // No wind
            &terrain,
            &[],        // No tanks
        );

        assert!(result.explosion.is_some());
        assert!(!result.out_of_bounds);
    }

    #[test]
    fn test_projectile_out_of_bounds() {
        let terrain = Terrain::new(80, 20); // Empty terrain

        let result = simulate_projectile(
            40.0, 5.0,
            30, 80,     // Low angle, high power - should go right
            "standard",
            0,
            &terrain,
            &[],
        );

        // Should either hit ground or go out of bounds
        assert!(result.out_of_bounds || result.explosion.is_some());
    }

    #[test]
    fn test_tank_damage_calculation() {
        let explosion = Explosion {
            x: 40,
            y: 15,
            radius: 3,
            damage: 25,
            is_dirt_bomb: false,
        };

        let tanks = vec![
            TankPosition { id: 1, x: 40, y: 15 }, // Direct hit
            TankPosition { id: 2, x: 42, y: 15 }, // Close
            TankPosition { id: 3, x: 50, y: 15 }, // Far away
        ];

        let hits = calculate_tank_hits(&explosion, &tanks);

        assert!(hits.len() >= 1); // At least direct hit
        let direct_hit = hits.iter().find(|(id, _)| *id == 1);
        assert!(direct_hit.is_some());
        assert!(direct_hit.unwrap().1 > 0);
    }

    #[test]
    fn test_wind_affects_trajectory() {
        let mut p1 = Projectile::new(40.0, 10.0, 90, 50, "standard"); // Straight up
        let mut p2 = Projectile::new(40.0, 10.0, 90, 50, "standard");

        // Simulate with different winds
        for _ in 0..20 {
            p1.step(-5); // Wind blowing left
            p2.step(5);  // Wind blowing right
        }

        // p1 should be to the left of p2
        assert!(p1.x < p2.x);
    }

    #[test]
    fn test_aim_preview() {
        let points = calculate_aim_preview(40.0, 10.0, 45, 50, 0, 10);
        assert!(!points.is_empty());
        // Points should generally trend right and eventually down
    }
}
