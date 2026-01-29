use rand::prelude::*;
use crate::game::{GameState, DeliveryQuest, CITIES, COMMODITIES, GANGS, get_city, get_gang, format_money};

// ============================================================================
// GANG RELATIONS
// ============================================================================

/// Pay tribute to a gang to improve relations
pub fn pay_tribute(state: &mut GameState, gang_key: &str) -> Result<String, String> {
    let gang = get_gang(gang_key).ok_or("Unknown gang.")?;

    if state.cash < gang.tribute_cost {
        return Err(format!(
            "The {} require {} tribute.",
            gang.name,
            format_money(gang.tribute_cost)
        ));
    }

    state.cash -= gang.tribute_cost;
    let current = state.gang_relations.get(gang_key).copied().unwrap_or(0);
    let new_relation = (current + 30).min(100);
    state.gang_relations.insert(gang_key.to_string(), new_relation);

    Ok(format!(
        "You paid tribute to {}. They appreciate your respect.",
        gang.name
    ))
}

/// Get gang relation status text
pub fn gang_status(relation: i32) -> &'static str {
    if relation >= 50 {
        "Allied"
    } else if relation >= 0 {
        "Neutral"
    } else if relation >= -50 {
        "Hostile"
    } else {
        "Enemy"
    }
}

// ============================================================================
// DELIVERY QUESTS
// ============================================================================

/// Generate a random delivery quest
pub fn generate_delivery_quest(state: &GameState, rng: &mut impl Rng) -> Option<DeliveryQuest> {
    // Can only have 3 active deliveries
    if state.quest_state.active_deliveries.len() >= 3 {
        return None;
    }

    // Pick random commodity
    let commodity = &COMMODITIES[rng.gen_range(0..COMMODITIES.len())];

    // Pick random destination (different city or different borough)
    let dest_city = &CITIES[rng.gen_range(0..CITIES.len())];
    let dest_borough = &dest_city.boroughs[rng.gen_range(0..dest_city.boroughs.len())];

    // Don't deliver to current location
    if dest_city.key == state.city && dest_borough.key == state.location {
        return None;
    }

    // Quest parameters
    let quantity = rng.gen_range(5..20);
    let base_reward = commodity.min_price * (quantity as i64);
    let reward = base_reward + rng.gen_range(base_reward / 2..base_reward * 2);
    let expires = state.day + rng.gen_range(3..7);

    let id = format!("del_{}_{}", state.day, rng.gen_range(1000..9999));

    Some(DeliveryQuest {
        id,
        commodity: commodity.key.to_string(),
        quantity,
        from_location: state.location.clone(),
        to_location: format!("{}/{}", dest_city.key, dest_borough.key),
        reward,
        expires_day: expires,
    })
}

/// Accept a delivery quest
pub fn accept_delivery(state: &mut GameState, quest: DeliveryQuest) -> Result<(), String> {
    if state.quest_state.active_deliveries.len() >= 3 {
        return Err("You can only have 3 active deliveries.".to_string());
    }

    // Check if player has the goods
    let owned = state.inventory.get(&quest.commodity).copied().unwrap_or(0);
    if owned < quest.quantity {
        return Err(format!(
            "You need {} {} to accept this job.",
            quest.quantity,
            quest.commodity
        ));
    }

    // Reserve the goods (remove from inventory)
    *state.inventory.entry(quest.commodity.clone()).or_insert(0) -= quest.quantity;
    if state.inventory.get(&quest.commodity).copied().unwrap_or(0) == 0 {
        state.inventory.remove(&quest.commodity);
    }

    state.quest_state.active_deliveries.push(quest);
    Ok(())
}

/// Check and complete deliveries at current location
pub fn check_deliveries(state: &mut GameState) -> Vec<(DeliveryQuest, i64)> {
    let current_loc = format!("{}/{}", state.city, state.location);
    let mut completed = Vec::new();

    // Find deliveries to this location
    let to_complete: Vec<usize> = state
        .quest_state
        .active_deliveries
        .iter()
        .enumerate()
        .filter(|(_, q)| q.to_location == current_loc)
        .map(|(i, _)| i)
        .collect();

    // Remove in reverse order to preserve indices
    for idx in to_complete.into_iter().rev() {
        let quest = state.quest_state.active_deliveries.remove(idx);
        let reward = quest.reward;
        state.cash += reward;
        state.quest_state.completed_deliveries += 1;
        completed.push((quest, reward));
    }

    completed
}

