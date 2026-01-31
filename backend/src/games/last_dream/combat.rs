//! Turn-based combat system for Last Dream
//! ATB-style timing with party vs enemy groups

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use super::data::{EnemyData, Element, SpellTarget, get_spell, get_item, get_encounter_enemies};
use super::party::Party;

/// Combat state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    /// Enemy combatants
    pub enemies: Vec<Enemy>,
    /// Combat log messages
    pub log: Vec<String>,
    /// Current phase
    pub phase: CombatPhase,
    /// Which party member's turn it is (if any)
    pub active_member: Option<usize>,
    /// Selected action
    pub selected_action: Option<CombatAction>,
    /// Target selection state
    pub selecting_target: bool,
    /// ATB tick counter
    pub tick: u32,
    /// Is combat finished
    pub finished: bool,
    /// Total EXP to award
    pub exp_reward: u64,
    /// Total gold to award
    pub gold_reward: u32,
    /// Did party win?
    pub victory: bool,
    /// Background area (for rendering)
    pub area: String,
}

/// Combat phase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatPhase {
    /// Waiting for ATB gauges
    Waiting,
    /// Player selecting action
    SelectAction,
    /// Player selecting target
    SelectTarget,
    /// Executing action
    Executing,
    /// Combat over
    Finished,
}

/// An enemy in combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub key: String,
    pub name: String,
    pub hp: u32,
    pub hp_max: u32,
    pub mp: u32,
    pub attack: u32,
    pub defense: u32,
    pub magic: u32,
    pub speed: u32,
    pub weakness: Element,
    pub resistance: Element,
    pub atb_gauge: u32,
    pub is_boss: bool,
}

impl Enemy {
    pub fn from_data(data: &EnemyData) -> Self {
        Self {
            key: data.key.to_string(),
            name: data.name.to_string(),
            hp: data.hp,
            hp_max: data.hp,
            mp: data.mp,
            attack: data.attack,
            defense: data.defense,
            magic: data.magic,
            speed: data.speed,
            weakness: data.weakness,
            resistance: data.resistance,
            atb_gauge: 0,
            is_boss: data.is_boss,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn tick_atb(&mut self) -> bool {
        if !self.is_alive() {
            return false;
        }
        self.atb_gauge = (self.atb_gauge + self.speed).min(100);
        self.atb_gauge >= 100
    }

    pub fn reset_atb(&mut self) {
        self.atb_gauge = 0;
    }
}

/// Player action in combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatAction {
    Attack,
    Magic { spell_key: String },
    Item { item_key: String },
    Defend,
    Run,
}

/// Result of a combat action
#[derive(Debug, Clone)]
pub enum ActionResult {
    Damage { target: String, amount: u32, element: Option<Element> },
    Heal { target: String, amount: u32 },
    Miss { target: String },
    StatusApplied { target: String, status: String },
    Defended { target: String },
    RunSuccess,
    RunFailed,
    EnemyDefeated { name: String, exp: u64, gold: u32 },
    PartyMemberDown { name: String },
}

impl CombatState {
    /// Start a new combat encounter
    pub fn new_encounter(party_level: u8, area: &str) -> Self {
        let enemy_count = thread_rng().gen_range(1..=4);
        let enemy_data = get_encounter_enemies(party_level, enemy_count);

        let enemies: Vec<Enemy> = enemy_data.iter()
            .map(|data| Enemy::from_data(data))
            .collect();

        let exp_reward: u64 = enemy_data.iter().map(|e| e.exp as u64).sum();
        let gold_reward: u32 = enemy_data.iter().map(|e| e.gold).sum();

        Self {
            enemies,
            log: vec!["Enemies appear!".to_string()],
            phase: CombatPhase::Waiting,
            active_member: None,
            selected_action: None,
            selecting_target: false,
            tick: 0,
            finished: false,
            exp_reward,
            gold_reward,
            victory: false,
            area: area.to_string(),
        }
    }

    /// Start a boss encounter
    pub fn boss_encounter(boss_key: &str, area: &str) -> Self {
        let boss_data = super::data::get_enemy(boss_key)
            .expect("Boss not found");

        let boss = Enemy::from_data(boss_data);

        Self {
            enemies: vec![boss],
            log: vec![format!("{} appears!", boss_data.name)],
            phase: CombatPhase::Waiting,
            active_member: None,
            selected_action: None,
            selecting_target: false,
            tick: 0,
            finished: false,
            exp_reward: boss_data.exp as u64,
            gold_reward: boss_data.gold,
            victory: false,
            area: area.to_string(),
        }
    }

