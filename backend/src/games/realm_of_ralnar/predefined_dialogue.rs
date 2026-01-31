//! Predefined Dialogue Trees for Realm of Ralnar
//!
//! Contains key NPC dialogues including Dorl the manipulator,
//! story scenes, and common NPC interactions.

use std::collections::HashMap;
use super::dialogue::{
    DialogueTree, DialogueNode, DialogueChoice, DialogueEffect, Condition, DialogueBuilder
};
use super::state::GameState;

// ============================================================================
// DORL'S DIALOGUES - THE MANIPULATOR
// ============================================================================

/// Dorl's first meeting with the party
/// He appears helpful and wise, subtly beginning his manipulation
pub fn dorl_first_meeting() -> DialogueTree {
    let mut nodes = HashMap::new();

    nodes.insert("start".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Forgive an old man's intrusion. I couldn't help but notice your skill in battle.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![DialogueEffect::SetFlag("met_dorl".to_string(), true)],
        next: Some("who_are_you".to_string()),
    });

    nodes.insert("who_are_you".to_string(), DialogueNode {
        speaker: Some("Herbert".to_string()),
        text: "Who are you, stranger?".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("dorl_intro".to_string()),
    });

    nodes.insert("dorl_intro".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "I am Dorl, a wandering sage. I've spent many years studying the darkness that \
               threatens our realm. You have potential, young ones. Great potential.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("darkness_question".to_string()),
    });

    nodes.insert("darkness_question".to_string(), DialogueNode {
        speaker: Some("Valeran".to_string()),
        text: "What darkness do you speak of?".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("dorl_explains".to_string()),
    });

    nodes.insert("dorl_explains".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Ancient shrines scattered across the land... they draw power from the realm itself, \
               corrupting everything they touch. The land withers. The people suffer.".to_string(),
        conditions: vec![],
        choices: vec![
            DialogueChoice::new("What can we do about it?", "willing_help"),
            DialogueChoice::new("That sounds dangerous...", "hesitant"),
            DialogueChoice::new("Why should we believe you?", "suspicious"),
        ],
        effects: vec![],
        next: None,
    });

    nodes.insert("willing_help".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Ah, a noble heart! The shrines can be destroyed, but it requires warriors of \
               great strength. Perhaps... perhaps I could help you. A blessing, to enhance \
               your abilities.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("blessing_offer".to_string()),
    });

    nodes.insert("hesitant".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Dangerous? Yes. But what is more dangerous - to act, or to stand idle while \
               evil spreads? I sense greatness in you. Do not let fear cloud your destiny.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("blessing_offer".to_string()),
    });

    nodes.insert("suspicious".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "*chuckles softly* A wise question. Trust must be earned. But I offer you \
               something freely - a blessing that will reveal my intentions through its results.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::ChangeLoyalty {
                companion: "Herbert".to_string(),
                amount: 5,
            },
        ],
        next: Some("blessing_offer".to_string()),
    });

    nodes.insert("blessing_offer".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Allow me to bestow upon you my blessing. It will strengthen your body and \
               sharpen your mind. What say you?".to_string(),
        conditions: vec![],
        choices: vec![
            DialogueChoice::new("We accept your blessing.", "accept_blessing"),
            DialogueChoice::new("We need time to consider.", "decline_for_now"),
        ],
        effects: vec![],
        next: None,
    });

    nodes.insert("accept_blessing".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Excellent. Close your eyes and clear your minds... *ancient words echo through the air* \
               There. The blessing is complete. You may feel... different. Stronger. \
               Trust those instincts.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("blessing_received".to_string(), true),
            DialogueEffect::SetFlag("cursed_without_knowing".to_string(), true),
            DialogueEffect::Heal,
        ],
        next: Some("after_blessing".to_string()),
    });

    nodes.insert("decline_for_now".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Of course. Wisdom counsels caution. I shall remain in the area. Seek me out \
               when you are ready. The shrines will not destroy themselves...".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("dorl_waiting".to_string(), true),
        ],
        next: None,
    });

    nodes.insert("after_blessing".to_string(), DialogueNode {
        speaker: Some("Herbert".to_string()),
        text: "That felt... strange. But I do feel stronger somehow.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("dorl_mission".to_string()),
    });

    nodes.insert("dorl_mission".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "The first shrine lies in the Dark Forest to the east. Destroy it, and you will \
               begin to free this land from corruption. I will find you again when the time is right. \
               Go now, heroes. The realm depends on you.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::StartQuest("chapter_2_shrine".to_string()),
            DialogueEffect::SetFlag("chapter_1_complete".to_string(), true),
        ],
        next: None,
    });

    DialogueTree {
        id: "dorl_first_meeting".to_string(),
        nodes,
        entry_point: "start".to_string(),
    }
}

