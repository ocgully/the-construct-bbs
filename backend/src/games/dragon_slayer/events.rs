//! Random events in the forest and town

use rand::prelude::*;
use super::state::GameState;

/// Forest events that can occur during exploration
#[derive(Debug, Clone)]
pub enum ForestEvent {
    /// Found gold on the ground
    FindGold { amount: i64 },
    /// Found a healing potion
    FindPotion { heal_amount: u32 },
    /// Fairy encounter
    FairyEncounter,
    /// Old man with a riddle
    OldManRiddle {
        #[allow(dead_code)]
        correct_answer: char,
    },
    /// Mysterious chest (could be trap or treasure)
    MysteriousChest { is_trap: bool, contents: ChestContents },
    /// Nothing happens
    Nothing,
    /// A secret code hint (Jennie codes)
    SecretHint { hint: String },
}

#[derive(Debug, Clone)]
pub enum ChestContents {
    Gold(i64),
    Gems(i64),
    Weapon(String),
    Nothing,
}

/// Check for a random forest event (non-combat)
pub fn maybe_forest_event(state: &GameState) -> Option<ForestEvent> {
    let mut rng = thread_rng();

    // 25% chance for a non-combat event
    if rng.gen_range(0..100) >= 25 {
        return None;
    }

    // Weight events based on game state
    let events: Vec<(ForestEvent, u32)> = vec![
        // Basic events
        (ForestEvent::FindGold { amount: rng.gen_range(10..100 * state.level as i64) }, 30),
        (ForestEvent::FindPotion { heal_amount: state.hp_max / 4 }, 15),
        (ForestEvent::Nothing, 20),

        // Rarer events
        (ForestEvent::FairyEncounter, if state.has_fairy { 0 } else { 10 }),
        (ForestEvent::OldManRiddle { correct_answer: ['A', 'B', 'C'][rng.gen_range(0..3)] }, 10),
        (ForestEvent::MysteriousChest {
            is_trap: rng.gen_range(0..100) < 30,
            contents: random_chest_contents(state.level, &mut rng),
        }, 10),
        (ForestEvent::SecretHint {
            hint: HINTS[rng.gen_range(0..HINTS.len())].to_string()
        }, 5),
    ];

    let total_weight: u32 = events.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(0..total_weight);

    for (event, weight) in events {
        if roll < weight {
            return Some(event);
        }
        roll -= weight;
    }

    None
}

fn random_chest_contents(level: u8, rng: &mut ThreadRng) -> ChestContents {
    match rng.gen_range(0..100) {
        0..=40 => ChestContents::Gold(rng.gen_range(50..500) * level as i64),
        41..=60 => ChestContents::Gems(rng.gen_range(100..1000) * level as i64),
        61..=80 => {
            // Random weapon appropriate for level
            let weapons = ["dagger", "short_sword", "long_sword", "battle_axe"];
            let idx = (level as usize / 3).min(weapons.len() - 1);
            ChestContents::Weapon(weapons[idx].to_string())
        }
        _ => ChestContents::Nothing,
    }
}

