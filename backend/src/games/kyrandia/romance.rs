//! Romance system for Kyrandia
//! Supports relationships with NPCs and other players (including same-sex)

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use super::state::GameState;
use super::data::{get_npc, Gender};

/// Romance status with a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomanceState {
    /// Partner identifier (NPC key or player user_id)
    pub partner_id: String,
    /// Is this an NPC or player
    pub is_npc: bool,
    /// Partner's display name
    pub partner_name: String,
    /// Affection level (0-100)
    pub affection: u32,
    /// Relationship stage
    pub stage: RomanceStage,
    /// Times flirted today
    pub flirts_today: u32,
    /// Last flirt date
    pub last_flirt_date: String,
    /// Is married
    pub married: bool,
    /// Marriage date
    pub marriage_date: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RomanceStage {
    /// Just met
    Stranger,
    /// Acquainted
    Acquaintance,
    /// Friends
    Friend,
    /// Close friends
    CloseFriend,
    /// Romantic interest
    Interested,
    /// Dating
    Dating,
    /// Engaged
    Engaged,
    /// Married
    Married,
}

impl RomanceStage {
    pub fn name(&self) -> &'static str {
        match self {
            RomanceStage::Stranger => "Stranger",
            RomanceStage::Acquaintance => "Acquaintance",
            RomanceStage::Friend => "Friend",
            RomanceStage::CloseFriend => "Close Friend",
            RomanceStage::Interested => "Interested",
            RomanceStage::Dating => "Dating",
            RomanceStage::Engaged => "Engaged",
            RomanceStage::Married => "Married",
        }
    }

    pub fn required_affection(&self) -> u32 {
        match self {
            RomanceStage::Stranger => 0,
            RomanceStage::Acquaintance => 10,
            RomanceStage::Friend => 25,
            RomanceStage::CloseFriend => 40,
            RomanceStage::Interested => 55,
            RomanceStage::Dating => 70,
            RomanceStage::Engaged => 85,
            RomanceStage::Married => 95,
        }
    }

    pub fn next(&self) -> Option<RomanceStage> {
        match self {
            RomanceStage::Stranger => Some(RomanceStage::Acquaintance),
            RomanceStage::Acquaintance => Some(RomanceStage::Friend),
            RomanceStage::Friend => Some(RomanceStage::CloseFriend),
            RomanceStage::CloseFriend => Some(RomanceStage::Interested),
            RomanceStage::Interested => Some(RomanceStage::Dating),
            RomanceStage::Dating => Some(RomanceStage::Engaged),
            RomanceStage::Engaged => Some(RomanceStage::Married),
            RomanceStage::Married => None,
        }
    }
}

/// Result of a flirt attempt
#[derive(Debug)]
pub enum FlirtResult {
    /// Flirt successful, affection increased
    Success { message: String, affection_gained: u32 },
    /// Flirt failed (wrong stage, not romanceable, etc)
    Failed { message: String },
    /// Too many flirts today
    TooManyFlirts,
    /// Stage advanced!
    StageAdvanced { new_stage: RomanceStage, message: String },
    /// Not romanceable
    NotRomanceable,
    /// Already married to someone else
    AlreadyMarried,
}

/// Flirt stat bonuses per stage
pub fn get_romance_bonuses(stage: RomanceStage) -> RomanceBonus {
    match stage {
        RomanceStage::Stranger | RomanceStage::Acquaintance => RomanceBonus::default(),
        RomanceStage::Friend => RomanceBonus {
            health_regen: 1,
            mana_regen: 1,
            gold_bonus: 0,
            xp_bonus: 0,
        },
        RomanceStage::CloseFriend => RomanceBonus {
            health_regen: 2,
            mana_regen: 2,
            gold_bonus: 5,
            xp_bonus: 0,
        },
        RomanceStage::Interested => RomanceBonus {
            health_regen: 3,
            mana_regen: 3,
            gold_bonus: 5,
            xp_bonus: 5,
        },
        RomanceStage::Dating => RomanceBonus {
            health_regen: 5,
            mana_regen: 5,
            gold_bonus: 10,
            xp_bonus: 10,
        },
        RomanceStage::Engaged => RomanceBonus {
            health_regen: 7,
            mana_regen: 7,
            gold_bonus: 15,
            xp_bonus: 15,
        },
        RomanceStage::Married => RomanceBonus {
            health_regen: 10,
            mana_regen: 10,
            gold_bonus: 20,
            xp_bonus: 20,
        },
    }
}

