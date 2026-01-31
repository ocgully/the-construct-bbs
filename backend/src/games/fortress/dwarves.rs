//! Dwarf management system
//!
//! Handles dwarf creation, needs, skills, and AI behavior.

use serde::{Serialize, Deserialize};
use rand::Rng;

/// A dwarf colonist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dwarf {
    pub id: u32,
    pub name: String,
    pub profession: String,

    // Position
    pub x: u32,
    pub y: u32,
    pub z: u32,

    // Stats
    pub health: u32,
    pub max_health: u32,
    pub age: u32,

    // Needs (0-100, lower is worse)
    pub needs: DwarfNeeds,

    // Skills (0-20 levels)
    pub skills: DwarfSkills,

    // Current state
    pub current_job: Option<u32>,  // Job ID if working
    pub status: DwarfStatus,
    pub mood: DwarfMood,

    // Equipment
    pub equipped_weapon: Option<String>,
    pub equipped_armor: Option<String>,

    // Personal
    pub room_id: Option<u32>,      // Assigned bedroom
    pub friends: Vec<u32>,          // Other dwarf IDs
    pub likes: Vec<String>,         // Preferred materials/activities
    pub dislikes: Vec<String>,
}

/// Dwarf needs that must be satisfied
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DwarfNeeds {
    pub hunger: u8,       // 100 = full, 0 = starving
    pub thirst: u8,       // 100 = satisfied, 0 = dehydrated
    pub rest: u8,         // 100 = rested, 0 = exhausted
    pub social: u8,       // 100 = happy, 0 = lonely
    pub comfort: u8,      // Quality of life satisfaction
}

impl DwarfNeeds {
    pub fn new() -> Self {
        Self {
            hunger: 100,
            thirst: 100,
            rest: 100,
            social: 70,
            comfort: 50,
        }
    }

    /// Get average need satisfaction (0-100)
    pub fn average(&self) -> u8 {
        ((self.hunger as u32 + self.thirst as u32 + self.rest as u32 +
          self.social as u32 + self.comfort as u32) / 5) as u8
    }

    /// Get most urgent need
    pub fn most_urgent(&self) -> (&'static str, u8) {
        let mut lowest = ("hunger", self.hunger);

        if self.thirst < lowest.1 { lowest = ("thirst", self.thirst); }
        if self.rest < lowest.1 { lowest = ("rest", self.rest); }
        if self.social < lowest.1 { lowest = ("social", self.social); }
        if self.comfort < lowest.1 { lowest = ("comfort", self.comfort); }

        lowest
    }

    /// Decay needs over time
    pub fn decay(&mut self, ticks: u32) {
        let decay_amount = (ticks * 2) as u8;

        self.hunger = self.hunger.saturating_sub(decay_amount);
        self.thirst = self.thirst.saturating_sub(decay_amount * 2); // Thirst decays faster
        self.rest = self.rest.saturating_sub(decay_amount / 2);
        self.social = self.social.saturating_sub(decay_amount / 4);
    }

    /// Check if dwarf is in critical state
    pub fn is_critical(&self) -> bool {
        self.hunger < 10 || self.thirst < 10 || self.rest < 10
    }
}

/// Dwarf skill levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DwarfSkills {
    pub mining: u8,
    pub woodcutting: u8,
    pub farming: u8,
    pub crafting: u8,
    pub cooking: u8,
    pub building: u8,
    pub combat: u8,
    pub hauling: u8,
    pub masonry: u8,
    pub smithing: u8,
    pub brewing: u8,
    pub healing: u8,
}

impl DwarfSkills {
    pub fn get(&self, skill: &str) -> u8 {
        match skill {
            "mining" => self.mining,
            "woodcutting" => self.woodcutting,
            "farming" => self.farming,
            "crafting" => self.crafting,
            "cooking" => self.cooking,
            "building" => self.building,
            "combat" => self.combat,
            "hauling" => self.hauling,
            "masonry" => self.masonry,
            "smithing" => self.smithing,
            "brewing" => self.brewing,
            "healing" => self.healing,
            _ => 0,
        }
    }

    pub fn set(&mut self, skill: &str, level: u8) {
        let level = level.min(20);
        match skill {
            "mining" => self.mining = level,
            "woodcutting" => self.woodcutting = level,
            "farming" => self.farming = level,
            "crafting" => self.crafting = level,
            "cooking" => self.cooking = level,
            "building" => self.building = level,
            "combat" => self.combat = level,
            "hauling" => self.hauling = level,
            "masonry" => self.masonry = level,
            "smithing" => self.smithing = level,
            "brewing" => self.brewing = level,
            "healing" => self.healing = level,
            _ => {}
        }
    }

