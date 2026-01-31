//! Enemy AI System for Realm of Ralnar Combat
//!
//! Different AI types control how enemies behave in battle.
//! The Guardian AI is particularly important for foreshadowing the twist.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::combat::{BattleAction, BattleEnemy, BattleState};
use super::magic::get_spell;
// Re-export EnemyAIType from data module for use in combat
pub use super::data::enemies::EnemyAIType;

/// Combat-specific enemy stats (extends base EnemyStats from data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEnemyStats {
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub magic_defense: i32,
    pub agility: i32,
    pub level: i32,
    /// Spells this enemy can use
    pub spells: Vec<String>,
    /// Is this enemy immune to instant death?
    pub death_immune: bool,
    /// Is this enemy immune to status effects?
    pub status_immune: bool,
}

impl Default for CombatEnemyStats {
    fn default() -> Self {
        Self {
            attack: 10,
            defense: 5,
            magic: 5,
            magic_defense: 5,
            agility: 10,
            level: 1,
            spells: Vec::new(),
            death_immune: false,
            status_immune: false,
        }
    }
}

impl From<super::data::enemies::EnemyStats> for CombatEnemyStats {
    fn from(stats: super::data::enemies::EnemyStats) -> Self {
        Self {
            attack: stats.attack,
            defense: stats.defense,
            magic: stats.magic,
            magic_defense: stats.magic_def,
            agility: stats.agility,
            level: 1, // Default level, can be overridden
            spells: Vec::new(),
            death_immune: false,
            status_immune: false,
        }
    }
}

impl BattleEnemy {
    /// Choose an action based on AI type and battle state
    pub fn choose_action(&self, battle: &BattleState) -> BattleAction {
        match self.ai_type {
            EnemyAIType::Normal => self.normal_ai(battle),
            EnemyAIType::Aggressive => self.aggressive_ai(battle),
            EnemyAIType::Defensive => self.defensive_ai(battle),
            EnemyAIType::Guardian => self.guardian_ai(battle),
            EnemyAIType::Boss => self.boss_ai(battle),
            EnemyAIType::Healer => self.healer_ai(battle),
            EnemyAIType::Mage | EnemyAIType::Magical => self.mage_ai(battle),
            EnemyAIType::Berserker => self.berserker_ai(battle),
            EnemyAIType::Debuffer => self.debuffer_ai(battle),
            EnemyAIType::Summoner => self.normal_ai(battle), // Fallback to normal for now
        }
    }

    /// Normal AI: Attack strongest or random target
    fn normal_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();

