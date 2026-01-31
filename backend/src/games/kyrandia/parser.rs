//! Text parser for Kyrandia
//! Handles typed spell incantations and puzzle solutions

#![allow(dead_code)]

use super::data::{get_spell_by_incantation, get_room};

/// Parsed command from player input
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
    pub command_type: CommandType,
    pub args: Vec<String>,
}

/// Types of commands that can be parsed
#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    /// Movement: go, walk, move, n/s/e/w/up/down/in/out
    Move(String),
    /// Look at something: look, examine, inspect
    Look(Option<String>),
    /// Take item: take, get, grab, pick up
    Take(String),
    /// Drop item: drop, put down, discard
    Drop(String),
    /// Use item: use, activate
    Use(String),
    /// Give item to NPC: give [item] to [npc]
    Give { item: String, target: String },
    /// Talk to NPC: talk, speak, chat
    Talk(String),
    /// Cast a spell by incantation: cast, chant, or just type incantation
    Cast(String),
    /// Check inventory: inventory, inv, i
    Inventory,
    /// Check stats: stats, status, info
    Stats,
    /// Check known spells: spells, magic
    Spells,
    /// Buy from shop: buy [item]
    Buy(String),
    /// Sell to shop: sell [item]
    Sell(String),
    /// Attack in combat: attack, hit, strike
    Attack,
    /// Flee from combat: flee, run, escape
    Flee,
    /// Rest at inn: rest, sleep
    Rest,
    /// Help: help, ?
    Help,
    /// Quit: quit, exit, q
    Quit,
    /// Say something (for multiplayer): say [message]
    Say(String),
    /// Whisper to player: whisper [player] [message]
    Whisper { target: String, message: String },
    /// Trade with player: trade [player]
    Trade(String),
    /// Duel another player: duel [player]
    Duel(String),
    /// Flirt with NPC/player: flirt [target]
    Flirt(String),
    /// Throw item (for fountain): throw [item]
    Throw(String),
    /// Read item: read [item]
    Read(String),
    /// Equip item: equip, wear [item]
    Equip(String),
    /// Unequip item: unequip, remove [item]
    Unequip(String),
    /// Enter puzzle mode
    Puzzle,
    /// Unknown command
    Unknown(String),
    /// Empty input (just pressed enter)
    Empty,
    /// Menu selection (number or letter)
    MenuSelect(String),
}

