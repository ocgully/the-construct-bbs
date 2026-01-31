//! Star Trader - Corporation System
//!
//! Player corporations for teamwork and territory control.

use serde::{Serialize, Deserialize};
use super::data::config;

/// Corporation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corporation {
    pub id: i64,
    pub name: String,
    pub tag: String,              // 3-5 character tag like [ABC]
    pub ceo_id: i64,              // Player who owns the corp
    pub members: Vec<CorpMember>,
    pub treasury: i64,            // Shared credits
    pub founded_date: String,
    pub planets_owned: Vec<u32>,  // Sector IDs with corp planets
    pub at_war_with: Vec<i64>,    // Corp IDs at war with
    pub allied_with: Vec<i64>,    // Corp IDs allied with
}

/// Corporation member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpMember {
    pub player_id: i64,
    pub handle: String,
    pub rank: CorpRank,
    pub contributed: i64,         // Total contributed to treasury
    pub joined_date: String,
}

/// Corporation rank
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorpRank {
    CEO,            // Full control
    Director,       // Can invite, kick members, declare war
    Commander,      // Can access planets, deploy defenses
    Member,         // Basic member
}

impl CorpRank {
    pub fn name(&self) -> &'static str {
        match self {
            CorpRank::CEO => "CEO",
            CorpRank::Director => "Director",
            CorpRank::Commander => "Commander",
            CorpRank::Member => "Member",
        }
    }
}

impl Corporation {
    /// Create a new corporation
    pub fn new(id: i64, name: String, tag: String, founder_id: i64, founder_handle: String) -> Self {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        Self {
            id,
            name,
            tag,
            ceo_id: founder_id,
            members: vec![CorpMember {
                player_id: founder_id,
                handle: founder_handle,
                rank: CorpRank::CEO,
                contributed: 0,
                joined_date: today.clone(),
            }],
            treasury: 0,
            founded_date: today,
            planets_owned: Vec::new(),
            at_war_with: Vec::new(),
            allied_with: Vec::new(),
        }
    }

    /// Get member count
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Check if corp is full
    pub fn is_full(&self) -> bool {
        self.members.len() >= config::MAX_CORP_SIZE
    }

    /// Check if player is a member
    pub fn is_member(&self, player_id: i64) -> bool {
        self.members.iter().any(|m| m.player_id == player_id)
    }

    /// Get member by ID
    pub fn get_member(&self, player_id: i64) -> Option<&CorpMember> {
        self.members.iter().find(|m| m.player_id == player_id)
    }

    /// Get member rank
    pub fn get_rank(&self, player_id: i64) -> Option<CorpRank> {
        self.get_member(player_id).map(|m| m.rank)
    }

    /// Check if player can invite members
    pub fn can_invite(&self, player_id: i64) -> bool {
        matches!(self.get_rank(player_id), Some(CorpRank::CEO) | Some(CorpRank::Director))
    }

    /// Check if player can kick members
    pub fn can_kick(&self, player_id: i64) -> bool {
        matches!(self.get_rank(player_id), Some(CorpRank::CEO) | Some(CorpRank::Director))
    }

    /// Check if player can manage planets
    pub fn can_manage_planets(&self, player_id: i64) -> bool {
        matches!(
            self.get_rank(player_id),
            Some(CorpRank::CEO) | Some(CorpRank::Director) | Some(CorpRank::Commander)
        )
    }

    /// Check if player can declare war
    pub fn can_declare_war(&self, player_id: i64) -> bool {
        matches!(self.get_rank(player_id), Some(CorpRank::CEO) | Some(CorpRank::Director))
    }

    /// Add a member
    pub fn add_member(&mut self, player_id: i64, handle: String) -> Result<(), String> {
        if self.is_full() {
            return Err(format!("Corporation is full ({} members max)", config::MAX_CORP_SIZE));
        }

        if self.is_member(player_id) {
            return Err("Player is already a member".to_string());
        }

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        self.members.push(CorpMember {
            player_id,
            handle,
            rank: CorpRank::Member,
            contributed: 0,
            joined_date: today,
        });

        Ok(())
    }

    /// Remove a member
    pub fn remove_member(&mut self, player_id: i64) -> Result<(), String> {
        if player_id == self.ceo_id {
            return Err("Cannot remove the CEO".to_string());
        }

        let idx = self.members.iter().position(|m| m.player_id == player_id);
        match idx {
            Some(i) => {
                self.members.remove(i);
                Ok(())
            }
            None => Err("Player is not a member".to_string()),
        }
    }