/// Bonuses from romance
#[derive(Debug, Clone, Default)]
pub struct RomanceBonus {
    /// Bonus HP regen per rest
    pub health_regen: u32,
    /// Bonus mana regen per rest
    pub mana_regen: u32,
    /// Percentage gold bonus
    pub gold_bonus: u32,
    /// Percentage XP bonus
    pub xp_bonus: u32,
}

/// Attempt to flirt with an NPC
pub fn flirt_with_npc(state: &mut GameState, npc_key: &str) -> FlirtResult {
    // Check if NPC exists and is romanceable
    let npc = match get_npc(npc_key) {
        Some(n) => n,
        None => return FlirtResult::Failed {
            message: "That person isn't here.".to_string(),
        },
    };

    if !npc.is_romanceable {
        return FlirtResult::NotRomanceable;
    }

    // Check if already married to someone else
    if let Some(ref partner) = state.romance_partner {
        if partner != npc_key && state.romance_level >= RomanceStage::Married.required_affection() {
            return FlirtResult::AlreadyMarried;
        }
    }

    // Check daily limit
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if state.get_flag("last_flirt_date").map(|d| d == &today).unwrap_or(false) {
        let flirts: u32 = state.get_flag("flirts_today")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if flirts >= 3 {
            return FlirtResult::TooManyFlirts;
        }

        state.set_flag("flirts_today", &(flirts + 1).to_string());
    } else {
        state.set_flag("last_flirt_date", &today);
        state.set_flag("flirts_today", "1");
    }

    // If flirting with new person, start fresh
    if state.romance_partner.as_deref() != Some(npc_key) {
        state.romance_partner = Some(npc_key.to_string());
        state.romance_level = 0;
    }

    // Calculate affection gain (influenced by level)
    let base_gain = 5 + (state.level as u32);
    let affection_gained = rand::random::<u32>() % base_gain + base_gain / 2;

    state.romance_level = (state.romance_level + affection_gained).min(100);

    // Check for stage advancement
    let current_stage = get_stage_from_affection(state.romance_level);
    let prev_stage = get_stage_from_affection(state.romance_level - affection_gained);

    if current_stage != prev_stage {
        let message = get_stage_advancement_message(npc.name, current_stage, npc.gender);
        return FlirtResult::StageAdvanced {
            new_stage: current_stage,
            message,
        };
    }

    // Generate flirt message
    let message = generate_flirt_message(npc.name, current_stage, npc.gender);

    FlirtResult::Success {
        message,
        affection_gained,
    }
}

fn get_stage_from_affection(affection: u32) -> RomanceStage {
    if affection >= 95 {
        RomanceStage::Married
    } else if affection >= 85 {
        RomanceStage::Engaged
    } else if affection >= 70 {
        RomanceStage::Dating
    } else if affection >= 55 {
        RomanceStage::Interested
    } else if affection >= 40 {
        RomanceStage::CloseFriend
    } else if affection >= 25 {
        RomanceStage::Friend
    } else if affection >= 10 {
        RomanceStage::Acquaintance
    } else {
        RomanceStage::Stranger
    }
}

fn generate_flirt_message(name: &str, stage: RomanceStage, _gender: Gender) -> String {
    match stage {
        RomanceStage::Stranger => {
            format!("{} gives you a curious look.", name)
        }
        RomanceStage::Acquaintance => {
            format!("{} smiles politely at your words.", name)
        }
        RomanceStage::Friend => {
            format!("{} laughs warmly at your jest.", name)
        }
        RomanceStage::CloseFriend => {
            format!("{} blushes slightly at the compliment.", name)
        }
        RomanceStage::Interested => {
            format!("{} gazes at you with interest.", name)
        }
        RomanceStage::Dating => {
            format!("{} takes your hand gently.", name)
        }
        RomanceStage::Engaged => {
            format!("{} looks at you with adoration.", name)
        }
        RomanceStage::Married => {
            format!("{} kisses you tenderly.", name)
        }
    }
}