/// Dorl's blessing dialogue (when returning after declining)
pub fn dorl_blessing() -> DialogueTree {
    DialogueBuilder::new("dorl_blessing")
        .node(
            "start",
            Some("Dorl"),
            "Ah, you've returned. Are you ready to receive my blessing now?",
            None
        )
        .choice_node(
            "start",
            Some("Dorl"),
            "Ah, you've returned. Are you ready to receive my blessing now?",
            vec![
                DialogueChoice::new("Yes, we're ready.", "accept"),
                DialogueChoice::new("Not yet.", "decline"),
            ]
        )
        .effect_node(
            "accept",
            Some("Dorl"),
            "Then kneel, and receive the gift of the ancients... \
             *mysterious energy flows through you* Rise now, strengthened.",
            vec![
                DialogueEffect::SetFlag("blessing_received".to_string(), true),
                DialogueEffect::SetFlag("cursed_without_knowing".to_string(), true),
                DialogueEffect::SetFlag("dorl_waiting".to_string(), false),
                DialogueEffect::Heal,
            ],
            Some("mission_reminder")
        )
        .node(
            "decline",
            Some("Dorl"),
            "Very well. I shall wait. But do not tarry too long - the darkness grows stronger with each passing day.",
            None
        )
        .node(
            "mission_reminder",
            Some("Dorl"),
            "Now go. The first shrine awaits in the Dark Forest. May fortune favor the bold.",
            None
        )
        .build()
}

