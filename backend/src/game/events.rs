use rand::prelude::*;
use rand::distributions::WeightedIndex;
use std::collections::HashMap;

use crate::game::{
    GameState, GameScreen, EnemyType, GameEvent,
    COMMODITIES, get_borough,
};

/// Outcome of resolving an event
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EventOutcome {
    /// Event continues (e.g., combat not resolved yet)
    Continue,
    /// Event resolved, return to main menu
    Resolved { message: Option<String> },
    /// Player died
    Death { reason: String },
    /// State was modified, save needed
    SaveNeeded,
}

/// Maybe trigger a random event (call after travel)
/// Returns Some(screen) if event triggered, None otherwise
pub fn maybe_trigger_event(state: &GameState) -> Option<GameScreen> {
    let mut rng = thread_rng();

    // 15% base chance for any event
    if rng.gen_range(0..100) >= 15 {
        return None;
    }

    // Build weighted event list based on game state
    let mut events: Vec<(GameScreen, u32)> = Vec::new();

    // === POSITIVE EVENTS ===

    // Find cash (common)
    events.push((
        GameScreen::Event {
            event: GameEvent::FindCash {
                amount: rng.gen_range(1000..10000), // $10-$100
            },
        },
        20,
    ));

    // Find drugs (less common)
    let random_commodity = &COMMODITIES[rng.gen_range(0..COMMODITIES.len())];
    events.push((
        GameScreen::Event {
            event: GameEvent::FindDrugs {
                commodity: random_commodity.key.to_string(),
                amount: rng.gen_range(1..5),
            },
        },
        10,
    ));

    // Price drop (good for buying)
    let price_commodity = &COMMODITIES[rng.gen_range(0..COMMODITIES.len())];
    events.push((
        GameScreen::Event {
            event: GameEvent::PriceDrop {
                commodity: price_commodity.key.to_string(),
                location: state.location.clone(),
            },
        },
        15,
    ));

    // Price spike (good for selling)
    events.push((
        GameScreen::Event {
            event: GameEvent::PriceSpike {
                commodity: price_commodity.key.to_string(),
                location: state.location.clone(),
            },
        },
        10,
    ));

    // === NEGATIVE EVENTS ===

    // Police encounter (base rate, increases with notoriety)
    let police_weight = 15 + (state.notoriety / 5);
    events.push((
        GameScreen::Combat {
            enemy_type: EnemyType::Police,
            enemy_hp: 50 + rng.gen_range(0..20),
        },
        police_weight,
    ));

    // Mugger (common)
    events.push((
        GameScreen::Combat {
            enemy_type: EnemyType::Mugger,
            enemy_hp: 30 + rng.gen_range(0..20),
        },
        20,
    ));

    // Loan shark enforcer (only if significant debt, weight increases with debt)
    if state.debt > 1000000 {
        // Over $10,000 debt
        let enforcer_weight = 5 + ((state.debt - 1000000) / 500000) as u32;
        events.push((
            GameScreen::Combat {
                enemy_type: EnemyType::LoanSharkEnforcer,
                enemy_hp: 80 + rng.gen_range(0..30),
            },
            enforcer_weight.min(25), // Cap at 25
        ));
    }

    // Gang encounter (only in gang territory with negative relations)
    if let Some(borough) = get_borough(&state.city, &state.location) {
        if let Some(gang_key) = borough.gang_territory {
            let relation = state.gang_relations.get(gang_key).copied().unwrap_or(0);
            if relation < 0 {
                let gang_weight = 10 + ((-relation) / 5) as u32;
                events.push((
                    GameScreen::Combat {
                        enemy_type: EnemyType::Gang {
                            gang_key: gang_key.to_string(),
                        },
                        enemy_hp: 60 + rng.gen_range(0..20),
                    },
                    gang_weight.min(30),
                ));
            }
        }
    }

    // === SPECIAL EVENTS ===

    // Trenchcoat guy (only if not max tier)
    if state.coat_tier < 3 {
        events.push((
            GameScreen::Event {
                event: GameEvent::TrenchcoatGuy,
            },
            5,
        ));
    }

    // Weighted selection
    let weights: Vec<u32> = events.iter().map(|(_, w)| *w).collect();
    if let Ok(dist) = WeightedIndex::new(&weights) {
        let idx = dist.sample(&mut rng);
        return Some(events.remove(idx).0);
    }

    None
}

/// Apply price event effects to market prices
pub fn apply_price_event(
    event: &GameEvent,
    prices: &mut HashMap<String, i64>,
) {
    match event {
        GameEvent::PriceDrop { commodity, .. } => {
            if let Some(price) = prices.get_mut(commodity) {
                // Drop price by 50-80%
                let mut rng = thread_rng();
                let multiplier = rng.gen_range(0.2..0.5);
                *price = ((*price as f64) * multiplier) as i64;
            }
        }
        GameEvent::PriceSpike { commodity, .. } => {
            if let Some(price) = prices.get_mut(commodity) {
                // Spike price by 200-400%
                let mut rng = thread_rng();
                let multiplier = rng.gen_range(2.0..4.0);
                *price = ((*price as f64) * multiplier) as i64;
            }
        }
        _ => {}
    }
}