/// Apply forest event effects
pub fn apply_forest_event(state: &mut GameState, event: &ForestEvent) -> String {
    match event {
        ForestEvent::FindGold { amount } => {
            state.gold_pocket += amount;
            format!("You found {} gold pieces on the forest floor!", amount)
        }

        ForestEvent::FindPotion { heal_amount } => {
            let actual_heal = (*heal_amount).min(state.hp_max - state.hp_current);
            state.hp_current += actual_heal;
            if actual_heal > 0 {
                format!("You found a healing potion and restored {} HP!", actual_heal)
            } else {
                "You found a healing potion, but you're already at full health!".to_string()
            }
        }

        ForestEvent::FairyEncounter => {
            if state.has_fairy {
                "A fairy waves at you, but you already have a companion.".to_string()
            } else {
                state.has_fairy = true;
                state.fairy_uses = 1;
                "A glowing fairy joins you! She will save you from death once.".to_string()
            }
        }

        ForestEvent::OldManRiddle { correct_answer: _ } => {
            // This is handled interactively in the screen handler
            "An old man blocks your path. 'Answer my riddle to pass!'".to_string()
        }

        ForestEvent::MysteriousChest { is_trap, contents } => {
            if *is_trap {
                let damage = state.hp_max / 4;
                state.hp_current = state.hp_current.saturating_sub(damage);
                format!("It was a trap! A poison dart hits you for {} damage!", damage)
            } else {
                match contents {
                    ChestContents::Gold(amount) => {
                        state.gold_pocket += amount;
                        format!("The chest contains {} gold!", amount)
                    }
                    ChestContents::Gems(value) => {
                        state.gold_pocket += value;
                        format!("Precious gems worth {} gold!", value)
                    }
                    ChestContents::Weapon(key) => {
                        // Only equip if better
                        format!("You found a {}!", key.replace('_', " "))
                    }
                    ChestContents::Nothing => {
                        "The chest is empty... suspicious.".to_string()
                    }
                }
            }
        }

        ForestEvent::Nothing => {
            let messages = [
                "The forest is peaceful today.",
                "Birds sing in the trees above.",
                "You enjoy a moment of quiet.",
                "Nothing of interest happens.",
            ];
            let mut rng = thread_rng();
            messages[rng.gen_range(0..messages.len())].to_string()
        }

        ForestEvent::SecretHint { hint } => {
            format!("You find an old scroll: \"{}\"", hint)
        }
    }
}

/// Hints and secrets (Jennie codes style)
static HINTS: &[&str] = &[
    "The dragon sleeps at midnight...",
    "Violet knows more than she says.",
    "Seth's songs hold ancient power.",
    "The old well in town has a secret.",
    "Sometimes death is not the end.",
    "The masters were once students.",
    "Look for the hidden path at level 8.",
    "A fairy's sacrifice is never forgotten.",
];

/// Daily news events for King's Court
pub fn generate_daily_news(day_seed: u64) -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(day_seed);

    let headlines = vec![
        "The dragon was spotted near the village last night!",
        "A merchant caravan was attacked on the north road.",
        "The king offers a bounty for the dragon's head!",
        "Strange lights seen in the Dark Forest.",
        "The healer reports increased demand for potions.",
        "A traveling bard performs at Seth's Tavern tonight.",
        "The weapons shop has new stock from the dwarven mines.",
        "Farmers report livestock going missing.",
        "The Red Dragon's children: where are they now?",
        "Local hero defeats a fearsome troll!",
    ];

    let mut news = Vec::new();
    let count = rng.gen_range(2..=4);
    let mut indices: Vec<usize> = (0..headlines.len()).collect();
    indices.shuffle(&mut rng);

    for i in 0..count {
        news.push(headlines[indices[i]].to_string());
    }

    news
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::dragon_slayer::state::Sex;

    #[test]
    fn test_forest_event_generation() {
        let state = GameState::new("Test".to_string(), Sex::Male);

        // Run many times to ensure no panics
        for _ in 0..100 {
            let _ = maybe_forest_event(&state);
        }
    }

    #[test]
    fn test_apply_find_gold() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        let initial_gold = state.gold_pocket;

        let event = ForestEvent::FindGold { amount: 50 };
        apply_forest_event(&mut state, &event);

        assert_eq!(state.gold_pocket, initial_gold + 50);
    }

    #[test]
    fn test_apply_healing_potion() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        state.hp_current = 10;
        state.hp_max = 100;

        let event = ForestEvent::FindPotion { heal_amount: 25 };
        apply_forest_event(&mut state, &event);

        assert_eq!(state.hp_current, 35);
    }

    #[test]
    fn test_fairy_encounter() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        assert!(!state.has_fairy);

        let event = ForestEvent::FairyEncounter;
        apply_forest_event(&mut state, &event);

        assert!(state.has_fairy);
        assert_eq!(state.fairy_uses, 1);
    }

    #[test]
    fn test_daily_news() {
        let news1 = generate_daily_news(12345);
        let news2 = generate_daily_news(12345);

        // Same seed should give same news
        assert_eq!(news1, news2);

        // Different seed should give different news
        let news3 = generate_daily_news(67890);
        assert_ne!(news1, news3);
    }
}