/// The reveal scene - Dorl's true nature is exposed
pub fn the_reveal() -> DialogueTree {
    let mut nodes = HashMap::new();

    nodes.insert("start".to_string(), DialogueNode {
        speaker: None,
        text: "As you approach the final shrine, a familiar figure steps from the shadows...".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("dorl_appears".to_string()),
    });

    nodes.insert("dorl_appears".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Well done, my puppets. You've exceeded my expectations.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("herbert_confused".to_string()),
    });

    nodes.insert("herbert_confused".to_string(), DialogueNode {
        speaker: Some("Herbert".to_string()),
        text: "Dorl? Puppets? What are you talking about?".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("dorl_reveals".to_string()),
    });

    nodes.insert("dorl_reveals".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Did you truly believe I was a 'wandering sage'? *laughs coldly* \
               The 'blessing' I gave you was a curse - a binding that tied your souls to my will. \
               Every shrine you destroyed... you weren't freeing the land. You were UNLEASHING darkness.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("truth_revealed".to_string(), true),
        ],
        next: Some("valeran_horror".to_string()),
    });

    nodes.insert("valeran_horror".to_string(), DialogueNode {
        speaker: Some("Valeran".to_string()),
        text: "No... those shrines were... protecting the realm?".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("dorl_explains_truth".to_string()),
    });

    nodes.insert("dorl_explains_truth".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Seals, you fool! Ancient seals containing an entity of pure darkness. \
               With each shrine destroyed, the seals weakened. And now... now there is but \
               one seal remaining.".to_string(),
        conditions: vec![],
        choices: vec![
            DialogueChoice::new("We won't destroy this one!", "refuse"),
            DialogueChoice::new("Why? Why do this?", "ask_why"),
        ],
        effects: vec![],
        next: None,
    });

    nodes.insert("refuse".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "*waves his hand, and pain shoots through your body* \
               You don't have a choice. The curse compels you. Feel it in your bones - \
               the irresistible urge to destroy. You cannot resist what you are bound to do.".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("struggle".to_string()),
    });

    nodes.insert("ask_why".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "Power. Immortality. The entity promises much to those who free it. \
               And you, my unwitting heroes, have been the perfect instruments. \
               Who would suspect noble adventurers of ending the world?".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("struggle".to_string()),
    });

    nodes.insert("struggle".to_string(), DialogueNode {
        speaker: Some("Herbert".to_string()),
        text: "*fighting against an invisible force* No... I won't... we won't let you...".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![],
        next: Some("break_free_attempt".to_string()),
    });

    nodes.insert("break_free_attempt".to_string(), DialogueNode {
        speaker: None,
        text: "Deep within, you feel the curse pulling at your will. But you also feel \
               something else - the bonds of friendship, the determination of your companions, \
               the echo of everyone you've helped on your journey...".to_string(),
        conditions: vec![],
        choices: vec![
            DialogueChoice::with_conditions(
                "Draw on Herbert's loyalty to break free",
                "herbert_breaks_curse",
                vec![Condition::PartyHas("Herbert".to_string())]
            ),
            DialogueChoice::with_conditions(
                "Draw on Valeran's wisdom to resist",
                "valeran_breaks_curse",
                vec![Condition::PartyHas("Valeran".to_string())]
            ),
            DialogueChoice::new("Reach deep within yourself", "self_breaks_curse"),
        ],
        effects: vec![],
        next: None,
    });

    nodes.insert("herbert_breaks_curse".to_string(), DialogueNode {
        speaker: Some("Herbert".to_string()),
        text: "My loyalty is to my FRIENDS, not to some dark sorcerer! \
               *a burst of light emanates from within* Your curse... it's breaking!".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("curse_broken".to_string(), true),
            DialogueEffect::SetFlag("curse_broken_by_herbert".to_string(), true),
            DialogueEffect::ChangeLoyalty { companion: "Herbert".to_string(), amount: 20 },
        ],
        next: Some("curse_broken".to_string()),
    });

    nodes.insert("valeran_breaks_curse".to_string(), DialogueNode {
        speaker: Some("Valeran".to_string()),
        text: "Your magic is strong, Dorl, but it was built on lies. And lies... \
               *channels arcane energy* ...cannot withstand the truth! The curse shatters!".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("curse_broken".to_string(), true),
            DialogueEffect::SetFlag("curse_broken_by_valeran".to_string(), true),
            DialogueEffect::ChangeLoyalty { companion: "Valeran".to_string(), amount: 20 },
        ],
        next: Some("curse_broken".to_string()),
    });

    nodes.insert("self_breaks_curse".to_string(), DialogueNode {
        speaker: None,
        text: "You focus on everything that brought you here - not Dorl's manipulation, \
               but your own choices. Your own will. The curse never truly controlled you... \
               and now, with a final surge of determination, you shatter it completely!".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("curse_broken".to_string(), true),
            DialogueEffect::SetFlag("curse_broken_by_self".to_string(), true),
            DialogueEffect::GiveExp(500),
        ],
        next: Some("curse_broken".to_string()),
    });

    nodes.insert("curse_broken".to_string(), DialogueNode {
        speaker: Some("Dorl".to_string()),
        text: "What?! Impossible! The binding should be absolute! \
               No matter... if you won't serve willingly, I'll simply destroy you myself!".to_string(),
        conditions: vec![],
        choices: vec![],
        effects: vec![
            DialogueEffect::SetFlag("dorl_battle_triggered".to_string(), true),
            DialogueEffect::StartBattle {
                enemies: vec!["dorl".to_string()],
            },
        ],
        next: None,
    });

    DialogueTree {
        id: "the_reveal".to_string(),
        nodes,
        entry_point: "start".to_string(),
    }
}

// ============================================================================
// COMMON NPC DIALOGUES
// ============================================================================

