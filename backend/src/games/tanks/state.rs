//! Game state for Tanks
//!
//! Manages individual tank state and overall game state.

use std::collections::HashMap;
use std::time::Instant;
use serde::{Serialize, Deserialize};

use super::data::{TANK_MAX_HEALTH, MAX_POWER, MIN_POWER, MAX_ANGLE, MIN_ANGLE, MAX_WIND, get_weapon, WeaponDef};
use super::terrain::{Terrain, TerrainStyle, generate_terrain};
use super::physics::{simulate_projectile, ProjectileResult, TankPosition, Explosion};

/// Weapon type with ammo count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponType {
    pub key: String,
    pub ammo: Option<u32>, // None = unlimited
}

impl WeaponType {
    pub fn standard() -> Self {
        Self {
            key: "standard".to_string(),
            ammo: None,
        }
    }

    pub fn with_ammo(key: &str, ammo: u32) -> Self {
        Self {
            key: key.to_string(),
            ammo: Some(ammo),
        }
    }

    pub fn use_ammo(&mut self) -> bool {
        match &mut self.ammo {
            None => true, // Unlimited
            Some(count) if *count > 0 => {
                *count -= 1;
                true
            }
            _ => false,
        }
    }
}

/// Individual tank state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TankState {
    pub user_id: i64,
    pub handle: String,
    pub health: u32,
    pub max_health: u32,
    pub x: usize,
    pub y: usize,
    pub angle: i32,        // 0-180 degrees
    pub power: u32,        // 0-100 percentage
    pub facing_right: bool,
    pub weapons: Vec<WeaponType>,
    pub current_weapon: usize,
    pub shield: u32,
    pub fuel: u32,
    pub is_alive: bool,
    pub is_connected: bool,
    pub kills: u32,
    pub damage_dealt: u32,
    pub shots_fired: u32,
    pub shots_hit: u32,
}

impl TankState {
    pub fn new(user_id: i64, handle: String, x: usize, y: usize, facing_right: bool) -> Self {
        Self {
            user_id,
            handle,
            health: TANK_MAX_HEALTH,
            max_health: TANK_MAX_HEALTH,
            x,
            y,
            angle: if facing_right { 45 } else { 135 },
            power: 50,
            facing_right,
            weapons: vec![
                WeaponType::standard(),
                WeaponType::with_ammo("baby_missile", 5),
                WeaponType::with_ammo("heavy_shell", 3),
            ],
            current_weapon: 0,
            shield: 0,
            fuel: 100,
            is_alive: true,
            is_connected: true,
            kills: 0,
            damage_dealt: 0,
            shots_fired: 0,
            shots_hit: 0,
        }
    }

    pub fn get_current_weapon(&self) -> Option<&WeaponType> {
        self.weapons.get(self.current_weapon)
    }

    pub fn get_current_weapon_def(&self) -> Option<&'static WeaponDef> {
        self.get_current_weapon().and_then(|w| get_weapon(&w.key))
    }

    pub fn adjust_angle(&mut self, delta: i32) {
        self.angle = (self.angle + delta).clamp(MIN_ANGLE, MAX_ANGLE);
    }

    pub fn adjust_power(&mut self, delta: i32) {
        self.power = ((self.power as i32 + delta).clamp(MIN_POWER as i32, MAX_POWER as i32)) as u32;
    }

    pub fn cycle_weapon(&mut self, forward: bool) {
        if self.weapons.is_empty() {
            return;
        }
        if forward {
            self.current_weapon = (self.current_weapon + 1) % self.weapons.len();
        } else {
            self.current_weapon = if self.current_weapon == 0 {
                self.weapons.len() - 1
            } else {
                self.current_weapon - 1
            };
        }
    }

    pub fn take_damage(&mut self, damage: u32) -> u32 {
        let actual_damage = if self.shield > 0 {
            let absorbed = damage.min(self.shield);
            self.shield -= absorbed;
            damage - absorbed
        } else {
            damage
        };

        self.health = self.health.saturating_sub(actual_damage);
        if self.health == 0 {
            self.is_alive = false;
        }

        actual_damage
    }

    pub fn fire(&mut self) -> bool {
        if !self.is_alive {
            return false;
        }

        if let Some(weapon) = self.weapons.get_mut(self.current_weapon) {
            if weapon.use_ammo() {
                self.shots_fired += 1;
                return true;
            }
        }
        false
    }
}

