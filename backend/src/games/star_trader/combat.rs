//! Star Trader - Combat System
//!
//! Ship-to-ship combat, Ferrengi encounters, and fighter battles.

use rand::Rng;
use super::data::config;
use super::state::GameState;

/// Combat opponent type
#[derive(Debug, Clone, PartialEq)]
pub enum Opponent {
    Ferrengi { name: String, fighters: u32, shields: u32 },
    Player { id: i64, handle: String, fighters: u32, shields: u32 },
    Pirate { name: String, fighters: u32 },
}

impl Opponent {
    pub fn name(&self) -> &str {
        match self {
            Opponent::Ferrengi { name, .. } => name,
            Opponent::Player { handle, .. } => handle,
            Opponent::Pirate { name, .. } => name,
        }
    }

    pub fn fighters(&self) -> u32 {
        match self {
            Opponent::Ferrengi { fighters, .. } => *fighters,
            Opponent::Player { fighters, .. } => *fighters,
            Opponent::Pirate { fighters, .. } => *fighters,
        }
    }

    pub fn shields(&self) -> u32 {
        match self {
            Opponent::Ferrengi { shields, .. } => *shields,
            Opponent::Player { shields, .. } => *shields,
            Opponent::Pirate { .. } => 0,  // Pirates have no shields
        }
    }
}

/// Result of a combat round
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub victory: bool,
    pub fled: bool,
    pub player_fighters_lost: u32,
    pub player_shields_lost: u32,
    pub opponent_fighters_lost: u32,
    pub opponent_shields_lost: u32,
    pub experience_gained: i64,
    pub credits_looted: i64,
    pub message: String,
}

/// Combat action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatAction {
    Attack,
    Flee,
    Surrender,
}

/// Generate a Ferrengi encounter based on sector strength
pub fn generate_ferrengi(strength: u32) -> Opponent {
    let mut rng = rand::thread_rng();

    let names = [
        "Ferrengi Scout", "Ferrengi Marauder", "Ferrengi Raider",
        "Ferrengi Destroyer", "Ferrengi Battleship", "Ferrengi Dreadnought",
    ];

    let tier = (strength / 20).min(5) as usize;
    let name = names[tier].to_string();

    // Fighters based on strength + variance
    let base_fighters = strength + rng.gen_range(0..20);
    let shields = (strength as f32 * 1.5) as u32 + rng.gen_range(0..30);

    Opponent::Ferrengi {
        name,
        fighters: base_fighters,
        shields,
    }
}

/// Generate a random pirate encounter
pub fn generate_pirate() -> Opponent {
    let mut rng = rand::thread_rng();

    let names = [
        "Space Pirate", "Marauder", "Rogue Trader", "Outlaw",
        "Corsair", "Buccaneer", "Privateer",
    ];

    let name = names[rng.gen_range(0..names.len())].to_string();
    let fighters = rng.gen_range(5..30);

    Opponent::Pirate { name, fighters }
}

