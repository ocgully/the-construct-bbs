//! Romance system for Dragon Slayer
//! Flirting, dating, marriage with NPCs and players

use rand::prelude::*;
use super::state::GameState;

/// Result of a romance action
#[derive(Debug, Clone)]
pub enum RomanceResult {
    Success {
        message: String,
        #[allow(dead_code)]
        affection_gain: u8,
    },
    Failure { message: String },
    AlreadyMarried { message: String },
    DailyLimitReached { message: String },
    ProposalAccepted { message: String },
    ProposalRejected { message: String },
    Divorced { message: String },
}

/// NPC romance targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomanceNpc {
    Violet,
    Seth,
}

impl RomanceNpc {
    pub fn name(&self) -> &'static str {
        match self {
            RomanceNpc::Violet => "Violet",
            RomanceNpc::Seth => "Seth the Bard",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            RomanceNpc::Violet => "The charming barmaid with violet eyes.",
            RomanceNpc::Seth => "A handsome bard whose songs enchant all who listen.",
        }
    }
}

/// Maximum flirts per day
const MAX_FLIRTS_PER_DAY: u8 = 5;

/// Affection needed to propose
const AFFECTION_TO_PROPOSE: u8 = 100;

/// Marriage stat bonuses
const MARRIAGE_CHARM_BONUS: u32 = 10;
const MARRIAGE_STRENGTH_BONUS: u32 = 5;

/// Attempt to flirt with an NPC
pub fn flirt_with_npc(state: &mut GameState, target: RomanceNpc) -> RomanceResult {
    // Check if already married
    if let Some(spouse) = &state.romance.spouse {
        if spouse == target.name() {
            return RomanceResult::AlreadyMarried {
                message: format!("You're already married to {}!", target.name()),
            };
        }
        return RomanceResult::Failure {
            message: format!("You're married to {}. That wouldn't be right.", spouse),
        };
    }

    // Check daily limit
    if state.romance.flirts_today >= MAX_FLIRTS_PER_DAY {
        return RomanceResult::DailyLimitReached {
            message: "You've flirted enough for today. Try again tomorrow!".to_string(),
        };
    }

    state.romance.flirts_today += 1;

    // Calculate success chance based on charm
    let mut rng = thread_rng();
    let success_chance = 20 + state.charm * 2;

    if rng.gen_range(0..100) < success_chance as u32 {
        // Success! Gain affection
        let affection_gain = rng.gen_range(3..8);
        let current = match target {
            RomanceNpc::Violet => &mut state.romance.violet_affection,
            RomanceNpc::Seth => &mut state.romance.seth_affection,
        };
        *current = current.saturating_add(affection_gain);

        let responses = get_flirt_responses(target, *current);
        let response = responses[rng.gen_range(0..responses.len())];

        RomanceResult::Success {
            message: response.to_string(),
            affection_gain,
        }
    } else {
        let rejections = get_rejection_responses(target);
        let response = rejections[rng.gen_range(0..rejections.len())];

        RomanceResult::Failure {
            message: response.to_string(),
        }
    }
}

/// Attempt to propose marriage to an NPC
pub fn propose_to_npc(state: &mut GameState, target: RomanceNpc, gold_cost: i64) -> RomanceResult {
    // Check if already married
    if state.romance.spouse.is_some() {
        return RomanceResult::AlreadyMarried {
            message: "You're already married!".to_string(),
        };
    }

    // Check affection level
    let affection = match target {
        RomanceNpc::Violet => state.romance.violet_affection,
        RomanceNpc::Seth => state.romance.seth_affection,
    };

    if affection < AFFECTION_TO_PROPOSE {
        return RomanceResult::ProposalRejected {
            message: format!(
                "{} isn't ready for marriage yet. (Need {} affection, have {})",
                target.name(), AFFECTION_TO_PROPOSE, affection
            ),
        };
    }

    // Check gold for ring
    if state.gold_pocket < gold_cost {
        return RomanceResult::ProposalRejected {
            message: format!(
                "You need {} gold for a ring to propose!",
                gold_cost
            ),
        };
    }

    // Proposal succeeds!
    state.gold_pocket -= gold_cost;
    state.romance.spouse = Some(target.name().to_string());
    state.romance.married_date = Some(chrono::Local::now().format("%Y-%m-%d").to_string());

    // Apply marriage bonuses
    state.charm += MARRIAGE_CHARM_BONUS;
    state.stats.strength += MARRIAGE_STRENGTH_BONUS;

    RomanceResult::ProposalAccepted {
        message: format!(
            "{} says YES! You are now married!\n+{} Charm, +{} Strength!",
            target.name(), MARRIAGE_CHARM_BONUS, MARRIAGE_STRENGTH_BONUS
        ),
    }
}