/// Game phase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GamePhase {
    Lobby,
    Starting,
    InProgress,
    TurnActive { current_tank: u64 },
    TurnResult { current_tank: u64, result: Option<String> },
    GameOver,
}

/// Main game state
#[derive(Debug, Clone)]
pub struct TankGame {
    pub id: u64,
    pub host_user_id: i64,
    pub phase: GamePhase,
    pub tanks: HashMap<u64, TankState>,
    pub terrain: Terrain,
    pub wind: i32,
    pub turn_order: Vec<u64>,
    pub current_turn_index: usize,
    pub round: u32,
    #[allow(dead_code)]
    pub max_rounds: u32,
    pub last_result: Option<ProjectileResult>,
    pub turn_started_at: Option<Instant>,
    pub created_at: Instant,
    next_tank_id: u64,
}

impl TankGame {
    pub fn new(id: u64, host_user_id: i64, terrain_seed: u64) -> Self {
        Self {
            id,
            host_user_id,
            phase: GamePhase::Lobby,
            tanks: HashMap::new(),
            terrain: generate_terrain(TerrainStyle::Random, terrain_seed),
            wind: 0,
            turn_order: Vec::new(),
            current_turn_index: 0,
            round: 1,
            max_rounds: 10,
            last_result: None,
            turn_started_at: None,
            created_at: Instant::now(),
            next_tank_id: 1,
        }
    }

    /// Add a tank for a player
    pub fn add_tank(&mut self, user_id: i64, handle: String) -> Result<u64, &'static str> {
        if self.phase != GamePhase::Lobby {
            return Err("Game already started");
        }

        if self.tanks.values().any(|t| t.user_id == user_id) {
            return Err("Already in game");
        }

        let tank_id = self.next_tank_id;
        self.next_tank_id += 1;

        // Position tanks spread across the field
        let num_tanks = self.tanks.len();
        let spacing = 80 / (num_tanks + 2);
        let x = spacing * (num_tanks + 1);
        let y = self.terrain.ground_level(x).saturating_sub(1);
        let facing_right = num_tanks % 2 == 0;

        let tank = TankState::new(user_id, handle, x, y, facing_right);
        self.tanks.insert(tank_id, tank);
        self.turn_order.push(tank_id);