        // Find living party members
        let living_targets: Vec<usize> = battle
            .party
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_alive())
            .map(|(i, _)| i)
            .collect();

        if living_targets.is_empty() {
            return BattleAction::Defend;
        }

        // 70% chance to attack strongest, 30% random
        let target = if rng.gen::<f32>() < 0.70 {
            // Find character with highest current HP
            *living_targets
                .iter()
                .max_by_key(|&&i| battle.party[i].hp)
                .unwrap_or(&0)
        } else {
            living_targets[rng.gen_range(0..living_targets.len())]
        };

        BattleAction::Attack { target }
    }

    /// Aggressive AI: Always attack, prefer wounded targets
    fn aggressive_ai(&self, battle: &BattleState) -> BattleAction {
        let living_targets: Vec<usize> = battle
            .party
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_alive())
            .map(|(i, _)| i)
            .collect();

        if living_targets.is_empty() {
            return BattleAction::Defend;
        }

        // Target character with lowest HP (finish them off)
        let target = *living_targets
            .iter()
            .min_by_key(|&&i| battle.party[i].hp)
            .unwrap_or(&0);

        BattleAction::Attack { target }
    }

    /// Defensive AI: Use buffs, heal when low
    fn defensive_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();
        let hp_percent = self.hp as f32 / self.hp_max as f32;

        // Heal if below 50% HP and has a heal spell
        if hp_percent < 0.50 {
            if let Some(heal_spell) = self.find_best_heal_spell() {
                return BattleAction::Magic {
                    spell_id: heal_spell.to_string(),
                    targets: vec![0], // Target self (enemy index would be handled differently)
                };
            }
        }

        // 40% defend, 60% attack
        if rng.gen::<f32>() < 0.40 {
            BattleAction::Defend
        } else {
            self.normal_ai(battle)
        }
    }

    /// Guardian AI - The key foreshadowing mechanic
    ///
    /// Guardians fight DEFENSIVELY and reluctantly. They are protecting
    /// something, not trying to kill the party. Their behavior hints
    /// at the twist that they're actually trying to protect humanity.
    fn guardian_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();
        let hp_percent = self.hp as f32 / self.hp_max as f32;

        // Guardians behave differently based on HP
        match hp_percent {
            p if p > 0.75 => {
                // Warning phase - mostly defensive, trying to discourage fight
                // 60% shield/buff, 30% weak attack, 10% just wait
                let roll: f32 = rng.gen();
                if roll < 0.60 {
                    // Try to use Protect/Shell on self
                    if let Some(buff) = self.find_buff_spell() {
                        return BattleAction::Magic {
                            spell_id: buff.to_string(),
                            targets: vec![],
                        };
                    }
                    BattleAction::Defend
                } else if roll < 0.90 {
                    // Light attack on a random target (not trying to kill)
                    let living: Vec<usize> = battle
                        .party
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_alive())
                        .map(|(i, _)| i)
                        .collect();

                    if living.is_empty() {
                        BattleAction::Defend
                    } else {
                        // Attack random target, not targeting weak enemies
                        let target = living[rng.gen_range(0..living.len())];
                        BattleAction::Attack { target }
                    }
                } else {
                    // Just wait/defend
                    BattleAction::Defend
                }
            }
            p if p > 0.50 => {
                // Reluctant phase - starting to fight back, but still hesitant
                // 40% defend, 30% weak attack, 30% heal
                let roll: f32 = rng.gen();
                if roll < 0.40 {
                    BattleAction::Defend
                } else if roll < 0.70 {
                    // Random attack
                    let living: Vec<usize> = battle
                        .party
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_alive())
                        .map(|(i, _)| i)
                        .collect();

                    if living.is_empty() {
                        BattleAction::Defend
                    } else {
                        let target = living[rng.gen_range(0..living.len())];
                        BattleAction::Attack { target }
                    }
                } else {
                    // Try to heal
                    if let Some(heal) = self.find_best_heal_spell() {
                        BattleAction::Magic {
                            spell_id: heal.to_string(),
                            targets: vec![],
                        }
                    } else {
                        BattleAction::Defend
                    }
                }
            }
            p if p > 0.25 => {
                // Desperate phase - fighting back more seriously
                // 30% heal (priority), 50% attack, 20% defend
                let roll: f32 = rng.gen();
                if roll < 0.30 {
                    if let Some(heal) = self.find_best_heal_spell() {
                        return BattleAction::Magic {
                            spell_id: heal.to_string(),
                            targets: vec![],
                        };
                    }
                }

                if roll < 0.80 {
                    // Now targeting properly
                    self.normal_ai(battle)
                } else {
                    BattleAction::Defend
                }
            }
            _ => {
                // Last stand - full power (survival mode)
                // Priority: heal if possible, otherwise attack strongest
                if let Some(heal) = self.find_best_heal_spell() {
                    if rng.gen::<f32>() < 0.50 {
                        return BattleAction::Magic {
                            spell_id: heal.to_string(),
                            targets: vec![],
                        };
                    }
                }

                // Attack strongest target (now actually fighting)
                let living: Vec<usize> = battle
                    .party
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_alive())
                    .map(|(i, _)| i)
                    .collect();

                if living.is_empty() {
                    return BattleAction::Defend;
                }

                // Target highest threat (most HP = most dangerous)
                let target = *living
                    .iter()
                    .max_by_key(|&&i| battle.party[i].hp)
                    .unwrap_or(&0);

                BattleAction::Attack { target }
            }
        }
    }

    /// Boss AI: Scripted patterns based on HP thresholds
    fn boss_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();
        let hp_percent = self.hp as f32 / self.hp_max as f32;

        // Phase-based behavior
        if hp_percent > 0.75 {
            // Phase 1: Standard attacks with occasional spells
            if rng.gen::<f32>() < 0.30 && !self.combat_stats.spells.is_empty() {
                if let Some(spell) = self.find_best_damage_spell(battle) {
                    return BattleAction::Magic {
                        spell_id: spell.to_string(),
                        targets: self.find_spell_targets(battle, &spell),
                    };
                }
            }
            self.aggressive_ai(battle)
        } else if hp_percent > 0.50 {
            // Phase 2: More aggressive, more spells
            if rng.gen::<f32>() < 0.50 && !self.combat_stats.spells.is_empty() {
                if let Some(spell) = self.find_best_damage_spell(battle) {
                    return BattleAction::Magic {
                        spell_id: spell.to_string(),
                        targets: self.find_spell_targets(battle, &spell),
                    };
                }
            }
            self.aggressive_ai(battle)
        } else if hp_percent > 0.25 {
            // Phase 3: Desperation - might heal or go all out
            if rng.gen::<f32>() < 0.25 {
                if let Some(heal) = self.find_best_heal_spell() {
                    return BattleAction::Magic {
                        spell_id: heal.to_string(),
                        targets: vec![],
                    };
                }
            }
            // Use strongest abilities
            if let Some(spell) = self.find_strongest_spell() {
                return BattleAction::Magic {
                    spell_id: spell.to_string(),
                    targets: self.find_spell_targets(battle, &spell),
                };
            }
            self.aggressive_ai(battle)
        } else {
            // Phase 4: Last stand - most powerful attacks
            if let Some(spell) = self.find_strongest_spell() {
                return BattleAction::Magic {
                    spell_id: spell.to_string(),
                    targets: self.find_spell_targets(battle, &spell),
                };
            }
            self.aggressive_ai(battle)
        }
    }

    /// Healer AI: Prioritizes healing allies
    fn healer_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();

        // Find wounded allies
        let wounded_allies: Vec<usize> = battle
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.hp > 0 && e.hp < e.hp_max / 2)
            .map(|(i, _)| i)
            .collect();

        // Heal if there are wounded allies
        if !wounded_allies.is_empty() {
            if let Some(heal) = self.find_best_heal_spell() {
                let target = wounded_allies[rng.gen_range(0..wounded_allies.len())];
                return BattleAction::Magic {
                    spell_id: heal.to_string(),
                    targets: vec![target],
                };
            }
        }

        // Otherwise, 50% buff, 50% attack
        if rng.gen::<f32>() < 0.50 {
            if let Some(buff) = self.find_buff_spell() {
                return BattleAction::Magic {
                    spell_id: buff.to_string(),
                    targets: vec![],
                };
            }
        }

        self.normal_ai(battle)
    }

    /// Mage AI: Prioritizes magic attacks
    fn mage_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();

        // 80% chance to use magic
        if rng.gen::<f32>() < 0.80 && !self.combat_stats.spells.is_empty() {
            if let Some(spell) = self.find_best_damage_spell(battle) {
                return BattleAction::Magic {
                    spell_id: spell.to_string(),
                    targets: self.find_spell_targets(battle, &spell),
                };
            }
        }

        // Fallback to physical attack
        self.normal_ai(battle)
    }

    /// Berserker AI: Attack relentlessly
    fn berserker_ai(&self, battle: &BattleState) -> BattleAction {
        // Always attack, never defend
        self.aggressive_ai(battle)
    }

    /// Debuffer AI: Prioritizes inflicting status effects
    fn debuffer_ai(&self, battle: &BattleState) -> BattleAction {
        let mut rng = rand::thread_rng();

        // 60% chance to try using a debuff spell
        if rng.gen::<f32>() < 0.60 && !self.combat_stats.spells.is_empty() {
            // Look for debuff spells
            for spell_id in ["poison", "sleep", "blind", "slow", "silence"] {
                if self.combat_stats.spells.contains(&spell_id.to_string()) {
                    return BattleAction::Magic {
                        spell_id: spell_id.to_string(),
                        targets: self.find_spell_targets(battle, spell_id),
                    };
                }
            }
        }

        // Fallback to normal attack
        self.normal_ai(battle)
    }

    // ========== Helper methods ==========

    fn find_best_heal_spell(&self) -> Option<&str> {
        // Look for healing spells in order of power
        for spell_id in ["curaga", "cura", "cure"] {
            if self.combat_stats.spells.contains(&spell_id.to_string()) {
                return Some(spell_id);
            }
        }
        None
    }

    fn find_buff_spell(&self) -> Option<&str> {
        for spell_id in ["protect", "shell", "haste", "regen"] {
            if self.combat_stats.spells.contains(&spell_id.to_string()) {
                return Some(spell_id);
            }
        }
        None
    }

    fn find_best_damage_spell(&self, _battle: &BattleState) -> Option<&str> {
        // Return first available damage spell
        // In a more sophisticated implementation, this would check weaknesses
        for spell_id in &self.combat_stats.spells {
            if let Some(spell) = get_spell(spell_id) {
                if spell.spell_type == super::magic::SpellType::Damage {
                    return Some(spell_id);
                }
            }
        }
        None
    }

    fn find_strongest_spell(&self) -> Option<&str> {
        let mut best: Option<(&str, i32)> = None;

        for spell_id in &self.combat_stats.spells {
            if let Some(spell) = get_spell(spell_id) {
                if spell.spell_type == super::magic::SpellType::Damage {
                    if best.is_none() || spell.power > best.unwrap().1 {
                        best = Some((spell_id, spell.power));
                    }
                }
            }
        }

        best.map(|(id, _)| id.as_ref())
    }

    fn find_spell_targets(&self, battle: &BattleState, spell_id: &str) -> Vec<usize> {
        let mut rng = rand::thread_rng();

        if let Some(spell) = get_spell(spell_id) {
            match spell.target_type {
                super::magic::TargetType::SingleEnemy => {
                    let living: Vec<usize> = battle
                        .party
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_alive())
                        .map(|(i, _)| i)
                        .collect();

                    if living.is_empty() {
                        vec![]
                    } else {
                        vec![living[rng.gen_range(0..living.len())]]
                    }
                }
                super::magic::TargetType::AllEnemies => {
                    battle
                        .party
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.is_alive())
                        .map(|(i, _)| i)
                        .collect()
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::realm_of_ralnar::combat::{BattleCharacter, BattleRow};

    fn create_test_party() -> Vec<BattleCharacter> {
        vec![
            BattleCharacter {
                member_index: 0,
                hp: 100,
                hp_max: 100,
                mp: 50,
                mp_max: 50,
                status: Default::default(),
                defending: false,
                row: BattleRow::Front,
            },
            BattleCharacter {
                member_index: 1,
                hp: 80,
                hp_max: 80,
                mp: 100,
                mp_max: 100,
                status: Default::default(),
                defending: false,
                row: BattleRow::Back,
            },
        ]
    }

    fn create_test_enemy(ai_type: EnemyAIType) -> BattleEnemy {
        BattleEnemy {
            enemy_id: "test_enemy".to_string(),
            name: "Test Enemy".to_string(),
            hp: 100,
            hp_max: 100,
            combat_stats: CombatEnemyStats::default(),
            status: Default::default(),
            ai_type,
        }
    }

    fn create_test_battle(ai_type: EnemyAIType) -> BattleState {
        BattleState {
            party: create_test_party(),
            enemies: vec![create_test_enemy(ai_type)],
            turn_order: vec![],
            current_turn: 0,
            phase: super::super::combat::BattlePhase::Start,
            combat_log: vec![],
            selected_action: None,
            selected_target: None,
            is_boss_fight: false,
            is_guardian_fight: false,
        }
    }

    #[test]
    fn test_normal_ai_returns_attack() {
        let battle = create_test_battle(EnemyAIType::Normal);
        let enemy = &battle.enemies[0];

        let action = enemy.choose_action(&battle);
        match action {
            BattleAction::Attack { target } => {
                assert!(target < battle.party.len());
            }
            BattleAction::Defend => {
                // Also valid if no targets
            }
            _ => panic!("Normal AI should attack or defend"),
        }
    }

    #[test]
    fn test_aggressive_ai_targets_weakest() {
        let mut battle = create_test_battle(EnemyAIType::Aggressive);
        // Make second party member very weak
        battle.party[1].hp = 10;

        let enemy = &battle.enemies[0];

        // Run multiple times to check pattern
        let mut targeted_weak = 0;
        for _ in 0..20 {
            let action = enemy.choose_action(&battle);
            if let BattleAction::Attack { target } = action {
                if target == 1 {
                    targeted_weak += 1;
                }
            }
        }

        // Aggressive AI should consistently target the weakest
        assert_eq!(targeted_weak, 20, "Aggressive AI should always target weakest");
    }

    #[test]
    fn test_defensive_ai_defends_more() {
        let battle = create_test_battle(EnemyAIType::Defensive);
        let enemy = &battle.enemies[0];

        let mut defend_count = 0;
        for _ in 0..100 {
            let action = enemy.choose_action(&battle);
            if matches!(action, BattleAction::Defend) {
                defend_count += 1;
            }
        }

        // Defensive AI should defend fairly often
        assert!(defend_count > 20, "Defensive AI should defend sometimes");
    }

    #[test]
    fn test_guardian_ai_defensive_at_high_hp() {
        let battle = create_test_battle(EnemyAIType::Guardian);
        let enemy = &battle.enemies[0];

        let mut defend_count = 0;
        for _ in 0..100 {
            let action = enemy.choose_action(&battle);
            if matches!(action, BattleAction::Defend) {
                defend_count += 1;
            }
        }

        // Guardian should be very defensive at high HP
        assert!(
            defend_count > 50,
            "Guardian should be defensive at high HP (got {})",
            defend_count
        );
    }

    #[test]
    fn test_guardian_ai_aggressive_at_low_hp() {
        let mut battle = create_test_battle(EnemyAIType::Guardian);
        // Set enemy to low HP
        battle.enemies[0].hp = 20;

        let enemy = &battle.enemies[0];

        let mut attack_count = 0;
        for _ in 0..100 {
            let action = enemy.choose_action(&battle);
            if matches!(action, BattleAction::Attack { .. }) {
                attack_count += 1;
            }
        }

        // Guardian should attack more at low HP
        assert!(
            attack_count > 30,
            "Guardian should attack more at low HP (got {})",
            attack_count
        );
    }

    #[test]
    fn test_berserker_always_attacks() {
        let battle = create_test_battle(EnemyAIType::Berserker);
        let enemy = &battle.enemies[0];

        for _ in 0..50 {
            let action = enemy.choose_action(&battle);
            assert!(
                matches!(action, BattleAction::Attack { .. }),
                "Berserker should always attack"
            );
        }
    }

    #[test]
    fn test_mage_ai_uses_spells() {
        let mut battle = create_test_battle(EnemyAIType::Mage);
        battle.enemies[0].combat_stats.spells = vec!["fire".to_string(), "thunder".to_string()];

        let enemy = &battle.enemies[0];

        let mut spell_count = 0;
        for _ in 0..100 {
            let action = enemy.choose_action(&battle);
            if matches!(action, BattleAction::Magic { .. }) {
                spell_count += 1;
            }
        }

        // Mage should use spells most of the time
        assert!(spell_count > 60, "Mage should use spells often (got {})", spell_count);
    }

    #[test]
    fn test_healer_ai_heals_wounded() {
        let mut battle = create_test_battle(EnemyAIType::Healer);
        // Add another enemy that's wounded
        let mut wounded_enemy = create_test_enemy(EnemyAIType::Normal);
        wounded_enemy.hp = 30;
        battle.enemies.push(wounded_enemy);

        // Give healer a heal spell
        battle.enemies[0].combat_stats.spells = vec!["cure".to_string()];

        let enemy = battle.enemies[0].clone();

        let mut heal_count = 0;
        for _ in 0..100 {
            let action = enemy.choose_action(&battle);
            if let BattleAction::Magic { spell_id, .. } = action {
                if spell_id == "cure" {
                    heal_count += 1;
                }
            }
        }

        // Healer should try to heal wounded allies
        assert!(
            heal_count > 0,
            "Healer should attempt to heal wounded allies"
        );
    }

    #[test]
    fn test_ai_handles_no_targets() {
        let mut battle = create_test_battle(EnemyAIType::Aggressive);
        // Kill all party members
        for member in &mut battle.party {
            member.hp = 0;
        }

        let enemy = &battle.enemies[0];
        let action = enemy.choose_action(&battle);

        // Should defend when no targets available
        assert!(
            matches!(action, BattleAction::Defend),
            "Should defend when no valid targets"
        );
    }

    #[test]
    fn test_boss_ai_phases() {
        let mut battle = create_test_battle(EnemyAIType::Boss);
        battle.enemies[0].combat_stats.spells = vec!["fire".to_string(), "firaga".to_string()];

        // Test high HP phase
        battle.enemies[0].hp = 90;
        let enemy = &battle.enemies[0];
        let _action = enemy.choose_action(&battle);

        // Test medium HP phase
        battle.enemies[0].hp = 60;
        let enemy = &battle.enemies[0];
        let _action = enemy.choose_action(&battle);

        // Test low HP phase
        battle.enemies[0].hp = 20;
        let enemy = &battle.enemies[0];
        let _action = enemy.choose_action(&battle);

        // Just verify it doesn't crash and returns valid actions
    }

    #[test]
    fn test_enemy_ai_type_names() {
        assert_eq!(EnemyAIType::Normal.name(), "Normal");
        assert_eq!(EnemyAIType::Guardian.name(), "Guardian");
        assert_eq!(EnemyAIType::Boss.name(), "Boss");
    }
}