    pub fn improve(&mut self, skill: &str, amount: u8) {
        let current = self.get(skill);
        self.set(skill, current.saturating_add(amount).min(20));
    }

    /// Get best skill
    pub fn best(&self) -> (&'static str, u8) {
        let mut best = ("hauling", self.hauling);

        if self.mining > best.1 { best = ("mining", self.mining); }
        if self.woodcutting > best.1 { best = ("woodcutting", self.woodcutting); }
        if self.farming > best.1 { best = ("farming", self.farming); }
        if self.crafting > best.1 { best = ("crafting", self.crafting); }
        if self.cooking > best.1 { best = ("cooking", self.cooking); }
        if self.building > best.1 { best = ("building", self.building); }
        if self.combat > best.1 { best = ("combat", self.combat); }
        if self.masonry > best.1 { best = ("masonry", self.masonry); }
        if self.smithing > best.1 { best = ("smithing", self.smithing); }
        if self.brewing > best.1 { best = ("brewing", self.brewing); }
        if self.healing > best.1 { best = ("healing", self.healing); }

        best
    }

    /// Get effective skill level (affected by mood)
    pub fn effective_level(&self, skill: &str, mood: &DwarfMood) -> u8 {
        let base = self.get(skill);
        let modifier: i8 = match mood {
            DwarfMood::Ecstatic => 4,
            DwarfMood::Happy => 2,
            DwarfMood::Content => 0,
            DwarfMood::Unhappy => -2,
            DwarfMood::Miserable => -4,
            DwarfMood::Tantrum => 0, // Too angry to work well
        };

        if modifier >= 0 {
            base.saturating_add(modifier as u8).min(20)
        } else {
            base.saturating_sub(modifier.unsigned_abs())
        }
    }
}

/// Dwarf's current status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DwarfStatus {
    Idle,
    Working,
    Eating,
    Drinking,
    Sleeping,
    Socializing,
    Fighting,
    Fleeing,
    Injured,
    Dead,
}

impl DwarfStatus {
    pub fn description(&self) -> &'static str {
        match self {
            DwarfStatus::Idle => "Idle",
            DwarfStatus::Working => "Working",
            DwarfStatus::Eating => "Eating",
            DwarfStatus::Drinking => "Drinking",
            DwarfStatus::Sleeping => "Sleeping",
            DwarfStatus::Socializing => "Socializing",
            DwarfStatus::Fighting => "Fighting",
            DwarfStatus::Fleeing => "Fleeing",
            DwarfStatus::Injured => "Injured",
            DwarfStatus::Dead => "Dead",
        }
    }
}

/// Dwarf's mood
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DwarfMood {
    Ecstatic,
    Happy,
    Content,
    Unhappy,
    Miserable,
    Tantrum,
}

impl DwarfMood {
    pub fn from_needs(needs: &DwarfNeeds) -> Self {
        let avg = needs.average();
        match avg {
            90..=100 => DwarfMood::Ecstatic,
            70..=89 => DwarfMood::Happy,
            50..=69 => DwarfMood::Content,
            30..=49 => DwarfMood::Unhappy,
            10..=29 => DwarfMood::Miserable,
            _ => DwarfMood::Tantrum,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DwarfMood::Ecstatic => "Ecstatic",
            DwarfMood::Happy => "Happy",
            DwarfMood::Content => "Content",
            DwarfMood::Unhappy => "Unhappy",
            DwarfMood::Miserable => "Miserable",
            DwarfMood::Tantrum => "Having a tantrum!",
        }
    }
}

impl Dwarf {
    /// Create a new dwarf with random traits
    pub fn new<R: Rng>(id: u32, rng: &mut R) -> Self {
        let name = Self::generate_name(rng);
        let profession = Self::random_profession(rng);

        let mut skills = DwarfSkills::default();

        // Give some starting skills based on profession
        match profession.as_str() {
            "Miner" => {
                skills.mining = rng.gen_range(3..8);
                skills.hauling = rng.gen_range(1..4);
            }
            "Woodcutter" => {
                skills.woodcutting = rng.gen_range(3..8);
                skills.building = rng.gen_range(1..4);
            }
            "Farmer" => {
                skills.farming = rng.gen_range(3..8);
                skills.cooking = rng.gen_range(1..4);
            }
            "Craftsdwarf" => {
                skills.crafting = rng.gen_range(3..8);
                skills.masonry = rng.gen_range(1..4);
            }
            "Smith" => {
                skills.smithing = rng.gen_range(3..8);
                skills.crafting = rng.gen_range(1..4);
            }
            "Cook" => {
                skills.cooking = rng.gen_range(3..8);
                skills.brewing = rng.gen_range(1..4);
            }
            "Soldier" => {
                skills.combat = rng.gen_range(3..8);
                skills.hauling = rng.gen_range(1..4);
            }
            _ => {
                // Peasant - basic hauling
                skills.hauling = rng.gen_range(2..5);
            }
        }

        Self {
            id,
            name,
            profession,
            x: 0,
            y: 0,
            z: 0,
            health: 100,
            max_health: 100,
            age: rng.gen_range(20..60),
            needs: DwarfNeeds::new(),
            skills,
            current_job: None,
            status: DwarfStatus::Idle,
            mood: DwarfMood::Content,
            equipped_weapon: None,
            equipped_armor: None,
            room_id: None,
            friends: Vec::new(),
            likes: Self::random_likes(rng),
            dislikes: Self::random_dislikes(rng),
        }
    }