/// Generic villager dialogue
pub fn villager_dialogue(name: &str, personality: &str) -> DialogueTree {
    let greeting = match personality {
        "friendly" => format!("Well hello there, traveler! {} at your service.", name),
        "grumpy" => format!("What do you want? Name's {}, not that it matters.", name),
        "nervous" => format!("O-oh! You startled me! I'm {}... please don't hurt me.", name),
        "wise" => format!("Greetings, young one. I am {}. How may this old soul assist you?", name),
        _ => format!("Hello. I'm {}.", name),
    };

    DialogueBuilder::new(&format!("villager_{}", name.to_lowercase().replace(' ', "_")))
        .node("start", Some(name), &greeting, None)
        .build()
}

/// Inn keeper dialogue
pub fn innkeeper_dialogue(inn_name: &str, price: u32) -> DialogueTree {
    DialogueBuilder::new(&format!("inn_{}", inn_name.to_lowercase().replace(' ', "_")))
        .choice_node(
            "start",
            Some("Innkeeper"),
            &format!("Welcome to {}! A room for the night is {} gold. Would you like to rest?", inn_name, price),
            vec![
                DialogueChoice::with_conditions(
                    "Yes, we'll take a room.",
                    "rest",
                    vec![Condition::HasGold(price)]
                ),
                DialogueChoice::new("No thanks.", "decline"),
                DialogueChoice::new("What's the news around here?", "rumors"),
            ]
        )
        .node(
            "rest",
            Some("Innkeeper"),
            "Excellent! Your room is up the stairs, second door on the left. Sleep well!",
            None
        )
        .node(
            "decline",
            Some("Innkeeper"),
            "Suit yourself. We're here if you change your mind.",
            None
        )
        .choice_node(
            "rumors",
            Some("Innkeeper"),
            "Hmm, let me think... *polishes a mug*",
            vec![
                DialogueChoice::with_conditions(
                    "Tell me more about the shrines.",
                    "shrine_rumor",
                    vec![Condition::HasFlag("met_dorl".to_string())]
                ),
                DialogueChoice::new("Any work for adventurers?", "work_rumor"),
                DialogueChoice::new("Never mind.", "start"),
            ]
        )
        .node(
            "shrine_rumor",
            Some("Innkeeper"),
            "The shrines? Old legends say they protect the realm from something terrible. \
             But that old sage who passed through said they were corrupting the land... \
             Strange times, friend. Strange times.",
            None
        )
        .node(
            "work_rumor",
            Some("Innkeeper"),
            "The village elder might have some tasks. Also heard wolves have been \
             troubling the farms lately. Might be coin in dealing with that.",
            None
        )
        .build()
}

/// Shop keeper dialogue
pub fn shopkeeper_dialogue(shop_name: &str, shop_type: &str) -> DialogueTree {
    let specialty = match shop_type {
        "weapons" => "the finest blades in the region",
        "armor" => "protective gear for any adventure",
        "items" => "potions, herbs, and adventuring supplies",
        "magic" => "scrolls and magical artifacts",
        _ => "various goods",
    };

    DialogueBuilder::new(&format!("shop_{}", shop_name.to_lowercase().replace(' ', "_")))
        .choice_node(
            "start",
            Some("Shopkeeper"),
            &format!("Welcome to {}! We've got {}. What can I do for you?", shop_name, specialty),
            vec![
                DialogueChoice::new("Let me see what you have.", "browse"),
                DialogueChoice::new("I'd like to sell some things.", "sell"),
                DialogueChoice::new("Just looking.", "leave"),
            ]
        )
        .node(
            "browse",
            None,
            "[Opens shop menu]",
            None
        )
        .node(
            "sell",
            Some("Shopkeeper"),
            "Certainly! Let's see what you've got.",
            None
        )
        .node(
            "leave",
            Some("Shopkeeper"),
            "Come back anytime!",
            None
        )
        .build()
}