        Ok(tank_id)
    }

    /// Remove a tank/player
    pub fn remove_tank(&mut self, tank_id: u64) {
        self.tanks.remove(&tank_id);
        self.turn_order.retain(|&id| id != tank_id);

        // Check if game should end
        self.check_game_over();
    }

    /// Start the game
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.tanks.len() < 2 {
            return Err("Need at least 2 players");
        }

        // Reposition tanks evenly across terrain
        self.reposition_tanks();

        // Randomize turn order
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.turn_order.shuffle(&mut rng);

        // Set initial wind
        self.randomize_wind();

        self.phase = GamePhase::Starting;
        Ok(())
    }

    fn reposition_tanks(&mut self) {
        let num_tanks = self.tanks.len();
        if num_tanks == 0 {
            return;
        }

        let spacing = 80 / (num_tanks + 1);
        let tank_ids: Vec<u64> = self.tanks.keys().cloned().collect();

        for (i, tank_id) in tank_ids.iter().enumerate() {
            if let Some(tank) = self.tanks.get_mut(tank_id) {
                let x = spacing * (i + 1);
                let y = self.terrain.ground_level(x).saturating_sub(1);
                tank.x = x;
                tank.y = y.max(1);
                tank.facing_right = i % 2 == 0;
                tank.angle = if tank.facing_right { 45 } else { 135 };
            }
        }
    }

    pub fn randomize_wind(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.wind = rng.gen_range(-MAX_WIND..=MAX_WIND);
    }

    /// Begin a player's turn
    pub fn begin_turn(&mut self) {
        if self.turn_order.is_empty() {
            return;
        }

        // Skip dead tanks
        while self.current_turn_index < self.turn_order.len() {
            let tank_id = self.turn_order[self.current_turn_index];
            if let Some(tank) = self.tanks.get(&tank_id) {
                if tank.is_alive && tank.is_connected {
                    break;
                }
            }
            self.current_turn_index += 1;
        }

        // Wrap around if needed
        if self.current_turn_index >= self.turn_order.len() {
            self.current_turn_index = 0;
            self.round += 1;
            self.randomize_wind();
            return self.begin_turn();
        }

        let tank_id = self.turn_order[self.current_turn_index];
        self.phase = GamePhase::TurnActive { current_tank: tank_id };
        self.turn_started_at = Some(Instant::now());
        self.last_result = None;
    }

    /// Execute a fire action
    pub fn fire(&mut self, tank_id: u64) -> Option<ProjectileResult> {
        // First, collect all the info we need from the firing tank
        let (weapon_key, start_x, start_y, angle, power) = {
            let tank = self.tanks.get_mut(&tank_id)?;
            if !tank.fire() {
                return None;
            }
            let weapon_key = tank.get_current_weapon()?.key.clone();
            let start_x = tank.x as f64;
            let start_y = (tank.y as f64) - 1.0;
            let angle = tank.angle;
            let power = tank.power;
            (weapon_key, start_x, start_y, angle, power)
        };

        // Get tank positions for collision
        let tank_positions: Vec<TankPosition> = self.tanks.iter()
            .filter(|(_, t)| t.is_alive)
            .map(|(&id, t)| TankPosition { id, x: t.x, y: t.y })
            .collect();

        let result = simulate_projectile(
            start_x,
            start_y,
            angle,
            power,
            &weapon_key,
            self.wind,
            &self.terrain,
            &tank_positions,
        );

        // Apply explosion
        if let Some(ref explosion) = result.explosion {
            self.apply_explosion(explosion, tank_id);
        }

        // Apply damage - collect hits first, then apply
        let hits_to_apply: Vec<(u64, u32)> = result.hits.clone();
        let firing_tank_id = tank_id;

        for (hit_tank_id, damage) in hits_to_apply {
            // Apply damage to hit tank
            let (actual_damage, hit_alive) = {
                if let Some(hit_tank) = self.tanks.get_mut(&hit_tank_id) {
                    let actual = hit_tank.take_damage(damage);
                    (actual, hit_tank.is_alive)
                } else {
                    continue;
                }
            };

            // Update firing tank stats
            if let Some(firing_tank) = self.tanks.get_mut(&firing_tank_id) {
                firing_tank.damage_dealt += actual_damage;
                if actual_damage > 0 {
                    firing_tank.shots_hit += 1;
                }
                if !hit_alive && hit_tank_id != firing_tank_id {
                    firing_tank.kills += 1;
                }
            }
        }

        self.last_result = Some(result.clone());

        // Transition to result phase
        self.phase = GamePhase::TurnResult {
            current_tank: tank_id,
            result: Some("Boom!".to_string()),
        };

        // Check for game over
        self.check_game_over();

        Some(result)
    }

    fn apply_explosion(&mut self, explosion: &Explosion, _firing_tank_id: u64) {
        if explosion.is_dirt_bomb {
            self.terrain.add_circle(explosion.x, explosion.y, explosion.radius);
        } else {
            self.terrain.destroy_circle(explosion.x, explosion.y, explosion.radius);
        }

        // Apply gravity to terrain
        while self.terrain.apply_gravity() {}

        // Update tank positions if terrain fell from under them
        self.update_tank_positions();
    }

    fn update_tank_positions(&mut self) {
        for tank in self.tanks.values_mut() {
            if !tank.is_alive {
                continue;
            }

            // If tank is floating, drop it
            while tank.y + 1 < self.terrain.height && !self.terrain.is_solid(tank.x, tank.y + 1) {
                tank.y += 1;
            }

            // If tank fell into terrain (buried), damage it
            if self.terrain.is_solid(tank.x, tank.y) {
                tank.take_damage(10);
            }
        }
    }

    /// Advance to next turn
    pub fn next_turn(&mut self) {
        self.current_turn_index += 1;
        if self.current_turn_index >= self.turn_order.len() {
            self.current_turn_index = 0;
            self.round += 1;
            self.randomize_wind();
        }
        self.begin_turn();
    }

    fn check_game_over(&mut self) {
        let alive: Vec<_> = self.tanks.values().filter(|t| t.is_alive).collect();

        if alive.len() <= 1 {
            self.phase = GamePhase::GameOver;
        }
    }

    /// Get the winner (if game is over)
    pub fn get_winner(&self) -> Option<&TankState> {
        if self.phase != GamePhase::GameOver {
            return None;
        }
        self.tanks.values().find(|t| t.is_alive)
    }

    /// Get current tank (whose turn it is)
    pub fn get_current_tank(&self) -> Option<&TankState> {
        match &self.phase {
            GamePhase::TurnActive { current_tank } |
            GamePhase::TurnResult { current_tank, .. } => {
                self.tanks.get(current_tank)
            }
            _ => None,
        }
    }

    /// Get tank by user_id
    pub fn get_tank_by_user(&self, user_id: i64) -> Option<(u64, &TankState)> {
        self.tanks.iter().find(|(_, t)| t.user_id == user_id).map(|(&id, t)| (id, t))
    }

    /// Get mutable tank by user_id
    pub fn get_tank_by_user_mut(&mut self, user_id: i64) -> Option<(u64, &mut TankState)> {
        self.tanks.iter_mut().find(|(_, t)| t.user_id == user_id).map(|(id, t)| (*id, t))
    }

    /// Disconnect a player
    pub fn disconnect_player(&mut self, user_id: i64) {
        if let Some((_, tank)) = self.get_tank_by_user_mut(user_id) {
            tank.is_connected = false;
        }

        // Check if should skip to next turn
        if let GamePhase::TurnActive { current_tank } = self.phase {
            if let Some(tank) = self.tanks.get(&current_tank) {
                if tank.user_id == user_id {
                    self.next_turn();
                }
            }
        }

        // Check for game over
        let connected_alive = self.tanks.values()
            .filter(|t| t.is_alive && t.is_connected)
            .count();

        if connected_alive < 2 && self.phase != GamePhase::Lobby {
            self.phase = GamePhase::GameOver;
        }
    }

    /// Reconnect a player
    pub fn reconnect_player(&mut self, user_id: i64) -> bool {
        if let Some((_, tank)) = self.get_tank_by_user_mut(user_id) {
            tank.is_connected = true;
            true
        } else {
            false
        }
    }

    /// Get game stats for display
    pub fn get_standings(&self) -> Vec<(String, u32, u32, bool)> {
        let mut standings: Vec<_> = self.tanks.values()
            .map(|t| (t.handle.clone(), t.kills, t.damage_dealt, t.is_alive))
            .collect();
        standings.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        standings
    }
}