    fn generate_name<R: Rng>(rng: &mut R) -> String {
        static FIRST_PARTS: &[&str] = &[
            "Ud", "Ug", "Ur", "Im", "In", "Ok", "Ak", "Ek", "Ol", "Ul",
            "Bom", "Bol", "Dol", "Dul", "Gim", "Gol", "Gor", "Kil", "Kol",
            "Thor", "Thur", "Dur", "Mur", "Nor", "Tor", "Bal", "Dal", "Nal",
        ];
        static SECOND_PARTS: &[&str] = &[
            "rist", "rim", "rin", "rok", "rek", "rak", "li", "lin", "dak",
            "grim", "grom", "thak", "bek", "lek", "drek", "fer", "gar", "bar",
            "mir", "nir", "sir", "zar", "zer", "zur", "dar", "kar", "tar",
        ];

        let first = FIRST_PARTS[rng.gen_range(0..FIRST_PARTS.len())];
        let second = SECOND_PARTS[rng.gen_range(0..SECOND_PARTS.len())];

        format!("{}{}", first, second)
    }

    fn random_profession<R: Rng>(rng: &mut R) -> String {
        static PROFESSIONS: &[&str] = &[
            "Miner", "Woodcutter", "Farmer", "Craftsdwarf", "Smith",
            "Cook", "Soldier", "Peasant", "Peasant", "Peasant", // More peasants
        ];
        PROFESSIONS[rng.gen_range(0..PROFESSIONS.len())].to_string()
    }

    fn random_likes<R: Rng>(rng: &mut R) -> Vec<String> {
        static LIKES: &[&str] = &[
            "gold", "gems", "ale", "fine meals", "crafts",
            "mining", "fighting", "sleeping", "stone", "iron",
        ];

        let count = rng.gen_range(1..4);
        (0..count)
            .map(|_| LIKES[rng.gen_range(0..LIKES.len())].to_string())
            .collect()
    }

    fn random_dislikes<R: Rng>(rng: &mut R) -> Vec<String> {
        static DISLIKES: &[&str] = &[
            "outdoors", "water", "vermin", "elves", "goblins",
            "sunlight", "cold", "heat", "noise", "crowds",
        ];

        let count = rng.gen_range(1..3);
        (0..count)
            .map(|_| DISLIKES[rng.gen_range(0..DISLIKES.len())].to_string())
            .collect()
    }

    /// Update mood based on current needs
    pub fn update_mood(&mut self) {
        self.mood = DwarfMood::from_needs(&self.needs);
    }

    /// Process one tick of dwarf behavior
    pub fn tick(&mut self) {
        // Decay needs
        self.needs.decay(1);

        // Update mood
        self.update_mood();

        // Handle critical states
        if self.needs.hunger == 0 || self.needs.thirst == 0 {
            self.health = self.health.saturating_sub(5);
        }

        if self.health == 0 {
            self.status = DwarfStatus::Dead;
        }
    }

    /// Eat food, restoring hunger
    pub fn eat(&mut self, quality: u8) {
        let restore = 30 + quality;
        self.needs.hunger = (self.needs.hunger as u16 + restore as u16).min(100) as u8;
        self.status = DwarfStatus::Eating;
    }

    /// Drink, restoring thirst
    pub fn drink(&mut self, quality: u8) {
        let restore = 40 + quality;
        self.needs.thirst = (self.needs.thirst as u16 + restore as u16).min(100) as u8;
        self.status = DwarfStatus::Drinking;

        // Alcohol boosts mood!
        if quality > 10 {
            self.needs.comfort = (self.needs.comfort as u16 + 5).min(100) as u8;
        }
    }

    /// Sleep, restoring rest
    pub fn sleep(&mut self, quality: u8) {
        let restore = 20 + quality;
        self.needs.rest = (self.needs.rest as u16 + restore as u16).min(100) as u8;
        self.status = DwarfStatus::Sleeping;
    }

