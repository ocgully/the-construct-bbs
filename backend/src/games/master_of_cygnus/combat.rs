//! Combat resolution for Master of Andromeda

use serde::{Deserialize, Serialize};
use rand::Rng;
use super::ships::{Fleet, ShipDesign};

/// Result of a space battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleResult {
    pub attacker_empire_id: u32,
    pub defender_empire_id: u32,
    pub location_star_id: u32,
    pub turn_number: u32,
    /// Winner's empire ID (None if draw)
    pub winner: Option<u32>,
    /// Ships lost by attacker (design_id -> count)
    pub attacker_losses: Vec<(u32, u32)>,
    /// Ships lost by defender (design_id -> count)
    pub defender_losses: Vec<(u32, u32)>,
    /// Narrative description of the battle
    pub description: String,
}

/// Combat statistics for a side in battle
#[derive(Debug, Clone)]
struct CombatStats {
    empire_id: u32,
    ships: Vec<(u32, u32)>, // (design_id, count)
    total_hp: u32,
    total_attack: u32,
    total_defense: u32,
}

/// Resolve combat between two fleets
pub fn resolve_battle(
    attacker: &Fleet,
    defender: &Fleet,
    attacker_designs: &[ShipDesign],
    defender_designs: &[ShipDesign],
    star_id: u32,
    turn_number: u32,
) -> BattleResult {
    let mut rng = rand::thread_rng();

    // Calculate initial stats
    let mut attacker_stats = calculate_combat_stats(attacker, attacker_designs);
    let mut defender_stats = calculate_combat_stats(defender, defender_designs);

    let initial_attacker_ships: u32 = attacker_stats.ships.iter().map(|(_, c)| c).sum();
    let initial_defender_ships: u32 = defender_stats.ships.iter().map(|(_, c)| c).sum();

    let mut battle_log = Vec::new();
    battle_log.push(format!(
        "Battle at Star {} - {} ships vs {} ships",
        star_id, initial_attacker_ships, initial_defender_ships
    ));

    // Simulate battle rounds (max 10)
    for round in 1..=10 {
        if attacker_stats.total_hp == 0 || defender_stats.total_hp == 0 {
            break;
        }

        // Both sides fire
        let attacker_damage = calculate_damage(&attacker_stats, &defender_stats, &mut rng);
        let defender_damage = calculate_damage(&defender_stats, &attacker_stats, &mut rng);

        // Apply damage
        apply_damage(&mut defender_stats, attacker_damage, defender_designs);
        apply_damage(&mut attacker_stats, defender_damage, attacker_designs);

        battle_log.push(format!(
            "Round {}: Attacker deals {} damage, Defender deals {} damage",
            round, attacker_damage, defender_damage
        ));
    }

    // Determine winner
    let winner = if attacker_stats.total_hp > 0 && defender_stats.total_hp == 0 {
        Some(attacker.empire_id)
    } else if defender_stats.total_hp > 0 && attacker_stats.total_hp == 0 {
        Some(defender.empire_id)
    } else if attacker_stats.total_hp > defender_stats.total_hp {
        Some(attacker.empire_id)
    } else if defender_stats.total_hp > attacker_stats.total_hp {
        Some(defender.empire_id)
    } else {
        None // Draw
    };

    // Calculate losses
    let attacker_losses = calculate_losses(attacker, &attacker_stats);
    let defender_losses = calculate_losses(defender, &defender_stats);

    let outcome = match winner {
        Some(id) if id == attacker.empire_id => "Attacker victorious!",
        Some(_) => "Defender victorious!",
        None => "Battle ends in stalemate.",
    };
    battle_log.push(outcome.to_string());

    BattleResult {
        attacker_empire_id: attacker.empire_id,
        defender_empire_id: defender.empire_id,
        location_star_id: star_id,
        turn_number,
        winner,
        attacker_losses,
        defender_losses,
        description: battle_log.join("\n"),
    }
}

/// Calculate combat statistics for a fleet
fn calculate_combat_stats(fleet: &Fleet, designs: &[ShipDesign]) -> CombatStats {
    let mut total_hp = 0;
    let mut total_attack = 0;
    let mut total_defense = 0;
    let mut ships = Vec::new();

    for (design_id, count) in &fleet.ships {
        if let Some(design) = designs.iter().find(|d| d.id == *design_id) {
            total_hp += design.total_hp * count;
            total_attack += design.attack_power * count;
            total_defense += design.defense * count;
            ships.push((*design_id, *count));
        }
    }

    CombatStats {
        empire_id: fleet.empire_id,
        ships,
        total_hp,
        total_attack,
        total_defense,
    }
}

/// Calculate damage dealt this round
fn calculate_damage<R: Rng>(attacker: &CombatStats, defender: &CombatStats, rng: &mut R) -> u32 {
    if attacker.total_attack == 0 {
        return 0;
    }

    // Base damage is attack power
    let base_damage = attacker.total_attack;

    // Defense reduces damage
    let damage_reduction = (defender.total_defense as f64 / 2.0).min(base_damage as f64 * 0.75);
    let effective_damage = (base_damage as f64 - damage_reduction).max(1.0);

    // Add randomness (80% to 120%)
    let variance = rng.gen_range(80..=120) as f64 / 100.0;
    let final_damage = (effective_damage * variance) as u32;

    final_damage.max(1)
}