    /// Tick the combat system
    pub fn tick(&mut self, party: &mut Party) -> Vec<ActionResult> {
        if self.finished {
            return Vec::new();
        }

        self.tick += 1;
        let mut results = Vec::new();

        // Check victory/defeat
        if !party.is_alive() {
            self.finished = true;
            self.victory = false;
            self.phase = CombatPhase::Finished;
            self.log.push("Party has been defeated!".to_string());
            return results;
        }

        if self.enemies.iter().all(|e| !e.is_alive()) {
            self.finished = true;
            self.victory = true;
            self.phase = CombatPhase::Finished;
            self.log.push("Victory!".to_string());
            return results;
        }

        // Don't tick during action selection
        if self.phase != CombatPhase::Waiting {
            return results;
        }

        // Tick party ATB
        for (i, member) in party.members.iter_mut().enumerate() {
            if member.tick_atb() && self.active_member.is_none() {
                self.active_member = Some(i);
                self.phase = CombatPhase::SelectAction;
                self.log.push(format!("{}'s turn!", member.name));
            }
        }

        // Tick enemy ATB and collect actions
        let mut enemy_actions: Vec<usize> = Vec::new();
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            if enemy.tick_atb() {
                enemy_actions.push(i);
                enemy.reset_atb();
            }
        }

        // Process enemy actions
        for enemy_idx in enemy_actions {
            let action_results = self.process_enemy_action(enemy_idx, party);
            results.extend(action_results);
        }

        results
    }

    /// Enemy AI action (processes by index to avoid borrow issues)
    fn process_enemy_action(&mut self, enemy_idx: usize, party: &mut Party) -> Vec<ActionResult> {
        let mut results = Vec::new();
        let mut rng = thread_rng();

        // Get enemy info
        let (enemy_name, enemy_attack) = {
            let enemy = &self.enemies[enemy_idx];
            (enemy.name.clone(), enemy.attack)
        };

        // Find living targets
        let living: Vec<usize> = party.members.iter()
            .enumerate()
            .filter(|(_, m)| m.is_alive())
            .map(|(i, _)| i)
            .collect();

        if living.is_empty() {
            return results;
        }

        // Pick random target
        let target_idx = living[rng.gen_range(0..living.len())];
        let target = &mut party.members[target_idx];

        // Calculate damage
        let base_damage = enemy_attack;
        let variance: u32 = rng.gen_range(0..10);
        let damage = base_damage.saturating_sub(target.defense_power() / 2) + variance;
        let actual_damage = target.take_damage(damage);

        self.log.push(format!("{} attacks {} for {} damage!",
            enemy_name, target.name, actual_damage));

        results.push(ActionResult::Damage {
            target: target.name.clone(),
            amount: actual_damage,
            element: None,
        });

        if !target.is_alive() {
            self.log.push(format!("{} has fallen!", target.name));
            results.push(ActionResult::PartyMemberDown { name: target.name.clone() });
        }

        results
    }