/// Serializable game state for database persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub game_id: u64,
    pub tanks: HashMap<u64, TankState>,
    pub terrain_seed: u64,
    pub wind: i32,
    pub turn_order: Vec<u64>,
    pub current_turn_index: usize,
    pub round: u32,
    pub phase_name: String,
}

impl From<&TankGame> for GameState {
    fn from(game: &TankGame) -> Self {
        Self {
            game_id: game.id,
            tanks: game.tanks.clone(),
            terrain_seed: 0, // Would need to store actual seed
            wind: game.wind,
            turn_order: game.turn_order.clone(),
            current_turn_index: game.current_turn_index,
            round: game.round,
            phase_name: match &game.phase {
                GamePhase::Lobby => "lobby".to_string(),
                GamePhase::Starting => "starting".to_string(),
                GamePhase::InProgress => "in_progress".to_string(),
                GamePhase::TurnActive { .. } => "turn_active".to_string(),
                GamePhase::TurnResult { .. } => "turn_result".to_string(),
                GamePhase::GameOver => "game_over".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_game() -> TankGame {
        TankGame::new(1, 100, 12345)
    }

    #[test]
    fn test_add_tank() {
        let mut game = create_test_game();
        let id1 = game.add_tank(1, "Player1".to_string()).unwrap();
        let id2 = game.add_tank(2, "Player2".to_string()).unwrap();

        assert!(id1 != id2);
        assert_eq!(game.tanks.len(), 2);
    }

    #[test]
    fn test_duplicate_player() {
        let mut game = create_test_game();
        game.add_tank(1, "Player1".to_string()).unwrap();
        let result = game.add_tank(1, "Player1Again".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_start_game() {
        let mut game = create_test_game();
        game.add_tank(1, "Player1".to_string()).unwrap();
        game.add_tank(2, "Player2".to_string()).unwrap();

        assert!(game.start().is_ok());
        assert_eq!(game.phase, GamePhase::Starting);
    }

    #[test]
    fn test_start_needs_two_players() {
        let mut game = create_test_game();
        game.add_tank(1, "Player1".to_string()).unwrap();

        let result = game.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_tank_damage() {
        let mut tank = TankState::new(1, "Test".to_string(), 40, 10, true);
        tank.take_damage(30);
        assert_eq!(tank.health, 70);
        assert!(tank.is_alive);

        tank.take_damage(80);
        assert_eq!(tank.health, 0);
        assert!(!tank.is_alive);
    }

    #[test]
    fn test_shield_absorbs_damage() {
        let mut tank = TankState::new(1, "Test".to_string(), 40, 10, true);
        tank.shield = 20;
        let actual = tank.take_damage(30);
        assert_eq!(actual, 10); // 20 absorbed by shield
        assert_eq!(tank.health, 90);
        assert_eq!(tank.shield, 0);
    }

    #[test]
    fn test_weapon_cycling() {
        let mut tank = TankState::new(1, "Test".to_string(), 40, 10, true);
        assert_eq!(tank.current_weapon, 0);

        tank.cycle_weapon(true);
        assert_eq!(tank.current_weapon, 1);

        tank.cycle_weapon(false);
        assert_eq!(tank.current_weapon, 0);
    }

    #[test]
    fn test_angle_clamping() {
        let mut tank = TankState::new(1, "Test".to_string(), 40, 10, true);
        tank.angle = 90;

        tank.adjust_angle(100);
        assert_eq!(tank.angle, MAX_ANGLE);

        tank.adjust_angle(-200);
        assert_eq!(tank.angle, MIN_ANGLE);
    }

    #[test]
    fn test_game_over_detection() {
        let mut game = create_test_game();
        let id1 = game.add_tank(1, "P1".to_string()).unwrap();
        let _id2 = game.add_tank(2, "P2".to_string()).unwrap();
        game.start().unwrap();
        game.begin_turn();

        // Kill one tank
        if let Some(tank) = game.tanks.get_mut(&id1) {
            tank.take_damage(100);
        }
        game.check_game_over();

        assert_eq!(game.phase, GamePhase::GameOver);
    }
}