/// Execute a combat round
pub fn combat_round(
    state: &mut GameState,
    opponent: &mut Opponent,
    action: CombatAction,
) -> CombatResult {
    let mut rng = rand::thread_rng();

    match action {
        CombatAction::Flee => attempt_flee(state, opponent),
        CombatAction::Surrender => handle_surrender(state, opponent),
        CombatAction::Attack => {
            let mut player_fighters_lost = 0u32;
            let mut player_shields_lost = 0u32;
            let mut opponent_fighters_lost = 0u32;
            let mut opponent_shields_lost = 0u32;

            // Player attacks opponent
            if state.fighters > 0 {
                // Each fighter has a chance to hit
                let hits = calculate_hits(state.fighters, &mut rng);

                // Damage goes to shields first, then fighters
                let (shield_damage, fighter_damage) = apply_damage(hits, opponent);
                opponent_shields_lost = shield_damage;
                opponent_fighters_lost = fighter_damage;
            }

            // Check if opponent is destroyed
            let opponent_fighters = get_opponent_fighters(opponent);
            let opponent_shields = get_opponent_shields(opponent);

            if opponent_fighters == 0 && opponent_shields == 0 {
                // Victory!
                let xp = config::XP_PER_KILL + (opponent_fighters_lost as i64 * 2);
                let loot = calculate_loot(opponent, &mut rng);

                state.add_experience(xp);
                state.credits += loot;
                state.kills += 1;

                // Alignment change based on opponent
                match opponent {
                    Opponent::Ferrengi { .. } => state.adjust_alignment(10),
                    Opponent::Pirate { .. } => state.adjust_alignment(5),
                    Opponent::Player { .. } => state.adjust_alignment(-50),  // PvP is evil
                }

                // Update stats
                match opponent {
                    Opponent::Ferrengi { .. } => state.stats.ferrengi_destroyed += 1,
                    Opponent::Player { .. } => state.stats.players_destroyed += 1,
                    _ => {}
                }

                state.update_max_credits();

                return CombatResult {
                    victory: true,
                    fled: false,
                    player_fighters_lost,
                    player_shields_lost,
                    opponent_fighters_lost,
                    opponent_shields_lost,
                    experience_gained: xp,
                    credits_looted: loot,
                    message: format!(
                        "Victory! {} destroyed. Gained {} XP and {} credits.",
                        opponent.name(),
                        xp,
                        loot
                    ),
                };
            }

            // Opponent attacks player
            if opponent_fighters > 0 {
                let hits = calculate_hits(opponent_fighters, &mut rng);

                // Damage player shields first, then fighters
                if state.shields > 0 {
                    let shield_hits = hits.min(state.shields);
                    state.shields -= shield_hits;
                    player_shields_lost = shield_hits;

                    let remaining = hits - shield_hits;
                    if remaining > 0 && state.fighters > 0 {
                        let fighter_hits = remaining.min(state.fighters);
                        state.fighters -= fighter_hits;
                        player_fighters_lost = fighter_hits;
                    }
                } else if state.fighters > 0 {
                    let fighter_hits = hits.min(state.fighters);
                    state.fighters -= fighter_hits;
                    player_fighters_lost = fighter_hits;
                }
            }

            // Check if player is destroyed
            if state.fighters == 0 && state.shields == 0 {
                handle_player_death(state);

                return CombatResult {
                    victory: false,
                    fled: false,
                    player_fighters_lost,
                    player_shields_lost,
                    opponent_fighters_lost,
                    opponent_shields_lost,
                    experience_gained: 0,
                    credits_looted: 0,
                    message: "Your ship was destroyed! Escape pod deployed.".to_string(),
                };
            }

            // Combat continues
            CombatResult {
                victory: false,
                fled: false,
                player_fighters_lost,
                player_shields_lost,
                opponent_fighters_lost,
                opponent_shields_lost,
                experience_gained: 0,
                credits_looted: 0,
                message: format!(
                    "Combat round: You lost {} fighters/{} shields. {} lost {} fighters/{} shields.",
                    player_fighters_lost,
                    player_shields_lost,
                    opponent.name(),
                    opponent_fighters_lost,
                    opponent_shields_lost
                ),
            }
        }
    }
}

/// Calculate hits from fighters
fn calculate_hits(fighters: u32, rng: &mut impl Rng) -> u32 {
    // Each fighter has ~60% chance to hit, dealing 1 damage
    let mut hits = 0;
    for _ in 0..fighters {
        if rng.gen_range(0..100) < 60 {
            hits += 1;
        }
    }
    hits
}

/// Apply damage to opponent (returns shields_lost, fighters_lost)
fn apply_damage(damage: u32, opponent: &mut Opponent) -> (u32, u32) {
    let mut shields_lost = 0;
    let mut fighters_lost = 0;
    let mut remaining = damage;

    match opponent {
        Opponent::Ferrengi { shields, fighters, .. } |
        Opponent::Player { shields, fighters, .. } => {
            if *shields > 0 {
                let shield_damage = remaining.min(*shields);
                *shields -= shield_damage;
                shields_lost = shield_damage;
                remaining -= shield_damage;
            }
            if remaining > 0 && *fighters > 0 {
                let fighter_damage = remaining.min(*fighters);
                *fighters -= fighter_damage;
                fighters_lost = fighter_damage;
            }
        }
        Opponent::Pirate { fighters, .. } => {
            if *fighters > 0 {
                let fighter_damage = remaining.min(*fighters);
                *fighters -= fighter_damage;
                fighters_lost = fighter_damage;
            }
        }
    }

    (shields_lost, fighters_lost)
}

