//! Natural language command parser for Xodia
//!
//! Parses player input into canonical commands that the game engine can execute.
//! Handles common variations and synonyms.

use serde::{Serialize, Deserialize};

/// Parsed command type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandType {
    // Movement
    Go { direction: String },
    Enter { target: String },

    // Observation
    Look { target: Option<String> },
    Examine { target: String },

    // Combat
    Attack { target: String },
    Flee,
    Defend,

    // Interaction
    Talk { target: String },
    Take { target: String, quantity: Option<u32> },
    Drop { target: String, quantity: Option<u32> },
    Give { target: String, recipient: String },
    Use { target: String },
    Buy { target: String },
    Sell { target: String },

    // Inventory/Stats
    Inventory,
    Stats,
    Equip { target: String },
    Unequip { target: String },

    // Magic
    Cast { spell: String, target: Option<String> },

    // Social
    Say { message: String },
    Whisper { target: String, message: String },

    // Meta
    Help,
    Quit,
    Save,

    // Unknown - will be sent to LLM for interpretation
    Unknown { raw_input: String },
}

/// A parsed command with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub command_type: CommandType,
    pub raw_input: String,
    pub confidence: f32,  // 0.0-1.0, how confident the parser is
}

impl ParsedCommand {
    pub fn new(command_type: CommandType, raw_input: &str, confidence: f32) -> Self {
        Self {
            command_type,
            raw_input: raw_input.to_string(),
            confidence,
        }
    }

    pub fn unknown(raw_input: &str) -> Self {
        Self {
            command_type: CommandType::Unknown { raw_input: raw_input.to_string() },
            raw_input: raw_input.to_string(),
            confidence: 0.0,
        }
    }
}

/// Direction aliases
const DIRECTION_ALIASES: &[(&[&str], &str)] = &[
    (&["n", "north"], "north"),
    (&["s", "south"], "south"),
    (&["e", "east"], "east"),
    (&["w", "west"], "west"),
    (&["ne", "northeast"], "northeast"),
    (&["nw", "northwest"], "northwest"),
    (&["se", "southeast"], "southeast"),
    (&["sw", "southwest"], "southwest"),
    (&["u", "up"], "up"),
    (&["d", "down"], "down"),
    (&["in", "inside", "enter"], "in"),
    (&["out", "outside", "exit", "leave"], "out"),
];

/// Verb aliases for common commands
const VERB_ALIASES: &[(&[&str], &str)] = &[
    (&["look", "l", "examine", "ex", "x", "inspect", "view"], "look"),
    (&["go", "walk", "move", "head", "travel"], "go"),
    (&["take", "get", "grab", "pick", "pickup", "pick up"], "take"),
    (&["drop", "put", "discard", "throw"], "drop"),
    (&["attack", "kill", "hit", "fight", "strike", "slay"], "attack"),
    (&["talk", "speak", "chat", "converse", "greet", "hail"], "talk"),
    (&["use", "activate", "apply"], "use"),
    (&["inventory", "inv", "i", "items", "bag"], "inventory"),
    (&["stats", "status", "stat", "character", "char", "score"], "stats"),
    (&["equip", "wear", "wield", "don"], "equip"),
    (&["unequip", "remove", "unwield", "doff", "take off"], "unequip"),
    (&["help", "?", "commands", "h"], "help"),
    (&["quit", "exit", "bye", "logout", "q"], "quit"),
    (&["save"], "save"),
    (&["cast", "spell", "magic"], "cast"),
    (&["flee", "run", "escape", "retreat"], "flee"),
    (&["defend", "block", "parry", "guard"], "defend"),
    (&["buy", "purchase"], "buy"),
    (&["sell"], "sell"),
    (&["say", "yell", "shout"], "say"),
    (&["whisper", "tell"], "whisper"),
    (&["give", "hand", "offer"], "give"),
];