/// Get a divorce
pub fn divorce(state: &mut GameState) -> RomanceResult {
    if let Some(spouse) = state.romance.spouse.take() {
        state.romance.married_date = None;

        // Remove bonuses
        state.charm = state.charm.saturating_sub(MARRIAGE_CHARM_BONUS);
        state.stats.strength = state.stats.strength.saturating_sub(MARRIAGE_STRENGTH_BONUS);

        // Reset affection
        state.romance.violet_affection = 0;
        state.romance.seth_affection = 0;

        RomanceResult::Divorced {
            message: format!(
                "You and {} have divorced. It's for the best.\n-{} Charm, -{} Strength",
                spouse, MARRIAGE_CHARM_BONUS, MARRIAGE_STRENGTH_BONUS
            ),
        }
    } else {
        RomanceResult::Failure {
            message: "You're not married!".to_string(),
        }
    }
}

/// Get flirt responses based on affection level
fn get_flirt_responses(target: RomanceNpc, affection: u8) -> Vec<&'static str> {
    match target {
        RomanceNpc::Violet => {
            if affection < 30 {
                vec![
                    "Violet blushes slightly. 'You're sweet.'",
                    "She smiles at you over the bar.",
                    "Violet winks. 'Buy a drink, handsome?'",
                ]
            } else if affection < 70 {
                vec![
                    "Violet touches your hand. 'I was hoping you'd come in.'",
                    "'My shift ends soon...' she says with a smile.",
                    "She leans close. 'You're different from the others.'",
                ]
            } else {
                vec![
                    "Violet embraces you. 'I've been thinking about you.'",
                    "'Stay with me tonight?' she whispers.",
                    "Her eyes sparkle. 'I think I'm falling for you.'",
                ]
            }
        }
        RomanceNpc::Seth => {
            if affection < 30 {
                vec![
                    "Seth strums a chord. 'A fan of my music?'",
                    "He gives you a dashing smile.",
                    "'Care to hear a song?' Seth asks.",
                ]
            } else if affection < 70 {
                vec![
                    "Seth sings a love ballad, looking at you.",
                    "'This next song is for someone special...'",
                    "He takes your hand. 'Walk with me?'",
                ]
            } else {
                vec![
                    "Seth pulls you into a passionate embrace.",
                    "'I write all my songs about you now.'",
                    "'My heart beats only for you,' he declares.",
                ]
            }
        }
    }
}

/// Get rejection responses
fn get_rejection_responses(target: RomanceNpc) -> Vec<&'static str> {
    match target {
        RomanceNpc::Violet => vec![
            "Violet is busy with other customers.",
            "'Not now, I'm working.' she says politely.",
            "She seems distracted tonight.",
        ],
        RomanceNpc::Seth => vec![
            "Seth is focused on his music.",
            "'Another time, perhaps.' he says.",
            "He's surrounded by adoring fans.",
        ],
    }
}

/// Check if player can visit a romance location
#[allow(dead_code)]
pub fn can_visit_romance(_state: &GameState, _target: RomanceNpc) -> bool {
    // Can always visit, but different interactions if married
    true
}

/// Get marriage benefits description
#[allow(dead_code)]
pub fn describe_marriage_benefits() -> String {
    format!(
        "Marriage Benefits:\n\
         - +{} Charm (helps with all interactions)\n\
         - +{} Strength (more damage in combat)\n\
         - Daily bonus gold from spouse\n\
         - Special dialogue options",
        MARRIAGE_CHARM_BONUS, MARRIAGE_STRENGTH_BONUS
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::Sex;

    #[test]
    fn test_flirt_daily_limit() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);

        for _ in 0..MAX_FLIRTS_PER_DAY {
            let result = flirt_with_npc(&mut state, RomanceNpc::Violet);
            assert!(!matches!(result, RomanceResult::DailyLimitReached { .. }));
        }

        // Next flirt should be limited
        let result = flirt_with_npc(&mut state, RomanceNpc::Violet);
        assert!(matches!(result, RomanceResult::DailyLimitReached { .. }));
    }

    #[test]
    fn test_proposal_requires_affection() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        state.gold_pocket = 10000;

        let result = propose_to_npc(&mut state, RomanceNpc::Violet, 1000);
        assert!(matches!(result, RomanceResult::ProposalRejected { .. }));
    }

    #[test]
    fn test_successful_marriage() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        state.gold_pocket = 10000;
        state.romance.violet_affection = 100;
        let initial_charm = state.charm;

        let result = propose_to_npc(&mut state, RomanceNpc::Violet, 1000);
        assert!(matches!(result, RomanceResult::ProposalAccepted { .. }));
        assert!(state.romance.spouse.is_some());
        assert!(state.charm > initial_charm);
    }

    #[test]
    fn test_divorce() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        state.romance.spouse = Some("Violet".to_string());
        state.charm = 50;

        let result = divorce(&mut state);
        assert!(matches!(result, RomanceResult::Divorced { .. }));
        assert!(state.romance.spouse.is_none());
    }

    #[test]
    fn test_cant_flirt_while_married() {
        let mut state = GameState::new("Test".to_string(), Sex::Male);
        state.romance.spouse = Some("Seth the Bard".to_string());

        // Flirting with someone else should fail
        let result = flirt_with_npc(&mut state, RomanceNpc::Violet);
        assert!(matches!(result, RomanceResult::Failure { .. }));
    }
}