    /// Promote a member
    pub fn promote(&mut self, player_id: i64) -> Result<CorpRank, String> {
        let member = self.members.iter_mut()
            .find(|m| m.player_id == player_id)
            .ok_or("Player is not a member")?;

        let new_rank = match member.rank {
            CorpRank::CEO => return Err("Cannot promote the CEO".to_string()),
            CorpRank::Director => return Err("Director is the highest promotable rank".to_string()),
            CorpRank::Commander => CorpRank::Director,
            CorpRank::Member => CorpRank::Commander,
        };

        member.rank = new_rank;
        Ok(new_rank)
    }

    /// Demote a member
    pub fn demote(&mut self, player_id: i64) -> Result<CorpRank, String> {
        if player_id == self.ceo_id {
            return Err("Cannot demote the CEO".to_string());
        }

        let member = self.members.iter_mut()
            .find(|m| m.player_id == player_id)
            .ok_or("Player is not a member")?;

        let new_rank = match member.rank {
            CorpRank::CEO => return Err("Cannot demote the CEO".to_string()),
            CorpRank::Director => CorpRank::Commander,
            CorpRank::Commander => CorpRank::Member,
            CorpRank::Member => return Err("Already at lowest rank".to_string()),
        };

        member.rank = new_rank;
        Ok(new_rank)
    }

    /// Transfer CEO ownership
    pub fn transfer_ownership(&mut self, new_ceo_id: i64) -> Result<(), String> {
        if !self.is_member(new_ceo_id) {
            return Err("Player is not a member".to_string());
        }

        // Demote old CEO to Director
        if let Some(old_ceo) = self.members.iter_mut().find(|m| m.player_id == self.ceo_id) {
            old_ceo.rank = CorpRank::Director;
        }

        // Promote new CEO
        if let Some(new_ceo) = self.members.iter_mut().find(|m| m.player_id == new_ceo_id) {
            new_ceo.rank = CorpRank::CEO;
        }

        self.ceo_id = new_ceo_id;
        Ok(())
    }

    /// Deposit credits to treasury
    pub fn deposit(&mut self, player_id: i64, amount: i64) -> Result<(), String> {
        let member = self.members.iter_mut()
            .find(|m| m.player_id == player_id)
            .ok_or("Player is not a member")?;

        member.contributed += amount;
        self.treasury += amount;
        Ok(())
    }

    /// Withdraw credits from treasury (CEO/Director only)
    pub fn withdraw(&mut self, player_id: i64, amount: i64) -> Result<(), String> {
        if !self.can_kick(player_id) {
            return Err("Only CEO and Directors can withdraw".to_string());
        }

        if self.treasury < amount {
            return Err(format!("Treasury only has {} credits", self.treasury));
        }

        self.treasury -= amount;
        Ok(())
    }

    /// Declare war on another corporation
    pub fn declare_war(&mut self, target_corp_id: i64) -> Result<(), String> {
        if self.at_war_with.contains(&target_corp_id) {
            return Err("Already at war with this corporation".to_string());
        }

        // Remove alliance if exists
        self.allied_with.retain(|&id| id != target_corp_id);
        self.at_war_with.push(target_corp_id);
        Ok(())
    }

    /// Make peace with another corporation
    pub fn make_peace(&mut self, target_corp_id: i64) -> Result<(), String> {
        if !self.at_war_with.contains(&target_corp_id) {
            return Err("Not at war with this corporation".to_string());
        }

        self.at_war_with.retain(|&id| id != target_corp_id);
        Ok(())
    }

    /// Form alliance with another corporation
    pub fn form_alliance(&mut self, target_corp_id: i64) -> Result<(), String> {
        if self.at_war_with.contains(&target_corp_id) {
            return Err("Cannot ally with a corporation you're at war with".to_string());
        }

        if self.allied_with.contains(&target_corp_id) {
            return Err("Already allied with this corporation".to_string());
        }

        self.allied_with.push(target_corp_id);
        Ok(())
    }

    /// Break alliance
    pub fn break_alliance(&mut self, target_corp_id: i64) -> Result<(), String> {
        if !self.allied_with.contains(&target_corp_id) {
            return Err("Not allied with this corporation".to_string());
        }

        self.allied_with.retain(|&id| id != target_corp_id);
        Ok(())
    }

    /// Check if at war with another corp
    pub fn is_at_war_with(&self, corp_id: i64) -> bool {
        self.at_war_with.contains(&corp_id)
    }

    /// Check if allied with another corp
    pub fn is_allied_with(&self, corp_id: i64) -> bool {
        self.allied_with.contains(&corp_id)
    }
}

/// Result of creating a corporation
#[derive(Debug)]
pub struct CreateCorpResult {
    pub success: bool,
    pub corporation: Option<Corporation>,
    pub message: String,
}