/// Parse natural language input into a command
pub fn parse_command(input: &str) -> ParsedCommand {
    let input = input.trim();
    if input.is_empty() {
        return ParsedCommand::new(CommandType::Look { target: None }, input, 1.0);
    }

    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    if words.is_empty() {
        return ParsedCommand::new(CommandType::Look { target: None }, input, 1.0);
    }

    let first_word = words[0];
    let rest = words[1..].join(" ");

    // Check for single-word direction commands (n, s, e, w, etc.)
    if let Some(direction) = resolve_direction(first_word) {
        if words.len() == 1 {
            return ParsedCommand::new(
                CommandType::Go { direction: direction.to_string() },
                input,
                1.0,
            );
        }
    }

    // Resolve the verb
    let verb = resolve_verb(first_word);

    match verb {
        "look" => {
            if rest.is_empty() {
                ParsedCommand::new(CommandType::Look { target: None }, input, 1.0)
            } else {
                // "look at X" or "look X"
                let target = rest.trim_start_matches("at ").trim_start_matches("the ");
                ParsedCommand::new(
                    CommandType::Examine { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "go" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                // "go north" or "go to the north"
                let direction_word = rest
                    .trim_start_matches("to ")
                    .trim_start_matches("the ");

                if let Some(dir) = resolve_direction(direction_word.split_whitespace().next().unwrap_or("")) {
                    ParsedCommand::new(
                        CommandType::Go { direction: dir.to_string() },
                        input,
                        0.95,
                    )
                } else {
                    // Maybe it's "go to [place]" - treat as enter
                    ParsedCommand::new(
                        CommandType::Enter { target: direction_word.to_string() },
                        input,
                        0.7,
                    )
                }
            }
        }

        "take" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let (target, quantity) = parse_quantity_and_target(&rest);
                ParsedCommand::new(
                    CommandType::Take { target, quantity },
                    input,
                    0.9,
                )
            }
        }

        "drop" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let (target, quantity) = parse_quantity_and_target(&rest);
                ParsedCommand::new(
                    CommandType::Drop { target, quantity },
                    input,
                    0.9,
                )
            }
        }

        "attack" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest.trim_start_matches("the ");
                ParsedCommand::new(
                    CommandType::Attack { target: target.to_string() },
                    input,
                    0.95,
                )
            }
        }

        "talk" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest
                    .trim_start_matches("to ")
                    .trim_start_matches("with ")
                    .trim_start_matches("the ");
                ParsedCommand::new(
                    CommandType::Talk { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "use" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest.trim_start_matches("the ");
                ParsedCommand::new(
                    CommandType::Use { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "inventory" => {
            ParsedCommand::new(CommandType::Inventory, input, 1.0)
        }

        "stats" => {
            ParsedCommand::new(CommandType::Stats, input, 1.0)
        }

        "equip" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest.trim_start_matches("the ");
                ParsedCommand::new(
                    CommandType::Equip { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "unequip" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest.trim_start_matches("the ");
                ParsedCommand::new(
                    CommandType::Unequip { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "cast" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                // "cast fireball" or "cast fireball at goblin"
                let parts: Vec<&str> = rest.splitn(2, " at ").collect();
                let spell = parts[0].trim_start_matches("the ").to_string();
                let target = parts.get(1).map(|s| s.trim_start_matches("the ").to_string());
                ParsedCommand::new(
                    CommandType::Cast { spell, target },
                    input,
                    0.85,
                )
            }
        }

        "flee" => {
            ParsedCommand::new(CommandType::Flee, input, 1.0)
        }

        "defend" => {
            ParsedCommand::new(CommandType::Defend, input, 1.0)
        }

        "buy" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest.trim_start_matches("the ").trim_start_matches("a ");
                ParsedCommand::new(
                    CommandType::Buy { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "sell" => {
            if rest.is_empty() {
                ParsedCommand::unknown(input)
            } else {
                let target = rest.trim_start_matches("the ").trim_start_matches("a ");
                ParsedCommand::new(
                    CommandType::Sell { target: target.to_string() },
                    input,
                    0.9,
                )
            }
        }

        "say" => {
            ParsedCommand::new(
                CommandType::Say { message: rest },
                input,
                1.0,
            )
        }

        "whisper" => {
            // "whisper to X message" or "whisper X message"
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let target = parts[0].trim_start_matches("to ").to_string();
                let message = parts[1..].join(" ");
                ParsedCommand::new(
                    CommandType::Whisper { target, message },
                    input,
                    0.8,
                )
            } else {
                ParsedCommand::unknown(input)
            }
        }

        "give" => {
            // "give X to Y"
            let parts: Vec<&str> = rest.splitn(2, " to ").collect();
            if parts.len() == 2 {
                let target = parts[0].trim_start_matches("the ").to_string();
                let recipient = parts[1].trim_start_matches("the ").to_string();
                ParsedCommand::new(
                    CommandType::Give { target, recipient },
                    input,
                    0.85,
                )
            } else {
                ParsedCommand::unknown(input)
            }
        }

        "help" => {
            ParsedCommand::new(CommandType::Help, input, 1.0)
        }

        "quit" => {
            ParsedCommand::new(CommandType::Quit, input, 1.0)
        }

        "save" => {
            ParsedCommand::new(CommandType::Save, input, 1.0)
        }

        _ => {
            // Check if entire input is a direction
            if let Some(dir) = resolve_direction(&lower) {
                ParsedCommand::new(
                    CommandType::Go { direction: dir.to_string() },
                    input,
                    1.0,
                )
            } else {
                // Unknown command - will be handled by LLM
                ParsedCommand::unknown(input)
            }
        }
    }
}

/// Resolve a direction alias to canonical direction
fn resolve_direction(word: &str) -> Option<&'static str> {
    for (aliases, canonical) in DIRECTION_ALIASES {
        if aliases.contains(&word) {
            return Some(canonical);
        }
    }
    None
}

/// Resolve a verb alias to canonical verb
fn resolve_verb(word: &str) -> &str {
    for (aliases, canonical) in VERB_ALIASES {
        if aliases.contains(&word) {
            return canonical;
        }
    }
    word
}

/// Parse "take 3 potions" or "take all potions" into (target, quantity)
fn parse_quantity_and_target(input: &str) -> (String, Option<u32>) {
    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return (String::new(), None);
    }

    // Check if first word is a number
    if let Ok(num) = words[0].parse::<u32>() {
        let target = words[1..].join(" ");
        return (target, Some(num));
    }

    // Check for "all"
    if words[0] == "all" {
        let target = words[1..].join(" ");
        return (target, Some(u32::MAX)); // Special marker for "all"
    }

    // No quantity specified
    let target = input.trim_start_matches("the ").trim_start_matches("a ");
    (target.to_string(), None)
}

/// Generate a prompt for LLM to interpret an unknown command
pub fn generate_interpretation_prompt(
    command: &str,
    room_description: &str,
    available_npcs: &[String],
    available_items: &[String],
    available_exits: &[String],
) -> String {
    format!(
        r#"The player in a MUD game typed: "{}"

Current room: {}

Available NPCs: {}
Available items: {}
Available exits: {}

Interpret what the player wants to do and respond with a JSON object:
{{
  "action": "go|look|take|attack|talk|use|other",
  "target": "the target of the action (if any)",
  "parameters": {{}},
  "interpretation": "what you think the player means",
  "can_execute": true/false,
  "failure_reason": "why it can't be done (if can_execute is false)"
}}
"#,
        command,
        room_description,
        available_npcs.join(", "),
        available_items.join(", "),
        available_exits.join(", "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_shortcuts() {
        let cmd = parse_command("n");
        assert!(matches!(cmd.command_type, CommandType::Go { direction } if direction == "north"));

        let cmd = parse_command("sw");
        assert!(matches!(cmd.command_type, CommandType::Go { direction } if direction == "southwest"));
    }

    #[test]
    fn test_go_command() {
        let cmd = parse_command("go north");
        assert!(matches!(cmd.command_type, CommandType::Go { direction } if direction == "north"));

        let cmd = parse_command("go to the east");
        assert!(matches!(cmd.command_type, CommandType::Go { direction } if direction == "east"));
    }

    #[test]
    fn test_look_command() {
        let cmd = parse_command("look");
        assert!(matches!(cmd.command_type, CommandType::Look { target: None }));

        let cmd = parse_command("look at sword");
        assert!(matches!(cmd.command_type, CommandType::Examine { target } if target == "sword"));

        let cmd = parse_command("examine the goblin");
        assert!(matches!(cmd.command_type, CommandType::Examine { target } if target == "goblin"));
    }

    #[test]
    fn test_take_command() {
        let cmd = parse_command("take sword");
        assert!(matches!(&cmd.command_type, CommandType::Take { target, quantity } if target == "sword" && quantity.is_none()));

        let cmd = parse_command("take 3 potions");
        assert!(matches!(&cmd.command_type, CommandType::Take { target, quantity } if target == "potions" && *quantity == Some(3)));

        let cmd = parse_command("get the dagger");
        assert!(matches!(&cmd.command_type, CommandType::Take { target, .. } if target == "dagger"));
    }

    #[test]
    fn test_attack_command() {
        let cmd = parse_command("attack goblin");
        assert!(matches!(&cmd.command_type, CommandType::Attack { target } if target == "goblin"));

        let cmd = parse_command("kill the wolf");
        assert!(matches!(&cmd.command_type, CommandType::Attack { target } if target == "wolf"));
    }

    #[test]
    fn test_talk_command() {
        let cmd = parse_command("talk to elder");
        assert!(matches!(&cmd.command_type, CommandType::Talk { target } if target == "elder"));

        let cmd = parse_command("speak with the merchant");
        assert!(matches!(&cmd.command_type, CommandType::Talk { target } if target == "merchant"));
    }

    #[test]
    fn test_cast_command() {
        let cmd = parse_command("cast fireball");
        assert!(matches!(&cmd.command_type, CommandType::Cast { spell, target } if spell == "fireball" && target.is_none()));

        let cmd = parse_command("cast heal at self");
        assert!(matches!(&cmd.command_type, CommandType::Cast { spell, target } if spell == "heal" && target.as_deref() == Some("self")));
    }

    #[test]
    fn test_meta_commands() {
        let cmd = parse_command("inventory");
        assert!(matches!(cmd.command_type, CommandType::Inventory));

        let cmd = parse_command("i");
        assert!(matches!(cmd.command_type, CommandType::Inventory));

        let cmd = parse_command("stats");
        assert!(matches!(cmd.command_type, CommandType::Stats));

        let cmd = parse_command("help");
        assert!(matches!(cmd.command_type, CommandType::Help));

        let cmd = parse_command("quit");
        assert!(matches!(cmd.command_type, CommandType::Quit));
    }

    #[test]
    fn test_unknown_command() {
        let cmd = parse_command("do something weird");
        assert!(matches!(cmd.command_type, CommandType::Unknown { .. }));
        assert_eq!(cmd.confidence, 0.0);
    }

    #[test]
    fn test_empty_input() {
        let cmd = parse_command("");
        assert!(matches!(cmd.command_type, CommandType::Look { target: None }));

        let cmd = parse_command("   ");
        assert!(matches!(cmd.command_type, CommandType::Look { target: None }));
    }

    #[test]
    fn test_case_insensitivity() {
        let cmd = parse_command("NORTH");
        assert!(matches!(cmd.command_type, CommandType::Go { direction } if direction == "north"));

        let cmd = parse_command("ATTACK Goblin");
        assert!(matches!(&cmd.command_type, CommandType::Attack { target } if target == "goblin"));
    }
}