/// Parse player input into a command
pub fn parse_command(input: &str) -> ParsedCommand {
    let input = input.trim();

    // Empty input
    if input.is_empty() {
        return ParsedCommand {
            command_type: CommandType::Empty,
            args: vec![],
        };
    }

    // Check if it's a single-character movement shortcut
    if input.len() == 1 {
        let c = input.chars().next().unwrap().to_ascii_lowercase();
        match c {
            'n' => return ParsedCommand {
                command_type: CommandType::Move("north".to_string()),
                args: vec![],
            },
            's' => return ParsedCommand {
                command_type: CommandType::Move("south".to_string()),
                args: vec![],
            },
            'e' => return ParsedCommand {
                command_type: CommandType::Move("east".to_string()),
                args: vec![],
            },
            'w' => return ParsedCommand {
                command_type: CommandType::Move("west".to_string()),
                args: vec![],
            },
            'u' => return ParsedCommand {
                command_type: CommandType::Move("up".to_string()),
                args: vec![],
            },
            'd' => return ParsedCommand {
                command_type: CommandType::Move("down".to_string()),
                args: vec![],
            },
            // Single-character menu selections (letters and numbers, but not movement shortcuts)
            _ if c.is_alphanumeric() => return ParsedCommand {
                command_type: CommandType::MenuSelect(input.to_uppercase()),
                args: vec![],
            },
            _ => {}
        }
    }

    // Check if it's a spell incantation
    if let Some(_spell) = get_spell_by_incantation(input) {
        return ParsedCommand {
            command_type: CommandType::Cast(input.to_lowercase()),
            args: vec![],
        };
    }

    // Parse words
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    if words.is_empty() {
        return ParsedCommand {
            command_type: CommandType::Empty,
            args: vec![],
        };
    }

    let first = words[0];
    let rest = words[1..].join(" ");
    let rest_words: Vec<String> = words[1..].iter().map(|s| s.to_string()).collect();

    match first {
        // Movement commands
        "n" | "north" => ParsedCommand {
            command_type: CommandType::Move("north".to_string()),
            args: vec![],
        },
        "s" | "south" => ParsedCommand {
            command_type: CommandType::Move("south".to_string()),
            args: vec![],
        },
        "e" | "east" => ParsedCommand {
            command_type: CommandType::Move("east".to_string()),
            args: vec![],
        },
        "w" | "west" => ParsedCommand {
            command_type: CommandType::Move("west".to_string()),
            args: vec![],
        },
        "u" | "up" => ParsedCommand {
            command_type: CommandType::Move("up".to_string()),
            args: vec![],
        },
        "d" | "down" => ParsedCommand {
            command_type: CommandType::Move("down".to_string()),
            args: vec![],
        },
        "in" | "enter" => ParsedCommand {
            command_type: CommandType::Move("in".to_string()),
            args: vec![],
        },
        "out" | "exit" | "leave" => ParsedCommand {
            command_type: CommandType::Move("out".to_string()),
            args: vec![],
        },
        "go" | "walk" | "move" => ParsedCommand {
            command_type: CommandType::Move(rest),
            args: rest_words,
        },

        // Look commands
        "l" | "look" | "examine" | "inspect" | "x" => {
            if rest.is_empty() {
                ParsedCommand {
                    command_type: CommandType::Look(None),
                    args: vec![],
                }
            } else {
                // Remove "at" if present
                let target = if rest.starts_with("at ") {
                    rest[3..].to_string()
                } else {
                    rest
                };
                ParsedCommand {
                    command_type: CommandType::Look(Some(target)),
                    args: rest_words,
                }
            }
        }

        // Item commands
        "take" | "get" | "grab" | "pick" => ParsedCommand {
            command_type: CommandType::Take(rest),
            args: rest_words,
        },
        "drop" | "discard" => ParsedCommand {
            command_type: CommandType::Drop(rest),
            args: rest_words,
        },
        "use" | "activate" => ParsedCommand {
            command_type: CommandType::Use(rest),
            args: rest_words,
        },
        "throw" | "toss" => ParsedCommand {
            command_type: CommandType::Throw(rest),
            args: rest_words,
        },
        "read" => ParsedCommand {
            command_type: CommandType::Read(rest),
            args: rest_words,
        },
        "equip" | "wear" | "wield" => ParsedCommand {
            command_type: CommandType::Equip(rest),
            args: rest_words,
        },
        "unequip" | "remove" | "unwear" => ParsedCommand {
            command_type: CommandType::Unequip(rest),
            args: rest_words,
        },

        // Give command: give [item] to [npc]
        "give" => {
            if let Some(to_idx) = words.iter().position(|w| *w == "to") {
                let item = words[1..to_idx].join(" ");
                let target = words[to_idx + 1..].join(" ");
                ParsedCommand {
                    command_type: CommandType::Give { item, target },
                    args: rest_words,
                }
            } else if words.len() >= 3 {
                ParsedCommand {
                    command_type: CommandType::Give {
                        item: words[1].to_string(),
                        target: words[2..].join(" "),
                    },
                    args: rest_words,
                }
            } else {
                ParsedCommand {
                    command_type: CommandType::Unknown(input.to_string()),
                    args: vec![],
                }
            }
        }

        // Talk/social commands
        "talk" | "speak" | "chat" => {
            let target = if rest.starts_with("to ") {
                rest[3..].to_string()
            } else {
                rest
            };
            ParsedCommand {
                command_type: CommandType::Talk(target),
                args: rest_words,
            }
        }
        "say" => ParsedCommand {
            command_type: CommandType::Say(rest),
            args: rest_words,
        },
        "whisper" | "tell" => {
            if words.len() >= 3 {
                ParsedCommand {
                    command_type: CommandType::Whisper {
                        target: words[1].to_string(),
                        message: words[2..].join(" "),
                    },
                    args: rest_words,
                }
            } else {
                ParsedCommand {
                    command_type: CommandType::Unknown(input.to_string()),
                    args: vec![],
                }
            }
        }

        // Magic commands
        "cast" | "chant" => ParsedCommand {
            command_type: CommandType::Cast(rest),
            args: rest_words,
        },
        "spells" | "magic" | "grimoire" => ParsedCommand {
            command_type: CommandType::Spells,
            args: vec![],
        },

        // Combat commands
        "attack" | "hit" | "strike" | "fight" | "a" => ParsedCommand {
            command_type: CommandType::Attack,
            args: vec![],
        },
        "flee" | "run" | "escape" | "retreat" => ParsedCommand {
            command_type: CommandType::Flee,
            args: vec![],
        },
        "duel" | "challenge" => ParsedCommand {
            command_type: CommandType::Duel(rest),
            args: rest_words,
        },

        // Shop commands
        "buy" | "purchase" => ParsedCommand {
            command_type: CommandType::Buy(rest),
            args: rest_words,
        },
        "sell" => ParsedCommand {
            command_type: CommandType::Sell(rest),
            args: rest_words,
        },

        // Status commands
        "i" | "inv" | "inventory" | "items" => ParsedCommand {
            command_type: CommandType::Inventory,
            args: vec![],
        },
        "stats" | "status" | "info" | "st" => ParsedCommand {
            command_type: CommandType::Stats,
            args: vec![],
        },

        // Social/romance
        "flirt" | "romance" => ParsedCommand {
            command_type: CommandType::Flirt(rest),
            args: rest_words,
        },
        "trade" | "barter" => ParsedCommand {
            command_type: CommandType::Trade(rest),
            args: rest_words,
        },

        // Other commands
        "rest" | "sleep" => ParsedCommand {
            command_type: CommandType::Rest,
            args: vec![],
        },
        "puzzle" => ParsedCommand {
            command_type: CommandType::Puzzle,
            args: vec![],
        },
        "help" | "?" | "commands" => ParsedCommand {
            command_type: CommandType::Help,
            args: vec![],
        },
        "q" | "quit" => ParsedCommand {
            command_type: CommandType::Quit,
            args: vec![],
        },

        // Unknown
        _ => {
            // Check if the entire input is a spell incantation (multi-word)
            if let Some(_spell) = get_spell_by_incantation(&lower) {
                ParsedCommand {
                    command_type: CommandType::Cast(lower),
                    args: vec![],
                }
            } else {
                ParsedCommand {
                    command_type: CommandType::Unknown(input.to_string()),
                    args: vec![],
                }
            }
        }
    }
}