/// Expire old deliveries
pub fn expire_deliveries(state: &mut GameState) -> Vec<DeliveryQuest> {
    let current_day = state.day;
    let mut expired = Vec::new();

    state.quest_state.active_deliveries.retain(|q| {
        if q.expires_day < current_day {
            expired.push(q.clone());
            false
        } else {
            true
        }
    });

    // Lost goods from expired deliveries hurt gang relations
    for quest in &expired {
        // Determine which gang this was for (based on destination)
        if let Some(gang_key) = get_territory_gang(&quest.to_location) {
            let current = state.gang_relations.get(&gang_key).copied().unwrap_or(0);
            state.gang_relations.insert(gang_key, (current - 10).max(-100));
        }
    }

    expired
}

/// Get gang controlling a territory
fn get_territory_gang(location: &str) -> Option<String> {
    let parts: Vec<&str> = location.split('/').collect();
    if parts.len() != 2 {
        return None;
    }

    if let Some(city) = get_city(parts[0]) {
        for borough in city.boroughs {
            if borough.key == parts[1] {
                return borough.gang_territory.map(|s| s.to_string());
            }
        }
    }
    None
}

// ============================================================================
// STORY QUEST
// ============================================================================

/// Story quest steps - each has location requirement and narrative
pub static STORY_STEPS: &[StoryStep] = &[
    StoryStep {
        step: 1,
        title: "The Old Contact",
        location: Some("nyc/bronx"),
        narrative: "Your old partner Marcus left a message. Meet him in the Bronx.",
        requirement: None,
        min_net_worth: None,
        reward: 50000,
    },
    StoryStep {
        step: 2,
        title: "A Simple Job",
        location: Some("nyc/brooklyn"),
        narrative: "Marcus has a small delivery. 5 units of weed to Brooklyn. Easy money.",
        requirement: Some("weed:5"),
        min_net_worth: None,
        reward: 25000,
    },
    StoryStep {
        step: 3,
        title: "Something's Off",
        location: Some("nyc/manhattan"),
        narrative: "The drop went wrong. Someone set you up. Meet the Mafia in Manhattan.",
        requirement: None,
        min_net_worth: None,
        reward: 0,
    },
    StoryStep {
        step: 4,
        title: "Earning Trust",
        location: Some("miami/little_havana"),
        narrative: "The Mafia wants proof of loyalty. Deliver 10 cocaine to their Cartel contact in Miami.",
        requirement: Some("cocaine:10"),
        min_net_worth: None,
        reward: 100000,
    },
    StoryStep {
        step: 5,
        title: "The Bigger Picture",
        location: Some("miami/south_beach"),
        narrative: "The Cartel has information. Something is happening. Meet at South Beach.",
        requirement: None,
        min_net_worth: None,
        reward: 50000,
    },
    StoryStep {
        step: 6,
        title: "Crossing the Pond",
        location: Some("london/east_end"),
        narrative: "An international syndicate is moving in. Travel to London's East End.",
        requirement: None,
        min_net_worth: None,
        reward: 75000,
    },
    StoryStep {
        step: 7,
        title: "The Triads",
        location: Some("tokyo/shinjuku"),
        narrative: "The Triads in Tokyo have a piece of the puzzle. Bring 15 meth as a gift.",
        requirement: Some("meth:15"),
        min_net_worth: None,
        reward: 150000,
    },
    StoryStep {
        step: 8,
        title: "The Source",
        location: Some("bogota/chapinero"),
        narrative: "Everything leads back to Bogota. The Cartel's stronghold.",
        requirement: None,
        min_net_worth: None,
        reward: 100000,
    },
    StoryStep {
        step: 9,
        title: "The Betrayal",
        location: Some("bogota/la_candelaria"),
        narrative: "Marcus. It was always Marcus. He's here, in La Candelaria.",
        requirement: None,
        min_net_worth: None,
        reward: 0,
    },
    StoryStep {
        step: 10,
        title: "Old Friends",
        location: Some("nyc/queens"),
        narrative: "Marcus escaped. But you have connections. The Triads owe you. Meet in Queens.",
        requirement: None,
        min_net_worth: None,
        reward: 75000,
    },
    StoryStep {
        step: 11,
        title: "The Plan",
        location: Some("london/soho"),
        narrative: "Every organization wants Marcus dead. Coordinate in London's Soho.",
        requirement: None,
        min_net_worth: None,
        reward: 100000,
    },
    StoryStep {
        step: 12,
        title: "War Chest",
        location: None,
        narrative: "You need resources. Accumulate $500,000 net worth to fund the operation.",
        requirement: None,
        min_net_worth: Some(50000000), // $500,000
        reward: 0,
    },
    StoryStep {
        step: 13,
        title: "The Hunt",
        location: Some("miami/downtown_miami"),
        narrative: "Marcus is in Miami. The final confrontation approaches.",
        requirement: None,
        min_net_worth: None,
        reward: 200000,
    },
    StoryStep {
        step: 14,
        title: "Showdown",
        location: Some("nyc/manhattan"),
        narrative: "Marcus made his last move. End this in Manhattan, where it all began.",
        requirement: None,
        min_net_worth: None,
        reward: 500000,
    },
    StoryStep {
        step: 15,
        title: "Kingpin",
        location: None,
        narrative: "You've taken down Marcus and united the syndicates. You are the Kingpin.",
        requirement: None,
        min_net_worth: None,
        reward: 1000000,
    },
];