/// Apply find cash/drugs event
pub fn apply_find_event(event: &GameEvent, state: &mut GameState) {
    match event {
        GameEvent::FindCash { amount } => {
            state.cash += amount;
        }
        GameEvent::FindDrugs { commodity, amount } => {
            // Only pick up if we have space
            let capacity_left = state.coat_capacity() - state.inventory_count();
            let actual_amount = (*amount).min(capacity_left);
            if actual_amount > 0 {
                // Found drugs have 0 cost basis (100% profit when sold)
                state.add_inventory(commodity, actual_amount, 0);
            }
        }
        _ => {}
    }
}

/// Resolve combat outcome
#[allow(dead_code)]
pub struct CombatResult {
    pub player_won: bool,
    pub player_damage: u32,
    pub enemy_killed: bool,
    pub loot_cash: i64,
    pub notoriety_change: i32,
    pub message: String,
}

/// Calculate combat resolution
pub fn resolve_combat(
    state: &GameState,
    enemy_type: &EnemyType,
    enemy_hp: u32,
    action: CombatAction,
) -> CombatResult {
    let mut rng = thread_rng();

    // Get player weapon damage
    let base_damage = if let Some(ref gun) = state.weapons.gun {
        crate::game::get_weapon(gun).map(|w| w.damage).unwrap_or(3)
    } else if let Some(ref melee) = state.weapons.melee {
        crate::game::get_weapon(melee).map(|w| w.damage).unwrap_or(3)
    } else {
        3 // Bare fists
    };

    // Enemy stats based on type
    let (enemy_damage, enemy_name, base_loot) = match enemy_type {
        EnemyType::Police => (15, "Police Officer", 0),
        EnemyType::Mugger => (10, "Mugger", rng.gen_range(500..2000)),
        EnemyType::Gang { gang_key } => {
            let name = match gang_key.as_str() {
                "triads" => "Triad Enforcer",
                "cartel" => "Cartel Soldier",
                "mafia" => "Mafia Goon",
                _ => "Gang Member",
            };
            (20, name, rng.gen_range(1000..5000))
        }
        EnemyType::LoanSharkEnforcer => (25, "Loan Shark Enforcer", 0),
    };

    match action {
        CombatAction::Fight => {
            // Combat resolution - player attacks first
            let player_roll = base_damage + rng.gen_range(0..10);
            let enemy_roll = enemy_damage + rng.gen_range(0..10);

            // Simple win condition: player damage > enemy HP, or player roll > enemy roll
            let player_won = player_roll > enemy_roll || player_roll > enemy_hp;
            let player_damage = if player_won {
                enemy_roll / 2 // Take some damage even when winning
            } else {
                enemy_roll
            };

            let notoriety_change = if player_won {
                match enemy_type {
                    EnemyType::Police => 15,  // Killing cops is very bad
                    EnemyType::Gang { .. } => 5,
                    _ => 2,
                }
            } else {
                -2 // Lost respect
            };

            let message = if player_won {
                format!("You defeated the {}! Took {} damage.", enemy_name, player_damage)
            } else {
                format!("The {} beat you! You lost {} HP.", enemy_name, player_damage)
            };

            CombatResult {
                player_won,
                player_damage,
                enemy_killed: player_won,
                loot_cash: if player_won { base_loot } else { 0 },
                notoriety_change,
                message,
            }
        }
        CombatAction::Run => {
            // 60% chance to escape, take damage if caught
            let escaped = rng.gen_range(0..100) < 60;
            let player_damage = if escaped { 0 } else { enemy_damage / 2 };

            let message = if escaped {
                format!("You escaped from the {}!", enemy_name)
            } else {
                format!("The {} caught you while fleeing! Took {} damage.", enemy_name, player_damage)
            };

            CombatResult {
                player_won: escaped,
                player_damage,
                enemy_killed: false,
                loot_cash: 0,
                notoriety_change: if escaped { -1 } else { -3 },
                message,
            }
        }
        CombatAction::Talk => {
            // Only works on police, 30% base chance + charisma (none for now)
            if !matches!(enemy_type, EnemyType::Police) {
                return CombatResult {
                    player_won: false,
                    player_damage: enemy_damage,
                    enemy_killed: false,
                    loot_cash: 0,
                    notoriety_change: 0,
                    message: "They're not interested in talking!".to_string(),
                };
            }

            let success = rng.gen_range(0..100) < 30;
            if success {
                CombatResult {
                    player_won: true,
                    player_damage: 0,
                    enemy_killed: false,
                    loot_cash: 0,
                    notoriety_change: -5, // Smooth talking reduces heat
                    message: "You talked your way out of it! The officer lets you go.".to_string(),
                }
            } else {
                CombatResult {
                    player_won: false,
                    player_damage: enemy_damage / 2,
                    enemy_killed: false,
                    loot_cash: 0,
                    notoriety_change: 5,
                    message: "They're not buying it! The officer gets aggressive.".to_string(),
                }
            }
        }
        CombatAction::Bribe { amount } => {
            // Only works on police, success rate based on amount
            if !matches!(enemy_type, EnemyType::Police) {
                return CombatResult {
                    player_won: false,
                    player_damage: enemy_damage,
                    enemy_killed: false,
                    loot_cash: 0,
                    notoriety_change: 0,
                    message: "They take your money AND attack you!".to_string(),
                };
            }

            // $100+ = 50%, $500+ = 80%, $1000+ = 95%
            let success_rate = if amount >= 100000 {
                95
            } else if amount >= 50000 {
                80
            } else if amount >= 10000 {
                50
            } else {
                20
            };

            let success = rng.gen_range(0..100) < success_rate;
            if success {
                CombatResult {
                    player_won: true,
                    player_damage: 0,
                    enemy_killed: false,
                    loot_cash: -amount, // Negative = spent money
                    notoriety_change: -3,
                    message: format!("The officer pockets {} and walks away.", super::render::format_money(amount)),
                }
            } else {
                CombatResult {
                    player_won: false,
                    player_damage: enemy_damage,
                    enemy_killed: false,
                    loot_cash: -amount / 2, // They take half and attack
                    notoriety_change: 10,
                    message: "The officer is insulted by your small bribe!".to_string(),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CombatAction {
    Fight,
    Run,
    Talk,
    Bribe { amount: i64 },
}

/// Apply combat result to game state
pub fn apply_combat_result(state: &mut GameState, result: &CombatResult) {
    // Apply damage
    state.health = state.health.saturating_sub(result.player_damage);

    // Apply notoriety change
    state.notoriety = (state.notoriety as i32 + result.notoriety_change)
        .max(0)
        .min(100) as u32;

    // Apply loot (or bribe cost if negative)
    if result.loot_cash >= 0 {
        state.cash += result.loot_cash;
    } else {
        state.cash = (state.cash + result.loot_cash).max(0);
    }

    // Update stats
    if matches!(result.enemy_killed, true) {
        state.stats.people_killed += 1;
    }
    state.stats.police_encounters += 1; // Increment for any combat
    state.stats.muggings_survived += 1;

    // Check for death
    if state.health == 0 {
        state.game_over = true;
        state.game_over_reason = Some("You died from your injuries.".to_string());
    }
}

/// Handle trenchcoat guy encounter
pub fn handle_trenchcoat_upgrade(state: &mut GameState, accept: bool) -> String {
    if !accept {
        return "You decline the offer and walk away.".to_string();
    }

    if state.coat_tier >= 3 {
        return "You already have the best coat!".to_string();
    }

    // Must dump all inventory
    let had_inventory = !state.inventory_lots.is_empty();
    state.inventory_lots.clear();
    state.coat_tier += 1;

    let new_capacity = state.coat_capacity();
    if had_inventory {
        format!(
            "You dump everything and take the new coat. Capacity: {} units.",
            new_capacity
        )
    } else {
        format!(
            "The trenchcoat guy hooks you up! New capacity: {} units.",
            new_capacity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_result_application() {
        let mut state = GameState::new();
        state.health = 100;

        let result = CombatResult {
            player_won: true,
            player_damage: 20,
            enemy_killed: true,
            loot_cash: 5000,
            notoriety_change: 5,
            message: "Victory!".to_string(),
        };

        apply_combat_result(&mut state, &result);

        assert_eq!(state.health, 80);
        assert_eq!(state.cash, 205000); // 200000 + 5000
        assert_eq!(state.notoriety, 5);
        assert_eq!(state.stats.people_killed, 1);
    }

    #[test]
    fn test_find_cash_event() {
        let mut state = GameState::new();
        let event = GameEvent::FindCash { amount: 10000 };
        apply_find_event(&event, &mut state);
        assert_eq!(state.cash, 210000);
    }

    #[test]
    fn test_find_drugs_event() {
        let mut state = GameState::new();
        let event = GameEvent::FindDrugs {
            commodity: "weed".to_string(),
            amount: 5,
        };
        apply_find_event(&event, &mut state);
        assert_eq!(state.get_quantity("weed"), 5);
    }

    #[test]
    fn test_trenchcoat_upgrade() {
        let mut state = GameState::new();
        state.coat_tier = 0;
        let msg = handle_trenchcoat_upgrade(&mut state, true);
        assert_eq!(state.coat_tier, 1);
        assert!(msg.contains("125"));
    }

    #[test]
    fn test_combat_run_action() {
        let state = GameState::new();
        let result = resolve_combat(
            &state,
            &EnemyType::Mugger,
            30,
            CombatAction::Run,
        );

        // Either escaped (won=true, damage=0) or caught (won=false, damage>0)
        if result.player_won {
            assert_eq!(result.player_damage, 0);
        } else {
            assert!(result.player_damage > 0);
        }
        assert!(!result.enemy_killed);
    }
}
