//! FF1-style Turn-Based Combat System for Realm of Ralnar
//!
//! This module implements a traditional turn-based combat system with:
//! - Party vs enemy group battles
//! - Turn order based on agility
//! - Physical and magical attacks
//! - Status effects and elemental interactions
//! - Special Guardian AI for foreshadowing

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::battle_ai::{CombatEnemyStats, EnemyAIType};
use super::damage::{
    calculate_flee_chance, calculate_healing, calculate_magic_damage, calculate_physical_damage,
    calculate_poison_damage, calculate_regen_healing, calculate_status_chance, AttackerStats,
    DefenderStats,
};
use super::magic::{get_spell, Element, SpellType};
use super::status::{StatusCollection, StatusEffect};

/// The main battle state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    /// Party members in combat
    pub party: Vec<BattleCharacter>,
    /// Enemy combatants
    pub enemies: Vec<BattleEnemy>,
    /// Order of actions this round
    pub turn_order: Vec<BattleActor>,
    /// Current position in turn order
    pub current_turn: usize,
    /// Current battle phase
    pub phase: BattlePhase,
    /// Combat log messages
    pub combat_log: Vec<String>,
    /// Currently selected action (for UI)
    pub selected_action: Option<BattleAction>,
    /// Currently selected target (for UI)
    pub selected_target: Option<usize>,
    /// Is this a boss fight (can't flee)?
    pub is_boss_fight: bool,
    /// Is this a guardian fight (special AI)?
    pub is_guardian_fight: bool,
}

/// A party member in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleCharacter {
    /// Index in the main party structure
    pub member_index: usize,
    /// Current HP
    pub hp: i32,
    /// Maximum HP
    pub hp_max: i32,
    /// Current MP
    pub mp: i32,
    /// Maximum MP
    pub mp_max: i32,
    /// Active status effects
    pub status: StatusCollection,
    /// Is currently defending?
    pub defending: bool,
    /// Front or back row
    pub row: BattleRow,
}

/// An enemy in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEnemy {
    /// Unique enemy type ID
    pub enemy_id: String,
    /// Display name
    pub name: String,
    /// Current HP
    pub hp: i32,
    /// Maximum HP
    pub hp_max: i32,
    /// Combat-specific stats (for spell/AI usage)
    pub combat_stats: CombatEnemyStats,
    /// Active status effects
    pub status: StatusCollection,
    /// AI behavior type
    pub ai_type: EnemyAIType,
}

/// Represents who is acting in the turn order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleActor {
    /// Party member by index
    Party(usize),
    /// Enemy by index
    Enemy(usize),
}

/// Current phase of the battle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePhase {
    /// Battle is starting
    Start,
    /// Character is selecting their action
    SelectAction { character_index: usize },
    /// Character is selecting a target for their action
    SelectTarget { character_index: usize },
    /// Character is selecting a spell
    SelectSpell { character_index: usize },
    /// Character is selecting an item
    SelectItem { character_index: usize },
    /// Actions are being executed
    ExecutingActions,
    /// Enemy's turn
    EnemyTurn { enemy_index: usize },
    /// Battle won
    Victory,
    /// Battle lost
    Defeat,
    /// Successfully fled
    Fled,
}

/// An action that can be taken in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleAction {
    /// Physical attack on a target
    Attack { target: usize },
    /// Cast a spell
    Magic { spell_id: String, targets: Vec<usize> },
    /// Use an item
    Item { item_id: String, target: usize },
    /// Defend (reduce damage taken)
    Defend,
    /// Attempt to flee
    Flee,
}

/// Front or back row positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleRow {
    /// Front row - normal damage dealt and taken
    Front,
    /// Back row - reduced physical damage dealt and taken
    Back,
}

/// An event that occurred during battle (for animation/display)
#[derive(Debug, Clone)]
pub enum BattleEvent {
    /// Damage was dealt
    Damage {
        attacker: BattleActor,
        target: BattleActor,
        amount: i32,
        is_critical: bool,
        element: Option<Element>,
    },
    /// Healing occurred
    Heal {
        target: BattleActor,
        amount: i32,
    },
    /// Attack missed
    Miss {
        attacker: BattleActor,
        target: BattleActor,
    },
    /// Status effect applied
    StatusApplied {
        target: BattleActor,
        status: StatusEffect,
    },
    /// Status effect removed
    StatusRemoved {
        target: BattleActor,
        status: StatusEffect,
    },
    /// Status effect resisted
    StatusResisted {
        target: BattleActor,
        status: StatusEffect,
    },
    /// Combatant is defending
    Defending {
        actor: BattleActor,
    },
    /// Combatant died
    Died {
        actor: BattleActor,
    },
    /// Flee attempt
    FleeAttempt {
        success: bool,
    },
    /// Spell cast
    SpellCast {
        caster: BattleActor,
        spell_name: String,
    },
    /// Item used
    ItemUsed {
        user: BattleActor,
        item_name: String,
    },
    /// Combat message
    Message {
        text: String,
    },
}

/// Rewards from winning a battle
#[derive(Debug, Clone, Default)]
pub struct BattleRewards {
    pub exp: u64,
    pub gold: i32,
    pub items: Vec<String>,
}

/// Data needed to spawn an enemy
#[derive(Debug, Clone)]
pub struct EnemySpawn {
    pub enemy_id: String,
    pub name: String,
    pub hp: i32,
    pub combat_stats: CombatEnemyStats,
    pub ai_type: EnemyAIType,
    pub exp: u64,
    pub gold: i32,
}