pub struct StoryStep {
    pub step: u32,
    pub title: &'static str,
    pub location: Option<&'static str>,
    pub narrative: &'static str,
    pub requirement: Option<&'static str>, // "commodity:quantity"
    pub min_net_worth: Option<i64>,
    pub reward: i64,
}

impl Default for StoryStep {
    fn default() -> Self {
        Self {
            step: 0,
            title: "",
            location: None,
            narrative: "",
            requirement: None,
            min_net_worth: None,
            reward: 0,
        }
    }
}

/// Get current story step info
pub fn get_current_story(state: &GameState) -> Option<&'static StoryStep> {
    let step = state.quest_state.story_step;
    STORY_STEPS.iter().find(|s| s.step == step + 1)
}

/// Check if story step can be completed at current location
pub fn can_complete_story_step(state: &GameState, prices: &std::collections::HashMap<String, i64>) -> bool {
    let step = match get_current_story(state) {
        Some(s) => s,
        None => return false,
    };

    // Check location requirement
    if let Some(loc) = step.location {
        let current = format!("{}/{}", state.city, state.location);
        if current != loc {
            return false;
        }
    }

    // Check commodity requirement
    if let Some(req) = step.requirement {
        let parts: Vec<&str> = req.split(':').collect();
        if parts.len() == 2 {
            let commodity = parts[0];
            let quantity: u32 = parts[1].parse().unwrap_or(0);
            let owned = state.inventory.get(commodity).copied().unwrap_or(0);
            if owned < quantity {
                return false;
            }
        }
    }

    // Check net worth requirement
    if let Some(min_worth) = step.min_net_worth {
        if state.net_worth(prices) < min_worth {
            return false;
        }
    }

    true
}

/// Complete current story step
pub fn complete_story_step(state: &mut GameState, prices: &std::collections::HashMap<String, i64>) -> Result<(String, i64), String> {
    if !can_complete_story_step(state, prices) {
        return Err("Requirements not met.".to_string());
    }

    let step = get_current_story(state).unwrap();

    // Consume required commodities
    if let Some(req) = step.requirement {
        let parts: Vec<&str> = req.split(':').collect();
        if parts.len() == 2 {
            let commodity = parts[0];
            let quantity: u32 = parts[1].parse().unwrap_or(0);
            *state.inventory.entry(commodity.to_string()).or_insert(0) -= quantity;
            if state.inventory.get(commodity).copied().unwrap_or(0) == 0 {
                state.inventory.remove(commodity);
            }
        }
    }

    // Award reward
    state.cash += step.reward;

    // Advance story
    state.quest_state.story_step += 1;

    let title = step.title.to_string();
    let reward = step.reward;

    // Boost gang relations on story completion
    for (_gang_key, relation) in state.gang_relations.iter_mut() {
        *relation = (*relation + 5).min(100);
    }

    Ok((title, reward))
}

/// Check if story is complete
pub fn is_story_complete(state: &GameState) -> bool {
    state.quest_state.story_step >= 15
}