    /// Execute player action
    pub fn execute_action(
        &mut self,
        party: &mut Party,
        action: CombatAction,
        target: Option<usize>,
    ) -> Vec<ActionResult> {
        let member_idx = match self.active_member {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        let mut rng = thread_rng();

        match action {
            CombatAction::Attack => {
                let target_idx = target.unwrap_or(0);
                if target_idx < self.enemies.len() && self.enemies[target_idx].is_alive() {
                    let member = &mut party.members[member_idx];
                    let damage = member.calculate_damage();
                    let variance: u32 = rng.gen_range(0..10);
                    let enemy = &mut self.enemies[target_idx];
                    let defense = enemy.defense / 2;
                    let actual_damage = (damage + variance).saturating_sub(defense).max(1);

                    if enemy.hp > actual_damage {
                        enemy.hp -= actual_damage;
                    } else {
                        enemy.hp = 0;
                    }

                    self.log.push(format!("{} attacks {} for {} damage!",
                        member.name, enemy.name, actual_damage));

                    results.push(ActionResult::Damage {
                        target: enemy.name.clone(),
                        amount: actual_damage,
                        element: None,
                    });

                    if !enemy.is_alive() {
                        self.log.push(format!("{} is defeated!", enemy.name));
                        results.push(ActionResult::EnemyDefeated {
                            name: enemy.name.clone(),
                            exp: 0, // Will be awarded at end
                            gold: 0,
                        });
                    }

                    member.reset_atb();
                }
            }

            CombatAction::Magic { spell_key } => {
                if let Some(spell) = get_spell(&spell_key) {
                    // First, try to use MP and get member info before any target borrowing
                    let (member_name, magic_power) = {
                        let member = &mut party.members[member_idx];
                        if !member.use_mp(spell.mp_cost) {
                            self.log.push("Not enough MP!".to_string());
                            return results;
                        }
                        (member.name.clone(), member.magic_power())
                    };

                    match spell.target {
                        SpellTarget::SingleEnemy => {
                            let target_idx = target.unwrap_or(0);
                            if target_idx < self.enemies.len() {
                                let enemy = &mut self.enemies[target_idx];
                                let mut damage = spell.power + magic_power;

                                // Weakness/resistance
                                if enemy.weakness == spell.element {
                                    damage = damage * 3 / 2;
                                } else if enemy.resistance == spell.element {
                                    damage = damage / 2;
                                }

                                if enemy.hp > damage {
                                    enemy.hp -= damage;
                                } else {
                                    enemy.hp = 0;
                                }

                                self.log.push(format!("{} casts {}! {} takes {} damage!",
                                    member_name, spell.name, enemy.name, damage));

                                results.push(ActionResult::Damage {
                                    target: enemy.name.clone(),
                                    amount: damage,
                                    element: Some(spell.element),
                                });

                                if !enemy.is_alive() {
                                    results.push(ActionResult::EnemyDefeated {
                                        name: enemy.name.clone(),
                                        exp: 0,
                                        gold: 0,
                                    });
                                }
                            }
                        }
                        SpellTarget::AllEnemies => {
                            for enemy in &mut self.enemies {
                                if enemy.is_alive() {
                                    let mut damage = spell.power + magic_power / 2;

                                    if enemy.weakness == spell.element {
                                        damage = damage * 3 / 2;
                                    } else if enemy.resistance == spell.element {
                                        damage = damage / 2;
                                    }

                                    if enemy.hp > damage {
                                        enemy.hp -= damage;
                                    } else {
                                        enemy.hp = 0;
                                    }

                                    results.push(ActionResult::Damage {
                                        target: enemy.name.clone(),
                                        amount: damage,
                                        element: Some(spell.element),
                                    });
                                }
                            }
                            self.log.push(format!("{} casts {}!", member_name, spell.name));
                        }
                        SpellTarget::SingleAlly if spell.is_healing => {
                            let target_idx = target.unwrap_or(member_idx);
                            if target_idx < party.members.len() {
                                let heal_amount = spell.power + magic_power / 2;
                                let ally = &mut party.members[target_idx];
                                ally.heal(heal_amount);
                                let ally_name = ally.name.clone();

                                self.log.push(format!("{} casts {}! {} healed for {} HP!",
                                    member_name, spell.name, ally_name, heal_amount));

                                results.push(ActionResult::Heal {
                                    target: ally_name,
                                    amount: heal_amount,
                                });
                            }
                        }
                        SpellTarget::AllAllies if spell.is_healing => {
                            let heal_amount = spell.power + magic_power / 3;
                            for ally in &mut party.members {
                                if ally.is_alive() {
                                    ally.heal(heal_amount);
                                    results.push(ActionResult::Heal {
                                        target: ally.name.clone(),
                                        amount: heal_amount,
                                    });
                                }
                            }
                            self.log.push(format!("{} casts {}!", member_name, spell.name));
                        }
                        SpellTarget::Self_ => {
                            // Focus ability
                            let member = &mut party.members[member_idx];
                            if spell_key == "focus" {
                                member.status.focused = true;
                                self.log.push(format!("{} focuses power!", member_name));
                            } else if spell_key == "chakra" {
                                let heal = member.hp_max / 4;
                                member.heal(heal);
                                self.log.push(format!("{} uses Chakra! Healed {} HP!",
                                    member_name, heal));
                            }
                        }
                        _ => {}
                    }

                    party.members[member_idx].reset_atb();
                }
            }

            CombatAction::Item { item_key } => {
                if let Some(item) = get_item(&item_key) {
                    let target_idx = target.unwrap_or(member_idx);
                    if target_idx < party.members.len() {
                        let target_member = &mut party.members[target_idx];

                        if item.heal_hp > 0 {
                            target_member.heal(item.heal_hp);
                            results.push(ActionResult::Heal {
                                target: target_member.name.clone(),
                                amount: item.heal_hp,
                            });
                        }
                        if item.heal_mp > 0 {
                            target_member.restore_mp(item.heal_mp);
                        }
                        if item.revive && target_member.status.dead {
                            target_member.revive(50);
                        }

                        self.log.push(format!("Used {} on {}!",
                            item.name, target_member.name));
                    }

                    party.members[member_idx].reset_atb();
                }
            }

            CombatAction::Defend => {
                let member = &mut party.members[member_idx];
                member.status.protected = true;
                self.log.push(format!("{} defends!", member.name));
                member.reset_atb();
                results.push(ActionResult::Defended { target: member.name.clone() });
            }

            CombatAction::Run => {
                let member = &party.members[member_idx];
                let run_chance = 50 + (member.agility as i32 - 10) * 2;
                let roll: i32 = rng.gen_range(0..100);

                if roll < run_chance && !self.enemies.iter().any(|e| e.is_boss) {
                    self.log.push("Escaped successfully!".to_string());
                    self.finished = true;
                    self.victory = false; // No rewards for running
                    self.exp_reward = 0;
                    self.gold_reward = 0;
                    results.push(ActionResult::RunSuccess);
                } else {
                    self.log.push("Couldn't escape!".to_string());
                    results.push(ActionResult::RunFailed);
                }

                party.members[member_idx].reset_atb();
            }
        }

        self.active_member = None;
        self.phase = CombatPhase::Waiting;
        self.selected_action = None;

        results
    }

    /// Get living enemies for target selection
    pub fn living_enemies(&self) -> Vec<(usize, &Enemy)> {
        self.enemies.iter()
            .enumerate()
            .filter(|(_, e)| e.is_alive())
            .collect()
    }

    /// Check if combat is over
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Get the most recent log entries
    pub fn recent_log(&self, count: usize) -> Vec<&str> {
        self.log.iter()
            .rev()
            .take(count)
            .map(|s| s.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::party::{Character, CharacterClass};

    fn create_test_party() -> Party {
        let mut party = Party::new();
        party.add_member(Character::new("Hero".to_string(), CharacterClass::Warrior));
        party.add_member(Character::new("Mage".to_string(), CharacterClass::Mage));
        party
    }

    #[test]
    fn test_combat_initialization() {
        let combat = CombatState::new_encounter(5, "forest");
        assert!(!combat.enemies.is_empty());
        assert!(!combat.finished);
    }

    #[test]
    fn test_enemy_from_data() {
        let data = super::super::data::get_enemy("goblin").unwrap();
        let enemy = Enemy::from_data(data);
        assert_eq!(enemy.hp, data.hp);
        assert!(enemy.is_alive());
    }

    #[test]
    fn test_attack_action() {
        let mut party = create_test_party();
        let mut combat = CombatState::new_encounter(1, "forest");

        combat.active_member = Some(0);
        combat.phase = CombatPhase::SelectAction;

        let initial_hp = combat.enemies[0].hp;
        let _results = combat.execute_action(&mut party, CombatAction::Attack, Some(0));

        // Enemy should have taken damage
        assert!(combat.enemies[0].hp < initial_hp || !combat.enemies[0].is_alive());
    }

    #[test]
    fn test_defend_action() {
        let mut party = create_test_party();
        let mut combat = CombatState::new_encounter(1, "forest");

        combat.active_member = Some(0);
        let results = combat.execute_action(&mut party, CombatAction::Defend, None);

        assert!(results.iter().any(|r| matches!(r, ActionResult::Defended { .. })));
    }

    #[test]
    fn test_atb_ticking() {
        let mut party = create_test_party();
        let mut combat = CombatState::new_encounter(1, "forest");

        // Tick multiple times
        for _ in 0..20 {
            combat.tick(&mut party);
        }

        // ATB gauges should have increased
        assert!(party.members[0].atb_gauge > 0 || combat.active_member.is_some());
    }

    #[test]
    fn test_boss_encounter() {
        let combat = CombatState::boss_encounter("earth_fiend", "dungeon");
        assert_eq!(combat.enemies.len(), 1);
        assert!(combat.enemies[0].is_boss);
    }

    #[test]
    fn test_living_enemies() {
        let mut combat = CombatState::new_encounter(1, "forest");
        let initial_count = combat.living_enemies().len();

        // Kill first enemy
        combat.enemies[0].hp = 0;
        let after_count = combat.living_enemies().len();

        assert_eq!(after_count, initial_count - 1);
    }
}