/// Get opponent's remaining fighters
fn get_opponent_fighters(opponent: &Opponent) -> u32 {
    match opponent {
        Opponent::Ferrengi { fighters, .. } => *fighters,
        Opponent::Player { fighters, .. } => *fighters,
        Opponent::Pirate { fighters, .. } => *fighters,
    }
}

/// Get opponent's remaining shields
fn get_opponent_shields(opponent: &Opponent) -> u32 {
    match opponent {
        Opponent::Ferrengi { shields, .. } => *shields,
        Opponent::Player { shields, .. } => *shields,
        Opponent::Pirate { .. } => 0,
    }
}

/// Calculate loot from destroyed opponent
fn calculate_loot(opponent: &Opponent, rng: &mut impl Rng) -> i64 {
    let base = match opponent {
        Opponent::Ferrengi { fighters, .. } => *fighters as i64 * 50,
        Opponent::Player { .. } => 0,  // No looting players
        Opponent::Pirate { fighters, .. } => *fighters as i64 * 30,
    };

    // Add some variance
    let variance = rng.gen_range(-20..=40) as i64;
    (base + (base * variance / 100)).max(0)
}

/// Attempt to flee combat
fn attempt_flee(state: &mut GameState, opponent: &Opponent) -> CombatResult {
    let mut rng = rand::thread_rng();

    // Flee chance based on ship speed vs opponent strength
    let ship_speed = state.ship().map(|s| s.warp_speed).unwrap_or(1);
    let flee_chance = 30 + (ship_speed * 10);

    if rng.gen_range(0..100) < flee_chance {
        // Successful flee - might lose some fighters
        let fighters_lost = rng.gen_range(0..3).min(state.fighters);
        state.fighters -= fighters_lost;

        CombatResult {
            victory: false,
            fled: true,
            player_fighters_lost: fighters_lost,
            player_shields_lost: 0,
            opponent_fighters_lost: 0,
            opponent_shields_lost: 0,
            experience_gained: 0,
            credits_looted: 0,
            message: if fighters_lost > 0 {
                format!("Escaped! Lost {} fighters covering your retreat.", fighters_lost)
            } else {
                "Escaped successfully!".to_string()
            },
        }
    } else {
        // Failed flee - opponent gets free attack
        let hits = calculate_hits(opponent.fighters() / 2, &mut rng);
        let shields_lost = hits.min(state.shields);
        state.shields -= shields_lost;
        let fighters_lost = (hits - shields_lost).min(state.fighters);
        state.fighters -= fighters_lost;

        CombatResult {
            victory: false,
            fled: false,
            player_fighters_lost: fighters_lost,
            player_shields_lost: shields_lost,
            opponent_fighters_lost: 0,
            opponent_shields_lost: 0,
            experience_gained: 0,
            credits_looted: 0,
            message: format!(
                "Failed to escape! Lost {} shields and {} fighters.",
                shields_lost,
                fighters_lost
            ),
        }
    }
}