fn get_stage_advancement_message(name: &str, stage: RomanceStage, _gender: Gender) -> String {
    match stage {
        RomanceStage::Acquaintance => {
            format!("{} seems to be warming up to you!", name)
        }
        RomanceStage::Friend => {
            format!("{} considers you a friend now.", name)
        }
        RomanceStage::CloseFriend => {
            format!("{} has become a close friend!", name)
        }
        RomanceStage::Interested => {
            format!("{} looks at you differently... could it be?", name)
        }
        RomanceStage::Dating => {
            format!("You and {} have started seeing each other!", name)
        }
        RomanceStage::Engaged => {
            format!("{} has said yes! You are engaged!", name)
        }
        RomanceStage::Married => {
            format!(
                "You and {} have married! May you live happily ever after!",
                name
            )
        }
        RomanceStage::Stranger => "".to_string(),
    }
}

/// Propose marriage (requires Dating stage, 85+ affection)
pub fn propose(state: &mut GameState) -> Result<String, String> {
    if state.romance_partner.is_none() {
        return Err("You aren't seeing anyone.".to_string());
    }

    if state.romance_level < 85 {
        return Err("Your relationship isn't strong enough yet.".to_string());
    }

    let partner = state.romance_partner.as_ref().unwrap().clone();

    // Check if NPC
    if let Some(npc) = get_npc(&partner) {
        state.romance_level = 95;
        state.set_flag("married_to", &partner);
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        state.set_flag("marriage_date", &date);

        Ok(format!(
            "You kneel before {} and ask for their hand in marriage.\n\
             They accept! You are now married!",
            npc.name
        ))
    } else {
        // Player marriage would require different handling
        Err("Player marriages require consent from both parties.".to_string())
    }
}

/// Divorce (costs gold and romance resets)
pub fn divorce(state: &mut GameState) -> Result<String, String> {
    if state.romance_partner.is_none() {
        return Err("You aren't in a relationship.".to_string());
    }

    let divorce_cost = 500;
    if state.gold < divorce_cost {
        return Err(format!("Divorce costs {} gold.", divorce_cost));
    }

    let partner = state.romance_partner.take().unwrap();
    let name = get_npc(&partner)
        .map(|n| n.name.to_string())
        .unwrap_or(partner.clone());

    state.gold -= divorce_cost;
    state.romance_level = 0;
    state.quest_flags.remove("married_to");
    state.quest_flags.remove("marriage_date");

    Ok(format!(
        "Your relationship with {} has ended. (-{} gold)",
        name, divorce_cost
    ))
}

/// Get romance status description
pub fn get_romance_status(state: &GameState) -> Option<String> {
    let partner = state.romance_partner.as_ref()?;
    let stage = get_stage_from_affection(state.romance_level);
    let name = get_npc(partner)
        .map(|n| n.name.to_string())
        .unwrap_or(partner.clone());

    Some(format!(
        "{}: {} ({}%)",
        name,
        stage.name(),
        state.romance_level
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_romance_stages() {
        assert_eq!(RomanceStage::Stranger.required_affection(), 0);
        assert_eq!(RomanceStage::Married.required_affection(), 95);
    }

    #[test]
    fn test_stage_from_affection() {
        assert_eq!(get_stage_from_affection(0), RomanceStage::Stranger);
        assert_eq!(get_stage_from_affection(50), RomanceStage::CloseFriend);
        assert_eq!(get_stage_from_affection(100), RomanceStage::Married);
    }

    #[test]
    fn test_flirt_with_romanceable() {
        let mut state = GameState::new("Test");
        state.current_room = "village_inn".to_string();

        let result = flirt_with_npc(&mut state, "innkeeper_mira");

        match result {
            FlirtResult::Success { affection_gained, .. } => {
                assert!(affection_gained > 0);
                assert_eq!(state.romance_partner, Some("innkeeper_mira".to_string()));
            }
            _ => panic!("Expected successful flirt"),
        }
    }

    #[test]
    fn test_flirt_with_non_romanceable() {
        let mut state = GameState::new("Test");

        let result = flirt_with_npc(&mut state, "elder_quinn");

        assert!(matches!(result, FlirtResult::NotRomanceable));
    }

    #[test]
    fn test_romance_bonuses() {
        let bonus = get_romance_bonuses(RomanceStage::Married);
        assert!(bonus.health_regen > 0);
        assert!(bonus.xp_bonus > 0);
    }

    #[test]
    fn test_divorce() {
        let mut state = GameState::new("Test");
        state.romance_partner = Some("innkeeper_mira".to_string());
        state.romance_level = 100;
        state.gold = 1000;
        state.set_flag("married_to", "innkeeper_mira");

        let result = divorce(&mut state);
        assert!(result.is_ok());
        assert!(state.romance_partner.is_none());
        assert!(state.gold < 1000);
    }
}