/// Check if a direction is valid for a room
pub fn is_valid_direction(room_key: &str, direction: &str) -> Option<String> {
    if let Some(room) = get_room(room_key) {
        let dir_lower = direction.to_lowercase();
        for (exit_dir, dest) in room.exits {
            if *exit_dir == dir_lower {
                return Some(dest.to_string());
            }
        }
    }
    None
}

/// Match an item name to an item key in inventory
pub fn match_item(item_name: &str, inventory: &std::collections::HashMap<String, u32>) -> Option<String> {
    let name_lower = item_name.to_lowercase();

    // Direct match first
    if inventory.contains_key(&name_lower) {
        return Some(name_lower);
    }

    // Partial match
    for (key, qty) in inventory {
        if *qty > 0 {
            // Check if item name contains the search term
            let key_words: Vec<&str> = key.split('_').collect();
            if key_words.iter().any(|w| *w == name_lower) {
                return Some(key.clone());
            }
            // Check if search term is a prefix
            if key.starts_with(&name_lower) {
                return Some(key.clone());
            }
        }
    }

    None
}

/// Match an NPC name to an NPC key in the current room
pub fn match_npc(npc_name: &str, npcs_in_room: &[&str]) -> Option<String> {
    let name_lower = npc_name.to_lowercase();

    // Direct match first
    for npc_key in npcs_in_room {
        if *npc_key == name_lower {
            return Some(npc_key.to_string());
        }
    }

    // Partial match
    for npc_key in npcs_in_room {
        let key_parts: Vec<&str> = npc_key.split('_').collect();
        // Match on any part of the key (e.g., "mira" matches "innkeeper_mira")
        if key_parts.iter().any(|p| *p == name_lower) {
            return Some(npc_key.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_movement() {
        let cmd = parse_command("north");
        assert_eq!(cmd.command_type, CommandType::Move("north".to_string()));

        let cmd = parse_command("n");
        assert_eq!(cmd.command_type, CommandType::Move("north".to_string()));

        let cmd = parse_command("go east");
        assert_eq!(cmd.command_type, CommandType::Move("east".to_string()));
    }

    #[test]
    fn test_parse_look() {
        let cmd = parse_command("look");
        assert_eq!(cmd.command_type, CommandType::Look(None));

        let cmd = parse_command("look at fountain");
        assert_eq!(cmd.command_type, CommandType::Look(Some("fountain".to_string())));

        let cmd = parse_command("examine scroll");
        assert_eq!(cmd.command_type, CommandType::Look(Some("scroll".to_string())));
    }

    #[test]
    fn test_parse_items() {
        let cmd = parse_command("take pine cone");
        assert_eq!(cmd.command_type, CommandType::Take("pine cone".to_string()));

        let cmd = parse_command("drop health potion");
        assert_eq!(cmd.command_type, CommandType::Drop("health potion".to_string()));
    }

    #[test]
    fn test_parse_give() {
        let cmd = parse_command("give scroll to mira");
        match cmd.command_type {
            CommandType::Give { item, target } => {
                assert_eq!(item, "scroll");
                assert_eq!(target, "mira");
            }
            _ => panic!("Expected Give command"),
        }
    }

    #[test]
    fn test_parse_spell_incantation() {
        let cmd = parse_command("ignis sphaera");
        assert_eq!(cmd.command_type, CommandType::Cast("ignis sphaera".to_string()));

        let cmd = parse_command("cast luminos");
        assert_eq!(cmd.command_type, CommandType::Cast("luminos".to_string()));
    }

    #[test]
    fn test_parse_menu_select() {
        let cmd = parse_command("1");
        assert_eq!(cmd.command_type, CommandType::MenuSelect("1".to_string()));

        let cmd = parse_command("A");
        assert_eq!(cmd.command_type, CommandType::MenuSelect("A".to_string()));
    }

    #[test]
    fn test_parse_empty() {
        let cmd = parse_command("");
        assert_eq!(cmd.command_type, CommandType::Empty);

        let cmd = parse_command("   ");
        assert_eq!(cmd.command_type, CommandType::Empty);
    }

    #[test]
    fn test_match_item() {
        let mut inv = std::collections::HashMap::new();
        inv.insert("pine_cone".to_string(), 3);
        inv.insert("health_potion".to_string(), 1);

        assert_eq!(match_item("pine", &inv), Some("pine_cone".to_string()));
        assert_eq!(match_item("potion", &inv), Some("health_potion".to_string()));
        assert_eq!(match_item("sword", &inv), None);
    }

    #[test]
    fn test_match_npc() {
        let npcs = vec!["innkeeper_mira", "elder_quinn"];

        assert_eq!(match_npc("mira", &npcs), Some("innkeeper_mira".to_string()));
        assert_eq!(match_npc("quinn", &npcs), Some("elder_quinn".to_string()));
        assert_eq!(match_npc("bob", &npcs), None);
    }

    #[test]
    fn test_is_valid_direction() {
        let dest = is_valid_direction("village_square", "north");
        assert_eq!(dest, Some("village_inn".to_string()));

        let dest = is_valid_direction("village_square", "up");
        assert_eq!(dest, None);
    }
}