/// Apply damage to a side, removing destroyed ships
fn apply_damage(stats: &mut CombatStats, damage: u32, designs: &[ShipDesign]) {
    if damage >= stats.total_hp {
        stats.total_hp = 0;
        stats.ships.clear();
        return;
    }

    stats.total_hp = stats.total_hp.saturating_sub(damage);

    // Proportionally reduce ships based on damage
    let damage_ratio = damage as f64 / stats.total_hp.max(1) as f64;

    for (design_id, count) in &mut stats.ships {
        if *count == 0 {
            continue;
        }

        if let Some(design) = designs.iter().find(|d| d.id == *design_id) {
            // Smaller ships more likely to be destroyed
            let vulnerability = 1.0 + (1.0 / design.total_hp as f64);
            let losses = ((*count as f64) * damage_ratio * vulnerability).ceil() as u32;
            *count = count.saturating_sub(losses.min(*count));

            // Update totals
            stats.total_attack = stats.total_attack.saturating_sub(design.attack_power * losses);
            stats.total_defense = stats.total_defense.saturating_sub(design.defense * losses);
        }
    }

    // Remove designs with 0 ships
    stats.ships.retain(|(_, count)| *count > 0);
}

/// Calculate ships lost in battle
fn calculate_losses(original: &Fleet, final_stats: &CombatStats) -> Vec<(u32, u32)> {
    let mut losses = Vec::new();

    for (design_id, original_count) in &original.ships {
        let final_count = final_stats.ships.iter()
            .find(|(id, _)| id == design_id)
            .map(|(_, c)| *c)
            .unwrap_or(0);

        if *original_count > final_count {
            losses.push((*design_id, original_count - final_count));
        }
    }

    losses
}

/// Simulate orbital bombardment of a colony
pub fn bombard_colony(
    attacker: &Fleet,
    designs: &[ShipDesign],
    colony_population: u32,
    has_planetary_shield: bool,
) -> (u32, String) {
    let mut rng = rand::thread_rng();

    // Calculate bombardment power
    let mut bomb_power: u32 = 0;
    for (design_id, count) in &attacker.ships {
        if let Some(design) = designs.iter().find(|d| d.id == *design_id) {
            // Only attack-capable ships can bombard
            if design.attack_power > 0 {
                bomb_power += (design.attack_power / 2) * count;
            }
        }
    }

    if bomb_power == 0 {
        return (0, "No bombardment capability in fleet.".to_string());
    }

    // Planetary shield reduces bombardment significantly
    if has_planetary_shield {
        bomb_power = bomb_power / 4;
    }

    // Calculate casualties (random factor)
    let variance = rng.gen_range(50..=150) as u32;
    let casualties = ((bomb_power * variance) / 100).min(colony_population);

    let description = if has_planetary_shield {
        format!(
            "Orbital bombardment partially blocked by planetary shield. {} population killed.",
            casualties
        )
    } else {
        format!(
            "Orbital bombardment devastates the colony. {} population killed.",
            casualties
        )
    };

    (casualties, description)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_fleet(empire_id: u32, design_id: u32, count: u32) -> Fleet {
        let mut fleet = Fleet::new(0, empire_id, "Test Fleet".to_string(), 0);
        fleet.add_ships(design_id, count);
        fleet
    }

    fn create_test_design(id: u32, attack: u32, defense: u32, hp: u32) -> ShipDesign {
        ShipDesign {
            id,
            name: format!("Design {}", id),
            hull: super::super::ships::HullSize::Destroyer,
            components: vec![],
            total_hp: hp,
            attack_power: attack,
            defense,
            speed: 3,
            range: 5,
            cost: 50,
            is_colony_ship: false,
        }
    }

    #[test]
    fn test_battle_resolution() {
        let attacker = create_test_fleet(0, 0, 10);
        let defender = create_test_fleet(1, 0, 5);

        let designs = vec![create_test_design(0, 10, 5, 25)];

        let result = resolve_battle(&attacker, &defender, &designs, &designs, 0, 1);

        // Attacker should win with numerical advantage
        assert_eq!(result.winner, Some(0));
        assert!(!result.description.is_empty());
    }

    #[test]
    fn test_evenly_matched_battle() {
        let attacker = create_test_fleet(0, 0, 5);
        let defender = create_test_fleet(1, 0, 5);

        let designs = vec![create_test_design(0, 10, 5, 25)];

        let result = resolve_battle(&attacker, &defender, &designs, &designs, 0, 1);

        // Should have casualties on both sides
        assert!(!result.attacker_losses.is_empty() || !result.defender_losses.is_empty());
    }

    #[test]
    fn test_bombardment() {
        let attacker = create_test_fleet(0, 0, 10);
        let designs = vec![create_test_design(0, 20, 5, 25)];

        let (casualties, desc) = bombard_colony(&attacker, &designs, 100, false);
        assert!(casualties > 0);
        assert!(desc.contains("bombardment"));

        // With shield
        let (shielded_casualties, _) = bombard_colony(&attacker, &designs, 100, true);
        assert!(shielded_casualties < casualties);
    }

    #[test]
    fn test_no_bombardment_capability() {
        let attacker = create_test_fleet(0, 0, 10);
        let designs = vec![create_test_design(0, 0, 5, 25)]; // No attack power

        let (casualties, desc) = bombard_colony(&attacker, &designs, 100, false);
        assert_eq!(casualties, 0);
        assert!(desc.contains("No bombardment"));
    }
}