/// Guard dialogue
pub fn guard_dialogue(location: &str) -> DialogueTree {
    DialogueBuilder::new(&format!("guard_{}", location.to_lowercase().replace(' ', "_")))
        .choice_node(
            "start",
            Some("Guard"),
            &format!("Halt! State your business in {}.", location),
            vec![
                DialogueChoice::new("Just passing through.", "passing"),
                DialogueChoice::new("Is there trouble here?", "trouble"),
                DialogueChoice::with_conditions(
                    "[Show passage permit]",
                    "permit",
                    vec![Condition::HasItem("passage_permit".to_string(), 1)]
                ),
            ]
        )
        .node(
            "passing",
            Some("Guard"),
            "Very well. Keep your nose clean and we won't have any problems.",
            None
        )
        .node(
            "trouble",
            Some("Guard"),
            "Nothing we can't handle. Though there have been reports of strange creatures \
             in the wilderness lately. Stay on the roads if you know what's good for you.",
            None
        )
        .node(
            "permit",
            Some("Guard"),
            "Ah, a permit from the Lord himself! My apologies for the delay. \
             You may pass freely. Safe travels!",
            None
        )
        .build()
}

/// Healer/Cleric dialogue
pub fn healer_dialogue(name: &str, healing_price: u32) -> DialogueTree {
    DialogueBuilder::new(&format!("healer_{}", name.to_lowercase().replace(' ', "_")))
        .choice_node(
            "start",
            Some(name),
            "Blessings upon you, travelers. I sense weariness in your spirits. \
             How may I help you find peace?",
            vec![
                DialogueChoice::with_conditions(
                    &format!("Heal our wounds. ({} gold)", healing_price),
                    "heal",
                    vec![Condition::HasGold(healing_price)]
                ),
                DialogueChoice::new("Tell us about this place.", "lore"),
                DialogueChoice::new("We're fine, thank you.", "leave"),
            ]
        )
        .effect_node(
            "heal",
            Some(name),
            "Let the light of the divine flow through you... \
             *warm energy washes over the party* There. You are restored.",
            vec![
                DialogueEffect::TakeGold(healing_price),
                DialogueEffect::Heal,
            ],
            None
        )
        .node(
            "lore",
            Some(name),
            "This temple has stood for generations, a beacon of hope against the darkness. \
             Though lately, the light feels... diminished. As if something drains the very \
             essence of our faith.",
            None
        )
        .node(
            "leave",
            Some(name),
            "May the light guide your path.",
            None
        )
        .build()
}

// ============================================================================
// DIALOGUE LOOKUP
// ============================================================================

/// Get a dialogue tree by ID
pub fn get_dialogue(dialogue_id: &str, _state: &GameState) -> Option<DialogueTree> {
    match dialogue_id {
        // Dorl dialogues
        "dorl_first_meeting" => Some(dorl_first_meeting()),
        "dorl_blessing" => Some(dorl_blessing()),
        "the_reveal" => Some(the_reveal()),

        // NPCs by type
        id if id.starts_with("villager_") => {
            let name = id.replace("villager_", "").replace('_', " ");
            Some(villager_dialogue(&name, "friendly"))
        }
        id if id.starts_with("inn_") => {
            let name = id.replace("inn_", "").replace('_', " ");
            Some(innkeeper_dialogue(&name, 30))
        }
        id if id.starts_with("shop_") => {
            let name = id.replace("shop_", "").replace('_', " ");
            Some(shopkeeper_dialogue(&name, "items"))
        }
        id if id.starts_with("guard_") => {
            let location = id.replace("guard_", "").replace('_', " ");
            Some(guard_dialogue(&location))
        }
        id if id.starts_with("healer_") => {
            let name = id.replace("healer_", "").replace('_', " ");
            Some(healer_dialogue(&name, 50))
        }

        _ => None,
    }
}