impl BattleCharacter {
    /// Check if character is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0 && !self.status.is_dead()
    }

    /// Check if character can act
    pub fn can_act(&self) -> bool {
        self.is_alive() && self.status.can_act()
    }

    /// Get effective agility for turn order
    pub fn effective_agility(&self, base_agility: i32) -> i32 {
        let mut agi = base_agility;

        if self.status.has(StatusEffect::Haste) {
            agi = agi * 3 / 2; // +50%
        }
        if self.status.has(StatusEffect::Slow) {
            agi = agi / 2; // -50%
        }

        agi.max(1)
    }
}

impl BattleEnemy {
    /// Check if enemy is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0 && !self.status.is_dead()
    }

    /// Check if enemy can act
    pub fn can_act(&self) -> bool {
        self.is_alive() && self.status.can_act()
    }

    /// Get effective agility for turn order
    pub fn effective_agility(&self) -> i32 {
        let mut agi = self.combat_stats.agility;

        if self.status.has(StatusEffect::Haste) {
            agi = agi * 3 / 2;
        }
        if self.status.has(StatusEffect::Slow) {
            agi = agi / 2;
        }

        agi.max(1)
    }
}

impl BattleState {
    /// Create a new battle state
    pub fn new(party: Vec<BattleCharacter>, enemies: Vec<EnemySpawn>) -> Self {
        let is_boss = enemies.iter().any(|e| e.ai_type == EnemyAIType::Boss);
        let is_guardian = enemies.iter().any(|e| e.ai_type == EnemyAIType::Guardian);

        let battle_enemies: Vec<BattleEnemy> = enemies
            .into_iter()
            .map(|spawn| BattleEnemy {
                enemy_id: spawn.enemy_id,
                name: spawn.name,
                hp: spawn.hp,
                hp_max: spawn.hp,
                combat_stats: spawn.combat_stats,
                status: StatusCollection::new(),
                ai_type: spawn.ai_type,
            })
            .collect();

        let mut state = Self {
            party,
            enemies: battle_enemies,
            turn_order: Vec::new(),
            current_turn: 0,
            phase: BattlePhase::Start,
            combat_log: Vec::new(),
            selected_action: None,
            selected_target: None,
            is_boss_fight: is_boss,
            is_guardian_fight: is_guardian,
        };

        state.log("Battle start!");
        state
    }

    /// Calculate turn order based on agility
    pub fn calculate_turn_order(&mut self, party_agilities: &[i32]) {
        let mut actors: Vec<(BattleActor, i32)> = Vec::new();
        let mut rng = rand::thread_rng();

        // Add party members
        for (i, character) in self.party.iter().enumerate() {
            if character.can_act() {
                let base_agi = party_agilities.get(i).copied().unwrap_or(10);
                let agi = character.effective_agility(base_agi);
                // Add small random variance to break ties
                let variance = rng.gen_range(0..5);
                actors.push((BattleActor::Party(i), agi + variance));
            }
        }

        // Add enemies
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.can_act() {
                let agi = enemy.effective_agility();
                let variance = rng.gen_range(0..5);
                actors.push((BattleActor::Enemy(i), agi + variance));
            }
        }

        // Sort by agility (highest first)
        actors.sort_by(|a, b| b.1.cmp(&a.1));