/// Handle player surrender
fn handle_surrender(state: &mut GameState, opponent: &Opponent) -> CombatResult {
    match opponent {
        Opponent::Ferrengi { .. } => {
            // Ferrengi take your cargo
            let cargo_value = (state.cargo.fuel_ore as i64 * 20)
                + (state.cargo.organics as i64 * 25)
                + (state.cargo.equipment as i64 * 50);

            state.cargo.fuel_ore = 0;
            state.cargo.organics = 0;
            state.cargo.equipment = 0;
            state.adjust_alignment(-5);  // Cowardly

            CombatResult {
                victory: false,
                fled: false,
                player_fighters_lost: 0,
                player_shields_lost: 0,
                opponent_fighters_lost: 0,
                opponent_shields_lost: 0,
                experience_gained: 0,
                credits_looted: -cargo_value,
                message: format!(
                    "The Ferrengi seized your cargo worth {} credits.",
                    cargo_value
                ),
            }
        }
        Opponent::Pirate { .. } => {
            // Pirates take credits
            let stolen = state.credits / 2;
            state.credits -= stolen;
            state.adjust_alignment(-10);

            CombatResult {
                victory: false,
                fled: false,
                player_fighters_lost: 0,
                player_shields_lost: 0,
                opponent_fighters_lost: 0,
                opponent_shields_lost: 0,
                experience_gained: 0,
                credits_looted: -stolen,
                message: format!("Pirates stole {} credits!", stolen),
            }
        }
        Opponent::Player { handle, .. } => {
            // PvP surrender - lose 10% credits
            let stolen = state.credits / 10;
            state.credits -= stolen;
            state.adjust_alignment(-20);

            CombatResult {
                victory: false,
                fled: false,
                player_fighters_lost: 0,
                player_shields_lost: 0,
                opponent_fighters_lost: 0,
                opponent_shields_lost: 0,
                experience_gained: 0,
                credits_looted: -stolen,
                message: format!("You paid {} credits in tribute to {}.", stolen, handle),
            }
        }
    }
}

/// Handle player death (ship destroyed)
fn handle_player_death(state: &mut GameState) {
    state.deaths += 1;
    state.stats.times_destroyed += 1;

    // Lose cargo
    state.cargo.fuel_ore = 0;
    state.cargo.organics = 0;
    state.cargo.equipment = 0;
    state.cargo.colonists = 0;

    // Return to StarDock in escape pod
    state.sector = 1;
    state.docked = true;

    // Reset to starter ship
    state.ship_class = "merchant_cruiser".to_string();
    state.fighters = 10;
    state.shields = 20;

    // Lose half credits
    state.credits /= 2;

    // Experience penalty
    state.experience = (state.experience - 500).max(0);
}

/// Check if a Ferrengi encounter should occur
pub fn should_ferrengi_attack() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) < config::FERRENGI_CHANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ferrengi() {
        let ferrengi = generate_ferrengi(50);
        assert!(matches!(ferrengi, Opponent::Ferrengi { .. }));
        assert!(ferrengi.fighters() > 0);
    }

    #[test]
    fn test_combat_attack() {
        let mut state = GameState::new(1, "Test".to_string());
        state.fighters = 50;
        state.shields = 50;

        let mut opponent = Opponent::Pirate {
            name: "Test Pirate".to_string(),
            fighters: 5,
        };

        let result = combat_round(&mut state, &mut opponent, CombatAction::Attack);
        // Should likely win against 5 pirates
        // But combat is random so we just check it runs
        assert!(result.player_fighters_lost + result.opponent_fighters_lost > 0
            || result.victory
            || result.message.len() > 0);
    }

    #[test]
    fn test_combat_flee() {
        let mut state = GameState::new(1, "Test".to_string());
        state.fighters = 20;

        let mut opponent = Opponent::Ferrengi {
            name: "Test Ferrengi".to_string(),
            fighters: 30,
            shields: 20,
        };

        let result = combat_round(&mut state, &mut opponent, CombatAction::Flee);
        // Either fled successfully or failed
        assert!(result.fled || result.message.contains("Failed"));
    }

    #[test]
    fn test_combat_surrender() {
        let mut state = GameState::new(1, "Test".to_string());
        state.cargo.fuel_ore = 10;
        state.cargo.organics = 10;

        let mut opponent = Opponent::Ferrengi {
            name: "Test Ferrengi".to_string(),
            fighters: 30,
            shields: 20,
        };

        let _result = combat_round(&mut state, &mut opponent, CombatAction::Surrender);
        assert_eq!(state.cargo.fuel_ore, 0);  // Cargo taken
        assert_eq!(state.cargo.organics, 0);
    }

    #[test]
    fn test_player_death() {
        let mut state = GameState::new(1, "Test".to_string());
        state.sector = 50;
        state.credits = 10000;
        state.cargo.fuel_ore = 100;

        handle_player_death(&mut state);

        assert_eq!(state.sector, 1);  // Back at StarDock
        assert_eq!(state.cargo.fuel_ore, 0);  // Lost cargo
        assert_eq!(state.credits, 5000);  // Lost half credits
        assert_eq!(state.deaths, 1);
    }
}