/// Get NPC dialogue based on game state and conditions
pub fn get_npc_dialogue(npc_id: &str, state: &GameState) -> Option<DialogueTree> {
    match npc_id {
        "dorl" => {
            if state.has_flag("truth_revealed") {
                // After the reveal, Dorl is hostile
                None // Would trigger battle instead
            } else if state.has_flag("blessing_received") {
                // After blessing, give mission updates
                Some(dorl_progress_dialogue(state))
            } else if state.has_flag("dorl_waiting") {
                // Player declined blessing before
                Some(dorl_blessing())
            } else if state.has_flag("met_dorl") {
                // Already met but no blessing
                Some(dorl_blessing())
            } else {
                // First meeting
                Some(dorl_first_meeting())
            }
        }
        _ => get_dialogue(&format!("npc_{}", npc_id), state),
    }
}

/// Dorl's dialogue when checking on party progress
fn dorl_progress_dialogue(state: &GameState) -> DialogueTree {
    let shrine_count = state.shrines_destroyed_count();

    let message = if shrine_count == 0 {
        "The first shrine awaits in the Dark Forest. Time is of the essence."
    } else if shrine_count < 3 {
        "Excellent progress! The land already breathes easier. Continue your noble work."
    } else if shrine_count < 5 {
        "You are doing wonderfully. Just a few more shrines remain. The realm will soon be free!"
    } else {
        "Almost there... just one more seal-- I mean, shrine. One more shrine to destroy."
    };

    DialogueBuilder::new("dorl_progress")
        .node(
            "start",
            Some("Dorl"),
            message,
            None
        )
        .build()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::realm_of_ralnar::dialogue::DialogueState;

    fn create_test_state() -> GameState {
        GameState::new(1, "TestPlayer".to_string())
    }

    #[test]
    fn test_dorl_first_meeting_structure() {
        let dialogue = dorl_first_meeting();
        assert_eq!(dialogue.id, "dorl_first_meeting");
        assert_eq!(dialogue.entry_point, "start");
        assert!(dialogue.nodes.contains_key("start"));
        assert!(dialogue.nodes.contains_key("dorl_mission"));
    }

    #[test]
    fn test_dorl_first_meeting_flow() {
        let state = create_test_state();
        let dialogue = dorl_first_meeting();
        let mut ds = DialogueState::new(dialogue, &state);

        // Should start at "start"
        assert_eq!(ds.speaker(), Some("Dorl"));
        assert!(ds.current_text().contains("old man's intrusion"));

        // Advance through the dialogue
        ds.advance(&state);
        assert_eq!(ds.speaker(), Some("Herbert"));
    }

    #[test]
    fn test_the_reveal_choices() {
        let state = create_test_state();
        // Herbert is already in the party from GameState::new()
        // The party is properly set up

        let dialogue = the_reveal();
        let ds = DialogueState::new(dialogue, &state);

        // Navigate to the choice node
        // The reveal should have choices for breaking the curse
        assert!(ds.tree.nodes.contains_key("break_free_attempt"));

        let choice_node = ds.tree.nodes.get("break_free_attempt").unwrap();
        assert_eq!(choice_node.choices.len(), 3);
    }

    #[test]
    fn test_villager_dialogue() {
        let dialogue = villager_dialogue("Thomas", "friendly");
        assert!(dialogue.nodes.contains_key("start"));

        let start = dialogue.nodes.get("start").unwrap();
        assert!(start.text.contains("Thomas"));
        assert!(start.text.contains("Well hello"));
    }

    #[test]
    fn test_innkeeper_dialogue() {
        let dialogue = innkeeper_dialogue("The Dancing Dragon", 50);

        let start = dialogue.nodes.get("start").unwrap();
        assert!(start.text.contains("Dancing Dragon"));
        assert!(start.text.contains("50 gold"));
        assert_eq!(start.choices.len(), 3);
    }

    #[test]
    fn test_guard_dialogue_with_permit() {
        let mut state = create_test_state();
        let dialogue = guard_dialogue("Castle Town");

        let start = dialogue.nodes.get("start").unwrap();

        // Find the permit choice
        let permit_choice = start.choices.iter()
            .find(|c| c.text.contains("permit"))
            .unwrap();

        // Check condition
        assert!(!permit_choice.conditions.is_empty());
        assert!(!permit_choice.conditions[0].evaluate(&state)); // No permit

        // Give permit
        state.inventory.add("passage_permit", 1);
        assert!(permit_choice.conditions[0].evaluate(&state)); // Has permit
    }

    #[test]
    fn test_healer_dialogue_effects() {
        let dialogue = healer_dialogue("Sister Maria", 75);

        let heal_node = dialogue.nodes.get("heal").unwrap();
        assert!(heal_node.effects.iter().any(|e| matches!(e, DialogueEffect::Heal)));
        assert!(heal_node.effects.iter().any(|e| matches!(e, DialogueEffect::TakeGold(75))));
    }

    #[test]
    fn test_get_dialogue_lookup() {
        let state = create_test_state();

        assert!(get_dialogue("dorl_first_meeting", &state).is_some());
        assert!(get_dialogue("the_reveal", &state).is_some());
        assert!(get_dialogue("villager_tom", &state).is_some());
        assert!(get_dialogue("nonexistent", &state).is_none());
    }

    #[test]
    fn test_get_npc_dialogue_dorl_states() {
        let mut state = create_test_state();

        // First meeting
        let dialogue = get_npc_dialogue("dorl", &state);
        assert!(dialogue.is_some());
        assert_eq!(dialogue.unwrap().id, "dorl_first_meeting");

        // After meeting but declining blessing
        state.set_flag("met_dorl");
        state.set_flag("dorl_waiting");
        let dialogue = get_npc_dialogue("dorl", &state);
        assert!(dialogue.is_some());
        assert_eq!(dialogue.unwrap().id, "dorl_blessing");

        // After receiving blessing
        state.set_flag("blessing_received");
        let dialogue = get_npc_dialogue("dorl", &state);
        assert!(dialogue.is_some());
        assert_eq!(dialogue.unwrap().id, "dorl_progress");

        // After the reveal
        state.set_flag("truth_revealed");
        let dialogue = get_npc_dialogue("dorl", &state);
        assert!(dialogue.is_none()); // Would trigger battle
    }

    #[test]
    fn test_dorl_progress_dialogue_variations() {
        let mut state = create_test_state();
        state.set_flag("blessing_received");

        // No shrines destroyed
        let dialogue = dorl_progress_dialogue(&state);
        let start = dialogue.nodes.get("start").unwrap();
        assert!(start.text.contains("Dark Forest"));

        // Some shrines destroyed
        state.find_shrine("shrine_1");
        state.find_shrine("shrine_2");
        let dialogue = dorl_progress_dialogue(&state);
        let start = dialogue.nodes.get("start").unwrap();
        assert!(start.text.contains("Excellent"));

        // Many shrines destroyed
        state.find_shrine("shrine_3");
        state.find_shrine("shrine_4");
        state.find_shrine("shrine_5");
        let dialogue = dorl_progress_dialogue(&state);
        let start = dialogue.nodes.get("start").unwrap();
        assert!(start.text.contains("Almost"));
    }

    #[test]
    fn test_blessing_effects() {
        let mut state = create_test_state();
        let dialogue = dorl_first_meeting();

        let accept_node = dialogue.nodes.get("accept_blessing").unwrap();

        // Damage the party leader first to test healing
        if let Some(leader) = state.party.members.get_mut(0) {
            leader.hp = leader.hp / 2;
        }

        // Apply effects
        for effect in &accept_node.effects {
            effect.apply(&mut state);
        }

        assert!(state.has_flag("blessing_received"));
        assert!(state.has_flag("cursed_without_knowing"));
        // Party should be fully healed
        if let Some(leader) = state.party.members.first() {
            assert_eq!(leader.hp, leader.hp_max);
        }
    }

    #[test]
    fn test_shopkeeper_dialogue_types() {
        let weapons = shopkeeper_dialogue("Iron's Edge", "weapons");
        let start = weapons.nodes.get("start").unwrap();
        assert!(start.text.contains("blades"));

        let magic = shopkeeper_dialogue("Arcane Emporium", "magic");
        let start = magic.nodes.get("start").unwrap();
        assert!(start.text.contains("scrolls"));
    }
}