/// Create a new corporation
pub fn create_corporation(
    corp_id: i64,
    name: String,
    tag: String,
    founder_id: i64,
    founder_handle: String,
    founder_credits: &mut i64,
) -> CreateCorpResult {
    let creation_cost = 100_000;  // 100k credits to create

    // Validate name
    if name.len() < 3 || name.len() > 30 {
        return CreateCorpResult {
            success: false,
            corporation: None,
            message: "Corporation name must be 3-30 characters".to_string(),
        };
    }

    // Validate tag
    if tag.len() < 2 || tag.len() > 5 {
        return CreateCorpResult {
            success: false,
            corporation: None,
            message: "Corporation tag must be 2-5 characters".to_string(),
        };
    }

    // Check credits
    if *founder_credits < creation_cost {
        return CreateCorpResult {
            success: false,
            corporation: None,
            message: format!("Need {} credits to create a corporation", creation_cost),
        };
    }

    *founder_credits -= creation_cost;

    CreateCorpResult {
        success: true,
        corporation: Some(Corporation::new(corp_id, name, tag, founder_id, founder_handle)),
        message: "Corporation created successfully!".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_corp() -> Corporation {
        Corporation::new(1, "Test Corp".to_string(), "TST".to_string(), 1, "CEO".to_string())
    }

    #[test]
    fn test_create_corporation() {
        let corp = create_test_corp();
        assert_eq!(corp.name, "Test Corp");
        assert_eq!(corp.member_count(), 1);
        assert_eq!(corp.ceo_id, 1);
    }

    #[test]
    fn test_add_member() {
        let mut corp = create_test_corp();
        assert!(corp.add_member(2, "Member1".to_string()).is_ok());
        assert_eq!(corp.member_count(), 2);
        assert!(corp.is_member(2));
    }

    #[test]
    fn test_max_members() {
        let mut corp = create_test_corp();
        for i in 2..=10 {
            corp.add_member(i, format!("Member{}", i)).unwrap();
        }
        assert!(corp.is_full());
        assert!(corp.add_member(11, "TooMany".to_string()).is_err());
    }

    #[test]
    fn test_remove_member() {
        let mut corp = create_test_corp();
        corp.add_member(2, "Member1".to_string()).unwrap();
        assert!(corp.remove_member(2).is_ok());
        assert!(!corp.is_member(2));
    }

    #[test]
    fn test_cannot_remove_ceo() {
        let mut corp = create_test_corp();
        assert!(corp.remove_member(1).is_err());
    }

    #[test]
    fn test_promote_demote() {
        let mut corp = create_test_corp();
        corp.add_member(2, "Member1".to_string()).unwrap();

        // Promote to Commander
        assert_eq!(corp.promote(2).unwrap(), CorpRank::Commander);
        assert_eq!(corp.get_rank(2), Some(CorpRank::Commander));

        // Promote to Director
        assert_eq!(corp.promote(2).unwrap(), CorpRank::Director);

        // Demote back
        assert_eq!(corp.demote(2).unwrap(), CorpRank::Commander);
    }

    #[test]
    fn test_treasury() {
        let mut corp = create_test_corp();
        corp.deposit(1, 1000).unwrap();
        assert_eq!(corp.treasury, 1000);

        corp.withdraw(1, 500).unwrap();
        assert_eq!(corp.treasury, 500);
    }

    #[test]
    fn test_war_and_alliance() {
        let mut corp = create_test_corp();

        // Declare war
        assert!(corp.declare_war(2).is_ok());
        assert!(corp.is_at_war_with(2));

        // Make peace
        assert!(corp.make_peace(2).is_ok());
        assert!(!corp.is_at_war_with(2));

        // Form alliance
        assert!(corp.form_alliance(2).is_ok());
        assert!(corp.is_allied_with(2));

        // Cannot ally and war at same time
        assert!(corp.declare_war(2).is_ok());
        assert!(!corp.is_allied_with(2));  // Alliance broken
    }

    #[test]
    fn test_transfer_ownership() {
        let mut corp = create_test_corp();
        corp.add_member(2, "NewCEO".to_string()).unwrap();

        assert!(corp.transfer_ownership(2).is_ok());
        assert_eq!(corp.ceo_id, 2);
        assert_eq!(corp.get_rank(2), Some(CorpRank::CEO));
        assert_eq!(corp.get_rank(1), Some(CorpRank::Director));
    }

    #[test]
    fn test_create_corporation_cost() {
        let mut credits = 150_000i64;
        let result = create_corporation(
            1,
            "Test Corp".to_string(),
            "TST".to_string(),
            1,
            "Founder".to_string(),
            &mut credits,
        );

        assert!(result.success);
        assert_eq!(credits, 50_000);  // 100k deducted
    }
}