    /// Socialize with another dwarf
    pub fn socialize(&mut self, other_id: u32) {
        self.needs.social = (self.needs.social as u16 + 10).min(100) as u8;

        // Maybe make a friend
        if !self.friends.contains(&other_id) && self.friends.len() < 5 {
            self.friends.push(other_id);
        }

        self.status = DwarfStatus::Socializing;
    }

    /// Take damage in combat
    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
        if self.health == 0 {
            self.status = DwarfStatus::Dead;
        } else if self.health < 20 {
            self.status = DwarfStatus::Injured;
        }
    }

    /// Heal damage
    pub fn heal(&mut self, amount: u32) {
        self.health = (self.health + amount).min(self.max_health);
        if self.status == DwarfStatus::Injured && self.health > 20 {
            self.status = DwarfStatus::Idle;
        }
    }

    /// Get combat power (attack + skill + weapon)
    pub fn combat_power(&self) -> u32 {
        let base = 5 + self.skills.combat as u32 * 3;
        let weapon_bonus = if self.equipped_weapon.is_some() { 10 } else { 0 };
        let armor_bonus = if self.equipped_armor.is_some() { 5 } else { 0 };

        base + weapon_bonus + armor_bonus
    }

    /// Get work efficiency (0-100 based on needs and mood)
    pub fn work_efficiency(&self) -> u8 {
        if self.status == DwarfStatus::Dead || self.status == DwarfStatus::Injured {
            return 0;
        }

        let need_factor = self.needs.average() as u32;
        let mood_factor = match self.mood {
            DwarfMood::Ecstatic => 120,
            DwarfMood::Happy => 110,
            DwarfMood::Content => 100,
            DwarfMood::Unhappy => 80,
            DwarfMood::Miserable => 60,
            DwarfMood::Tantrum => 0,
        };

        ((need_factor * mood_factor / 100) as u8).min(100)
    }

    /// Check if dwarf can take a job
    pub fn can_work(&self) -> bool {
        self.status == DwarfStatus::Idle
            && self.current_job.is_none()
            && self.mood != DwarfMood::Tantrum
            && !self.needs.is_critical()
    }

    /// Assign a job to this dwarf
    pub fn assign_job(&mut self, job_id: u32) {
        self.current_job = Some(job_id);
        self.status = DwarfStatus::Working;
    }

    /// Complete current job
    pub fn complete_job(&mut self, skill: &str) {
        // Improve skill
        self.skills.improve(skill, 1);

        self.current_job = None;
        self.status = DwarfStatus::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_dwarf_creation() {
        let mut rng = StdRng::seed_from_u64(42);
        let dwarf = Dwarf::new(1, &mut rng);

        assert_eq!(dwarf.id, 1);
        assert!(!dwarf.name.is_empty());
        assert_eq!(dwarf.health, 100);
        assert_eq!(dwarf.status, DwarfStatus::Idle);
    }

    #[test]
    fn test_needs_decay() {
        let mut needs = DwarfNeeds::new();
        needs.decay(5);

        assert!(needs.hunger < 100);
        assert!(needs.thirst < 100);
        assert!(needs.rest < 100);
    }

    #[test]
    fn test_mood_calculation() {
        // Start with all needs at maximum for Ecstatic mood
        let mut needs = DwarfNeeds {
            hunger: 100,
            thirst: 100,
            rest: 100,
            social: 100,
            comfort: 100,
        };
        assert_eq!(DwarfMood::from_needs(&needs), DwarfMood::Ecstatic);

        needs.hunger = 50;
        needs.thirst = 50;
        assert!(matches!(DwarfMood::from_needs(&needs), DwarfMood::Content | DwarfMood::Happy));

        needs.hunger = 10;
        needs.thirst = 10;
        needs.rest = 10;
        assert!(matches!(DwarfMood::from_needs(&needs), DwarfMood::Miserable | DwarfMood::Unhappy));
    }

    #[test]
    fn test_skill_improvement() {
        let mut skills = DwarfSkills::default();
        skills.improve("mining", 3);
        assert_eq!(skills.mining, 3);

        // Can't exceed 20
        skills.improve("mining", 25);
        assert_eq!(skills.mining, 20);
    }

    #[test]
    fn test_damage_and_healing() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut dwarf = Dwarf::new(1, &mut rng);

        dwarf.take_damage(30);
        assert_eq!(dwarf.health, 70);

        dwarf.heal(20);
        assert_eq!(dwarf.health, 90);
    }

    #[test]
    fn test_work_efficiency() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut dwarf = Dwarf::new(1, &mut rng);

        // Fresh dwarf should have high efficiency
        assert!(dwarf.work_efficiency() >= 80);

        // Drain needs
        dwarf.needs.hunger = 20;
        dwarf.needs.thirst = 20;
        dwarf.update_mood();

        // Efficiency should drop
        assert!(dwarf.work_efficiency() < 60);
    }
}