        self.turn_order = actors.into_iter().map(|(actor, _)| actor).collect();
        self.current_turn = 0;
    }

    /// Process a player action
    pub fn process_action(&mut self, action: BattleAction, party_stats: &[AttackerStats]) -> Vec<BattleEvent> {
        let mut events = Vec::new();

        // Get current actor
        let actor = match self.turn_order.get(self.current_turn) {
            Some(a) => *a,
            None => return events,
        };

        let character_index = match actor {
            BattleActor::Party(i) => i,
            BattleActor::Enemy(_) => return events, // Enemy actions handled separately
        };

        match action {
            BattleAction::Attack { target } => {
                events.extend(self.execute_attack(character_index, target, party_stats));
            }
            BattleAction::Magic { ref spell_id, ref targets } => {
                events.extend(self.execute_magic(character_index, spell_id, targets, party_stats));
            }
            BattleAction::Item { ref item_id, target } => {
                events.extend(self.execute_item(character_index, item_id, target));
            }
            BattleAction::Defend => {
                self.party[character_index].defending = true;
                events.push(BattleEvent::Defending {
                    actor: BattleActor::Party(character_index),
                });
                self.log(&format!("{} is defending!", self.get_character_name(character_index)));
            }
            BattleAction::Flee => {
                events.extend(self.attempt_flee(party_stats));
            }
        }

        self.selected_action = None;
        self.selected_target = None;

        events
    }

    /// Execute a physical attack
    fn execute_attack(
        &mut self,
        attacker_idx: usize,
        target_idx: usize,
        party_stats: &[AttackerStats],
    ) -> Vec<BattleEvent> {
        let mut events = Vec::new();

        if target_idx >= self.enemies.len() || !self.enemies[target_idx].is_alive() {
            return events;
        }

        let attacker_stats = party_stats.get(attacker_idx).cloned().unwrap_or(AttackerStats {
            strength: 10,
            agility: 10,
            intelligence: 10,
            luck: 10,
            level: 1,
            weapon_attack: 5,
            has_protect: self.party[attacker_idx].status.has(StatusEffect::Protect),
            has_haste: self.party[attacker_idx].status.has(StatusEffect::Haste),
            is_blind: self.party[attacker_idx].status.has(StatusEffect::Blind),
            is_berserk: self.party[attacker_idx].status.has(StatusEffect::Berserk),
        });

        let enemy = &self.enemies[target_idx];
        let defender_stats = DefenderStats {
            defense: enemy.combat_stats.defense,
            magic_defense: enemy.combat_stats.magic_defense,
            agility: enemy.combat_stats.agility,
            level: enemy.combat_stats.level,
            has_protect: enemy.status.has(StatusEffect::Protect),
            has_shell: enemy.status.has(StatusEffect::Shell),
            is_defending: false, // Enemies don't typically "defend"
            is_back_row: false,
            weakness: None, // Could add enemy weaknesses here
            resistance: None,
            absorb: None,
        };

        let result = calculate_physical_damage(&attacker_stats, &defender_stats);

        if result.is_miss {
            events.push(BattleEvent::Miss {
                attacker: BattleActor::Party(attacker_idx),
                target: BattleActor::Enemy(target_idx),
            });
            self.log(&format!("{} missed!", self.get_character_name(attacker_idx)));
        } else {
            // Clone enemy name before mutably borrowing to avoid borrow conflicts
            let enemy_name = self.enemies[target_idx].name.clone();

            // Remove sleep on damage
            let removed = self.enemies[target_idx].status.on_damage();
            for status in removed {
                events.push(BattleEvent::StatusRemoved {
                    target: BattleActor::Enemy(target_idx),
                    status,
                });
            }

            self.enemies[target_idx].hp = (self.enemies[target_idx].hp - result.damage).max(0);

            events.push(BattleEvent::Damage {
                attacker: BattleActor::Party(attacker_idx),
                target: BattleActor::Enemy(target_idx),
                amount: result.damage,
                is_critical: result.is_critical,
                element: None,
            });

            let attacker_name = self.get_character_name(attacker_idx);
            if result.is_critical {
                self.log(&format!(
                    "Critical hit! {} deals {} damage to {}!",
                    attacker_name,
                    result.damage,
                    enemy_name
                ));
            } else {
                self.log(&format!(
                    "{} deals {} damage to {}!",
                    attacker_name,
                    result.damage,
                    enemy_name
                ));
            }

            if self.enemies[target_idx].hp <= 0 {
                self.enemies[target_idx].status.add(StatusEffect::Dead);
                events.push(BattleEvent::Died {
                    actor: BattleActor::Enemy(target_idx),
                });
                self.log(&format!("{} is defeated!", enemy_name));
            }
        }

        events
    }

    /// Execute a magic spell
    fn execute_magic(
        &mut self,
        caster_idx: usize,
        spell_id: &str,
        targets: &[usize],
        party_stats: &[AttackerStats],
    ) -> Vec<BattleEvent> {
        let mut events = Vec::new();

        let spell = match get_spell(spell_id) {
            Some(s) => s,
            None => return events,
        };

        // Check if silenced
        if self.party[caster_idx].status.has(StatusEffect::Silence) {
            self.log(&format!(
                "{} is silenced and cannot cast spells!",
                self.get_character_name(caster_idx)
            ));
            return events;
        }

        // Check MP cost
        if self.party[caster_idx].mp < spell.mp_cost {
            self.log("Not enough MP!");
            return events;
        }

        // Deduct MP
        self.party[caster_idx].mp -= spell.mp_cost;

        events.push(BattleEvent::SpellCast {
            caster: BattleActor::Party(caster_idx),
            spell_name: spell.name.to_string(),
        });
        self.log(&format!(
            "{} casts {}!",
            self.get_character_name(caster_idx),
            spell.name
        ));

        let caster_int = party_stats
            .get(caster_idx)
            .map(|s| s.intelligence)
            .unwrap_or(10);

        match spell.spell_type {
            SpellType::Damage => {
                for &target in targets {
                    if target < self.enemies.len() && self.enemies[target].is_alive() {
                        let enemy = &self.enemies[target];
                        let defender_stats = DefenderStats {
                            defense: enemy.combat_stats.defense,
                            magic_defense: enemy.combat_stats.magic_defense,
                            agility: enemy.combat_stats.agility,
                            level: enemy.combat_stats.level,
                            has_protect: enemy.status.has(StatusEffect::Protect),
                            has_shell: enemy.status.has(StatusEffect::Shell),
                            is_defending: false,
                            is_back_row: false,
                            weakness: None,
                            resistance: None,
                            absorb: None,
                        };

                        let result = calculate_magic_damage(caster_int, spell, &defender_stats);

                        if result.is_absorb {
                            let enemy_name = self.enemies[target].name.clone();
                            let enemy = &mut self.enemies[target];
                            enemy.hp = (enemy.hp - result.damage).min(enemy.hp_max);
                            events.push(BattleEvent::Heal {
                                target: BattleActor::Enemy(target),
                                amount: -result.damage,
                            });
                            self.log(&format!("{} absorbs the spell!", enemy_name));
                        } else {
                            // Get enemy name before mutable borrow
                            let enemy_name = self.enemies[target].name.clone();

                            // Remove sleep on damage
                            let removed = self.enemies[target].status.on_damage();
                            for status in &removed {
                                events.push(BattleEvent::StatusRemoved {
                                    target: BattleActor::Enemy(target),
                                    status: *status,
                                });
                            }

                            // Apply damage
                            self.enemies[target].hp = (self.enemies[target].hp - result.damage).max(0);
                            let enemy_hp = self.enemies[target].hp;

                            events.push(BattleEvent::Damage {
                                attacker: BattleActor::Party(caster_idx),
                                target: BattleActor::Enemy(target),
                                amount: result.damage,
                                is_critical: false,
                                element: Some(spell.element),
                            });

                            let msg = if result.is_weak {
                                format!("It's super effective! {} takes {} damage!", enemy_name, result.damage)
                            } else if result.is_resist {
                                format!("{} resists! {} damage.", enemy_name, result.damage)
                            } else {
                                format!("{} takes {} damage!", enemy_name, result.damage)
                            };
                            self.log(&msg);

                            if enemy_hp <= 0 {
                                self.enemies[target].status.add(StatusEffect::Dead);
                                events.push(BattleEvent::Died {
                                    actor: BattleActor::Enemy(target),
                                });
                                self.log(&format!("{} is defeated!", enemy_name));
                            }
                        }
                    }
                }
            }
            SpellType::Heal => {
                for &target in targets {
                    if target < self.party.len() && self.party[target].is_alive() {
                        let healing = calculate_healing(caster_int, spell);
                        let character = &mut self.party[target];
                        let old_hp = character.hp;
                        character.hp = (character.hp + healing).min(character.hp_max);
                        let actual_heal = character.hp - old_hp;

                        events.push(BattleEvent::Heal {
                            target: BattleActor::Party(target),
                            amount: actual_heal,
                        });
                        self.log(&format!(
                            "{} recovers {} HP!",
                            self.get_character_name(target),
                            actual_heal
                        ));
                    }
                }
            }
            SpellType::Buff => {
                let status = match spell.id {
                    "protect" | "protectga" => StatusEffect::Protect,
                    "shell" | "shellga" => StatusEffect::Shell,
                    "haste" => StatusEffect::Haste,
                    "regen" => StatusEffect::Regen,
                    _ => return events,
                };

                for &target in targets {
                    if target < self.party.len() && self.party[target].is_alive() {
                        self.party[target].status.add(status);
                        events.push(BattleEvent::StatusApplied {
                            target: BattleActor::Party(target),
                            status,
                        });
                        self.log(&format!(
                            "{} gains {}!",
                            self.get_character_name(target),
                            status.name()
                        ));
                    }
                }
            }
            SpellType::Debuff | SpellType::StatusInflict => {
                let status = match spell.id {
                    "poison" => StatusEffect::Poison,
                    "sleep" => StatusEffect::Sleep,
                    "silence" => StatusEffect::Silence,
                    "blind" => StatusEffect::Blind,
                    "slow" => StatusEffect::Slow,
                    "break" => StatusEffect::Stone,
                    "death" => StatusEffect::Dead,
                    _ => return events,
                };

                let caster_level = party_stats
                    .get(caster_idx)
                    .map(|s| s.level)
                    .unwrap_or(1);

                for &target in targets {
                    if target < self.enemies.len() && self.enemies[target].is_alive() {
                        let enemy_name = self.enemies[target].name.clone();
                        let is_immune = (status == StatusEffect::Dead && self.enemies[target].combat_stats.death_immune)
                            || self.enemies[target].combat_stats.status_immune;
                        let enemy_level = self.enemies[target].combat_stats.level;

                        let chance = calculate_status_chance(
                            spell.power,
                            caster_level,
                            enemy_level,
                            is_immune,
                        );

                        let mut rng = rand::thread_rng();
                        if rng.gen::<f32>() < chance {
                            self.enemies[target].status.add(status);

                            if status == StatusEffect::Dead {
                                self.enemies[target].hp = 0;
                            }

                            events.push(BattleEvent::StatusApplied {
                                target: BattleActor::Enemy(target),
                                status,
                            });
                            self.log(&format!("{} is afflicted with {}!", enemy_name, status.name()));

                            if status == StatusEffect::Dead {
                                events.push(BattleEvent::Died {
                                    actor: BattleActor::Enemy(target),
                                });
                                self.log(&format!("{} is defeated!", enemy_name));
                            }
                        } else {
                            events.push(BattleEvent::StatusResisted {
                                target: BattleActor::Enemy(target),
                                status,
                            });
                            self.log(&format!("{} resisted!", enemy_name));
                        }
                    }
                }
            }
            SpellType::StatusCure => {
                for &target in targets {
                    if target < self.party.len() {
                        let character = &mut self.party[target];

                        if spell.id == "esuna" {
                            character.status.clear_negative();
                            self.log(&format!(
                                "{}'s ailments are cured!",
                                self.get_character_name(target)
                            ));
                        } else if spell.id == "stona" {
                            if character.status.has(StatusEffect::Stone) {
                                character.status.remove(StatusEffect::Stone);
                                events.push(BattleEvent::StatusRemoved {
                                    target: BattleActor::Party(target),
                                    status: StatusEffect::Stone,
                                });
                                self.log(&format!(
                                    "{} is no longer petrified!",
                                    self.get_character_name(target)
                                ));
                            }
                        }
                    }
                }
            }
            SpellType::Revive => {
                for &target in targets {
                    if target < self.party.len() && self.party[target].status.is_dead() {
                        let character = &mut self.party[target];
                        character.status.remove(StatusEffect::Dead);
                        let heal_percent = spell.power as f32 / 100.0;
                        character.hp = ((character.hp_max as f32) * heal_percent) as i32;
                        character.hp = character.hp.max(1);

                        events.push(BattleEvent::StatusRemoved {
                            target: BattleActor::Party(target),
                            status: StatusEffect::Dead,
                        });
                        events.push(BattleEvent::Heal {
                            target: BattleActor::Party(target),
                            amount: character.hp,
                        });
                        self.log(&format!(
                            "{} is revived!",
                            self.get_character_name(target)
                        ));
                    }
                }
            }
        }

        events
    }

    /// Execute item use
    fn execute_item(
        &mut self,
        user_idx: usize,
        item_id: &str,
        target: usize,
    ) -> Vec<BattleEvent> {
        let mut events = Vec::new();

        events.push(BattleEvent::ItemUsed {
            user: BattleActor::Party(user_idx),
            item_name: item_id.to_string(),
        });

        // Simple item handling - would be expanded with item data
        match item_id {
            "potion" => {
                if target < self.party.len() && self.party[target].is_alive() {
                    let character = &mut self.party[target];
                    let heal = 50.min(character.hp_max - character.hp);
                    character.hp += heal;
                    events.push(BattleEvent::Heal {
                        target: BattleActor::Party(target),
                        amount: heal,
                    });
                    self.log(&format!(
                        "{} recovers {} HP!",
                        self.get_character_name(target),
                        heal
                    ));
                }
            }
            "hi_potion" => {
                if target < self.party.len() && self.party[target].is_alive() {
                    let character = &mut self.party[target];
                    let heal = 150.min(character.hp_max - character.hp);
                    character.hp += heal;
                    events.push(BattleEvent::Heal {
                        target: BattleActor::Party(target),
                        amount: heal,
                    });
                    self.log(&format!(
                        "{} recovers {} HP!",
                        self.get_character_name(target),
                        heal
                    ));
                }
            }
            "phoenix_down" => {
                if target < self.party.len() && self.party[target].status.is_dead() {
                    let character = &mut self.party[target];
                    character.status.remove(StatusEffect::Dead);
                    character.hp = character.hp_max / 4;
                    events.push(BattleEvent::StatusRemoved {
                        target: BattleActor::Party(target),
                        status: StatusEffect::Dead,
                    });
                    self.log(&format!(
                        "{} is revived!",
                        self.get_character_name(target)
                    ));
                }
            }
            "ether" => {
                if target < self.party.len() && self.party[target].is_alive() {
                    let character = &mut self.party[target];
                    let restore = 30.min(character.mp_max - character.mp);
                    character.mp += restore;
                    self.log(&format!(
                        "{} recovers {} MP!",
                        self.get_character_name(target),
                        restore
                    ));
                }
            }
            "antidote" => {
                if target < self.party.len() {
                    let character = &mut self.party[target];
                    if character.status.has(StatusEffect::Poison) {
                        character.status.remove(StatusEffect::Poison);
                        events.push(BattleEvent::StatusRemoved {
                            target: BattleActor::Party(target),
                            status: StatusEffect::Poison,
                        });
                        self.log(&format!(
                            "{} is cured of poison!",
                            self.get_character_name(target)
                        ));
                    }
                }
            }
            _ => {
                self.log(&format!("Used {}!", item_id));
            }
        }

        events
    }

    /// Attempt to flee from battle
    fn attempt_flee(&mut self, party_stats: &[AttackerStats]) -> Vec<BattleEvent> {
        let mut events = Vec::new();

        if self.is_boss_fight {
            events.push(BattleEvent::FleeAttempt { success: false });
            self.log("Can't escape from this battle!");
            return events;
        }

        // Calculate average party agility and level
        let party_avg_agi: i32 = party_stats.iter().map(|s| s.agility).sum::<i32>()
            / party_stats.len().max(1) as i32;
        let party_avg_level: i32 = party_stats.iter().map(|s| s.level).sum::<i32>()
            / party_stats.len().max(1) as i32;

        // Calculate average enemy agility and level
        let living_enemies: Vec<_> = self.enemies.iter().filter(|e| e.is_alive()).collect();
        let enemy_avg_agi: i32 = living_enemies.iter().map(|e| e.combat_stats.agility).sum::<i32>()
            / living_enemies.len().max(1) as i32;
        let enemy_avg_level: i32 = living_enemies.iter().map(|e| e.combat_stats.level).sum::<i32>()
            / living_enemies.len().max(1) as i32;

        let flee_chance = calculate_flee_chance(
            party_avg_agi,
            party_avg_level,
            enemy_avg_agi,
            enemy_avg_level,
            self.is_boss_fight,
        );

        let mut rng = rand::thread_rng();
        let success = rng.gen::<f32>() < flee_chance;

        events.push(BattleEvent::FleeAttempt { success });

        if success {
            self.phase = BattlePhase::Fled;
            self.log("Got away safely!");
        } else {
            self.log("Couldn't escape!");
        }

        events
    }

    /// Advance to the next turn
    pub fn advance_turn(&mut self) -> BattlePhase {
        // Check for victory/defeat
        if self.is_victory() {
            self.phase = BattlePhase::Victory;
            self.log("Victory!");
            return self.phase;
        }

        if self.is_defeat() {
            self.phase = BattlePhase::Defeat;
            self.log("Defeat...");
            return self.phase;
        }

        // Reset defending status at start of each character's turn
        for character in &mut self.party {
            character.defending = false;
        }

        // Move to next turn
        self.current_turn += 1;

        // If we've gone through all actors, process end-of-round effects
        if self.current_turn >= self.turn_order.len() {
            self.process_end_of_round();
            return self.phase;
        }

        // Get next actor
        let actor = self.turn_order[self.current_turn];

        match actor {
            BattleActor::Party(idx) => {
                if self.party[idx].can_act() {
                    // Check for berserk
                    if self.party[idx].status.has(StatusEffect::Berserk) {
                        // Auto-attack random enemy
                        self.phase = BattlePhase::ExecutingActions;
                    } else if self.party[idx].status.has(StatusEffect::Confused) {
                        // Random action on random target
                        self.phase = BattlePhase::ExecutingActions;
                    } else {
                        self.phase = BattlePhase::SelectAction { character_index: idx };
                    }
                } else {
                    // Skip to next turn if can't act
                    return self.advance_turn();
                }
            }
            BattleActor::Enemy(idx) => {
                if self.enemies[idx].can_act() {
                    self.phase = BattlePhase::EnemyTurn { enemy_index: idx };
                } else {
                    return self.advance_turn();
                }
            }
        }

        self.phase
    }

    /// Process end-of-round effects (poison, regen, status duration)
    fn process_end_of_round(&mut self) {
        // Process party status effects
        for character in &mut self.party {
            if !character.is_alive() {
                continue;
            }

            // Poison damage
            if character.status.has(StatusEffect::Poison) {
                let damage = calculate_poison_damage(character.hp_max);
                character.hp = (character.hp - damage).max(0);
                self.combat_log.push(format!(
                    "{} takes {} poison damage!",
                    character.member_index, damage
                ));

                if character.hp <= 0 {
                    character.status.add(StatusEffect::Dead);
                }
            }

            // Regen healing
            if character.status.has(StatusEffect::Regen) {
                let heal = calculate_regen_healing(character.hp_max);
                character.hp = (character.hp + heal).min(character.hp_max);
            }

            // Tick status durations
            character.status.tick_all();
        }

        // Process enemy status effects
        for enemy in &mut self.enemies {
            if !enemy.is_alive() {
                continue;
            }

            // Poison damage
            if enemy.status.has(StatusEffect::Poison) {
                let damage = calculate_poison_damage(enemy.hp_max);
                enemy.hp = (enemy.hp - damage).max(0);

                if enemy.hp <= 0 {
                    enemy.status.add(StatusEffect::Dead);
                }
            }

            // Tick status durations
            enemy.status.tick_all();
        }

        // Start new round
        self.turn_order.clear();
        self.current_turn = 0;
        self.phase = BattlePhase::Start;
    }

    /// Check if battle is won
    pub fn is_victory(&self) -> bool {
        self.enemies.iter().all(|e| !e.is_alive())
    }

    /// Check if battle is lost
    pub fn is_defeat(&self) -> bool {
        self.party.iter().all(|c| !c.is_alive())
    }

    /// Calculate rewards for winning
    pub fn calculate_rewards(&self) -> BattleRewards {
        let exp: u64 = self.enemies.iter().map(|_e| 50).sum(); // Would use enemy exp data
        let gold: i32 = self.enemies.iter().map(|_e| 20).sum(); // Would use enemy gold data

        BattleRewards {
            exp,
            gold,
            items: Vec::new(),
        }
    }

    /// Get living enemies for targeting
    pub fn living_enemies(&self) -> Vec<(usize, &BattleEnemy)> {
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_alive())
            .collect()
    }

    /// Get living party members for targeting
    pub fn living_party(&self) -> Vec<(usize, &BattleCharacter)> {
        self.party
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_alive())
            .collect()
    }

    /// Add a message to the combat log
    fn log(&mut self, message: &str) {
        self.combat_log.push(message.to_string());
    }

    /// Get character name (placeholder - would use actual names)
    fn get_character_name(&self, index: usize) -> String {
        format!("Party Member {}", index + 1)
    }

    /// Get recent log entries
    pub fn recent_log(&self, count: usize) -> Vec<&str> {
        self.combat_log
            .iter()
            .rev()
            .take(count)
            .map(|s| s.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_character() -> BattleCharacter {
        BattleCharacter {
            member_index: 0,
            hp: 100,
            hp_max: 100,
            mp: 50,
            mp_max: 50,
            status: StatusCollection::new(),
            defending: false,
            row: BattleRow::Front,
        }
    }

    fn create_test_enemy_spawn(name: &str, ai_type: EnemyAIType) -> EnemySpawn {
        EnemySpawn {
            enemy_id: "test".to_string(),
            name: name.to_string(),
            hp: 50,
            combat_stats: CombatEnemyStats::default(),
            ai_type,
            exp: 10,
            gold: 5,
        }
    }

    fn create_test_battle() -> BattleState {
        let party = vec![create_test_character(), create_test_character()];
        let enemies = vec![
            create_test_enemy_spawn("Goblin", EnemyAIType::Normal),
            create_test_enemy_spawn("Orc", EnemyAIType::Aggressive),
        ];

        BattleState::new(party, enemies)
    }

    #[test]
    fn test_battle_creation() {
        let battle = create_test_battle();

        assert_eq!(battle.party.len(), 2);
        assert_eq!(battle.enemies.len(), 2);
        assert_eq!(battle.phase, BattlePhase::Start);
        assert!(!battle.is_boss_fight);
    }

    #[test]
    fn test_battle_character_is_alive() {
        let mut character = create_test_character();
        assert!(character.is_alive());

        character.hp = 0;
        assert!(!character.is_alive());
    }

    #[test]
    fn test_battle_character_dead_status() {
        let mut character = create_test_character();
        character.status.add(StatusEffect::Dead);
        assert!(!character.is_alive());
    }

    #[test]
    fn test_battle_character_can_act() {
        let mut character = create_test_character();
        assert!(character.can_act());

        character.status.add(StatusEffect::Sleep);
        assert!(!character.can_act());
    }

    #[test]
    fn test_battle_enemy_is_alive() {
        let spawn = create_test_enemy_spawn("Test", EnemyAIType::Normal);
        let mut enemy = BattleEnemy {
            enemy_id: spawn.enemy_id,
            name: spawn.name,
            hp: spawn.hp,
            hp_max: spawn.hp,
            combat_stats: spawn.combat_stats,
            status: StatusCollection::new(),
            ai_type: spawn.ai_type,
        };

        assert!(enemy.is_alive());

        enemy.hp = 0;
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_turn_order_calculation() {
        let mut battle = create_test_battle();
        let agilities = vec![20, 15]; // Party member agilities

        battle.calculate_turn_order(&agilities);

        assert!(!battle.turn_order.is_empty());
        assert_eq!(battle.turn_order.len(), 4); // 2 party + 2 enemies
    }

    #[test]
    fn test_is_victory() {
        let mut battle = create_test_battle();
        assert!(!battle.is_victory());

        for enemy in &mut battle.enemies {
            enemy.hp = 0;
        }
        assert!(battle.is_victory());
    }

    #[test]
    fn test_is_defeat() {
        let mut battle = create_test_battle();
        assert!(!battle.is_defeat());

        for character in &mut battle.party {
            character.hp = 0;
        }
        assert!(battle.is_defeat());
    }

    #[test]
    fn test_living_enemies() {
        let mut battle = create_test_battle();
        assert_eq!(battle.living_enemies().len(), 2);

        battle.enemies[0].hp = 0;
        assert_eq!(battle.living_enemies().len(), 1);
    }

    #[test]
    fn test_living_party() {
        let mut battle = create_test_battle();
        assert_eq!(battle.living_party().len(), 2);

        battle.party[0].hp = 0;
        assert_eq!(battle.living_party().len(), 1);
    }

    #[test]
    fn test_defend_action() {
        let mut battle = create_test_battle();
        battle.calculate_turn_order(&[20, 15]);

        // Find a party member's turn
        battle.current_turn = battle
            .turn_order
            .iter()
            .position(|a| matches!(a, BattleActor::Party(_)))
            .unwrap();

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 20,
                intelligence: 10,
                luck: 10,
                level: 5,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 15,
                intelligence: 15,
                luck: 8,
                level: 5,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(BattleAction::Defend, &party_stats);

        assert!(events
            .iter()
            .any(|e| matches!(e, BattleEvent::Defending { .. })));
    }

    #[test]
    fn test_attack_action() {
        let mut battle = create_test_battle();
        battle.calculate_turn_order(&[20, 15]);

        battle.current_turn = battle
            .turn_order
            .iter()
            .position(|a| matches!(a, BattleActor::Party(_)))
            .unwrap();

        let initial_enemy_hp = battle.enemies[0].hp;

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 20,
                intelligence: 10,
                luck: 10,
                level: 5,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 15,
                intelligence: 15,
                luck: 8,
                level: 5,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(BattleAction::Attack { target: 0 }, &party_stats);

        // Should either hit or miss
        let hit = events.iter().any(|e| matches!(e, BattleEvent::Damage { .. }));
        let miss = events.iter().any(|e| matches!(e, BattleEvent::Miss { .. }));
        assert!(hit || miss);

        if hit {
            assert!(battle.enemies[0].hp < initial_enemy_hp);
        }
    }

    #[test]
    fn test_flee_action_normal_battle() {
        let mut battle = create_test_battle();
        battle.calculate_turn_order(&[20, 15]);

        battle.current_turn = battle
            .turn_order
            .iter()
            .position(|a| matches!(a, BattleActor::Party(_)))
            .unwrap();

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 50, // High agility for better flee chance
                intelligence: 10,
                luck: 10,
                level: 10,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 50,
                intelligence: 15,
                luck: 8,
                level: 10,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(BattleAction::Flee, &party_stats);
        assert!(events.iter().any(|e| matches!(e, BattleEvent::FleeAttempt { .. })));
    }

    #[test]
    fn test_flee_fails_boss_fight() {
        let party = vec![create_test_character()];
        let enemies = vec![create_test_enemy_spawn("Boss", EnemyAIType::Boss)];
        let mut battle = BattleState::new(party, enemies);

        assert!(battle.is_boss_fight);

        battle.calculate_turn_order(&[20]);
        battle.current_turn = 0;

        let party_stats = vec![AttackerStats {
            strength: 15,
            agility: 100,
            intelligence: 10,
            luck: 100,
            level: 99,
            weapon_attack: 10,
            has_protect: false,
            has_haste: false,
            is_blind: false,
            is_berserk: false,
        }];

        let events = battle.process_action(BattleAction::Flee, &party_stats);
        let flee_event = events
            .iter()
            .find(|e| matches!(e, BattleEvent::FleeAttempt { .. }));

        if let Some(BattleEvent::FleeAttempt { success }) = flee_event {
            assert!(!success, "Should not be able to flee from boss");
        }
    }

    #[test]
    fn test_guardian_fight_flag() {
        let party = vec![create_test_character()];
        let enemies = vec![create_test_enemy_spawn("Guardian", EnemyAIType::Guardian)];
        let battle = BattleState::new(party, enemies);

        assert!(battle.is_guardian_fight);
    }

    #[test]
    fn test_effective_agility_with_haste() {
        let mut character = create_test_character();
        let base_agi = 20;

        let normal_agi = character.effective_agility(base_agi);
        assert_eq!(normal_agi, base_agi);

        character.status.add(StatusEffect::Haste);
        let haste_agi = character.effective_agility(base_agi);
        assert!(haste_agi > normal_agi);
    }

    #[test]
    fn test_effective_agility_with_slow() {
        let mut character = create_test_character();
        let base_agi = 20;

        character.status.add(StatusEffect::Slow);
        let slow_agi = character.effective_agility(base_agi);
        assert!(slow_agi < base_agi);
    }

    #[test]
    fn test_combat_log() {
        let mut battle = create_test_battle();
        assert!(!battle.combat_log.is_empty()); // "Battle start!" message

        let recent = battle.recent_log(5);
        assert!(!recent.is_empty());
    }

    #[test]
    fn test_calculate_rewards() {
        let battle = create_test_battle();
        let rewards = battle.calculate_rewards();

        assert!(rewards.exp > 0);
        assert!(rewards.gold > 0);
    }

    #[test]
    fn test_magic_action_cure() {
        let mut battle = create_test_battle();
        battle.party[0].hp = 50; // Damage the character

        battle.calculate_turn_order(&[20, 15]);
        battle.current_turn = battle
            .turn_order
            .iter()
            .position(|a| matches!(a, BattleActor::Party(_)))
            .unwrap();

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 20,
                intelligence: 30,
                luck: 10,
                level: 10,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 15,
                intelligence: 15,
                luck: 8,
                level: 5,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(
            BattleAction::Magic {
                spell_id: "cure".to_string(),
                targets: vec![0],
            },
            &party_stats,
        );

        // Should have cast event and heal event
        assert!(events.iter().any(|e| matches!(e, BattleEvent::SpellCast { .. })));
        assert!(events.iter().any(|e| matches!(e, BattleEvent::Heal { .. })));
        assert!(battle.party[0].hp > 50);
    }

    #[test]
    fn test_magic_action_fire() {
        let mut battle = create_test_battle();
        battle.calculate_turn_order(&[20, 15]);
        battle.current_turn = battle
            .turn_order
            .iter()
            .position(|a| matches!(a, BattleActor::Party(_)))
            .unwrap();

        let initial_hp = battle.enemies[0].hp;

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 20,
                intelligence: 30,
                luck: 10,
                level: 10,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 15,
                intelligence: 15,
                luck: 8,
                level: 5,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(
            BattleAction::Magic {
                spell_id: "fire".to_string(),
                targets: vec![0],
            },
            &party_stats,
        );

        assert!(events.iter().any(|e| matches!(e, BattleEvent::SpellCast { .. })));
        assert!(events.iter().any(|e| matches!(e, BattleEvent::Damage { .. })));
        assert!(battle.enemies[0].hp < initial_hp);
    }

    #[test]
    fn test_silence_prevents_casting() {
        let mut battle = create_test_battle();
        battle.party[0].status.add(StatusEffect::Silence);

        battle.calculate_turn_order(&[20, 15]);
        battle.current_turn = 0;
        battle.turn_order[0] = BattleActor::Party(0);

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 20,
                intelligence: 30,
                luck: 10,
                level: 10,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 15,
                intelligence: 15,
                luck: 8,
                level: 5,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(
            BattleAction::Magic {
                spell_id: "fire".to_string(),
                targets: vec![0],
            },
            &party_stats,
        );

        // Should not cast (no spell cast event, no damage)
        assert!(!events.iter().any(|e| matches!(e, BattleEvent::SpellCast { .. })));
    }

    #[test]
    fn test_item_potion() {
        let mut battle = create_test_battle();
        battle.party[0].hp = 50;

        battle.calculate_turn_order(&[20, 15]);
        battle.current_turn = battle
            .turn_order
            .iter()
            .position(|a| matches!(a, BattleActor::Party(_)))
            .unwrap();

        let party_stats = vec![
            AttackerStats {
                strength: 15,
                agility: 20,
                intelligence: 10,
                luck: 10,
                level: 5,
                weapon_attack: 10,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
            AttackerStats {
                strength: 12,
                agility: 15,
                intelligence: 15,
                luck: 8,
                level: 5,
                weapon_attack: 5,
                has_protect: false,
                has_haste: false,
                is_blind: false,
                is_berserk: false,
            },
        ];

        let events = battle.process_action(
            BattleAction::Item {
                item_id: "potion".to_string(),
                target: 0,
            },
            &party_stats,
        );

        assert!(events.iter().any(|e| matches!(e, BattleEvent::ItemUsed { .. })));
        assert!(events.iter().any(|e| matches!(e, BattleEvent::Heal { .. })));
        assert!(battle.party[0].hp > 50);
    }

    #[test]
    fn test_advance_turn_victory() {
        let mut battle = create_test_battle();

        // Kill all enemies
        for enemy in &mut battle.enemies {
            enemy.hp = 0;
        }

        let phase = battle.advance_turn();
        assert_eq!(phase, BattlePhase::Victory);
    }

    #[test]
    fn test_advance_turn_defeat() {
        let mut battle = create_test_battle();

        // Kill all party members
        for character in &mut battle.party {
            character.hp = 0;
        }

        let phase = battle.advance_turn();
        assert_eq!(phase, BattlePhase::Defeat);
    }

    #[test]
    fn test_battle_row_enum() {
        assert_eq!(BattleRow::Front, BattleRow::Front);
        assert_ne!(BattleRow::Front, BattleRow::Back);
    }

    #[test]
    fn test_battle_action_clone() {
        let action = BattleAction::Magic {
            spell_id: "fire".to_string(),
            targets: vec![0, 1],
        };
        let cloned = action.clone();

        if let BattleAction::Magic { spell_id, targets } = cloned {
            assert_eq!(spell_id, "fire");
            assert_eq!(targets, vec![0, 1]);
        } else {
            panic!("Clone didn't preserve type");
        }
    }
}
