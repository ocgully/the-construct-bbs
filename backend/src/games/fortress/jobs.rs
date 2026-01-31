//! Job system for Fortress
//!
//! Manages work orders, job assignments, and task completion.

use serde::{Serialize, Deserialize};

/// Type of job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobType {
    // Gathering
    Mine { x: u32, y: u32, z: u32 },
    Chop { x: u32, y: u32 },
    Farm { x: u32, y: u32, z: u32 },
    Gather { x: u32, y: u32, z: u32 },

    // Hauling
    Haul { from: (u32, u32, u32), to: (u32, u32, u32), item: String },
    Store { item: String, stockpile_id: u32 },

    // Crafting
    Craft { workshop_id: u32, recipe: String },

    // Construction
    Build { x: u32, y: u32, z: u32, building_type: String },
    Construct { x: u32, y: u32, z: u32, wall: bool },
    Furnish { x: u32, y: u32, z: u32, furniture: String },

    // Needs
    Eat,
    Drink,
    Sleep,
    Socialize,

    // Combat
    Fight { enemy_id: u32 },
    Patrol { path: Vec<(u32, u32, u32)> },
    Guard { x: u32, y: u32, z: u32 },

    // Other
    Heal { target_id: u32 },
    Clean { x: u32, y: u32, z: u32 },
}

impl JobType {
    /// Get the skill required for this job
    pub fn required_skill(&self) -> Option<&'static str> {
        match self {
            JobType::Mine { .. } => Some("mining"),
            JobType::Chop { .. } => Some("woodcutting"),
            JobType::Farm { .. } => Some("farming"),
            JobType::Craft { .. } => Some("crafting"),
            JobType::Build { .. } => Some("building"),
            JobType::Construct { .. } => Some("masonry"),
            JobType::Fight { .. } | JobType::Patrol { .. } | JobType::Guard { .. } => Some("combat"),
            JobType::Heal { .. } => Some("healing"),
            JobType::Haul { .. } | JobType::Store { .. } => Some("hauling"),
            _ => None,
        }
    }

    /// Get base work time in ticks
    pub fn base_work_time(&self) -> u32 {
        match self {
            JobType::Mine { .. } => 10,
            JobType::Chop { .. } => 15,
            JobType::Farm { .. } => 8,
            JobType::Gather { .. } => 5,
            JobType::Haul { .. } => 5,
            JobType::Store { .. } => 3,
            JobType::Craft { .. } => 20,
            JobType::Build { .. } => 30,
            JobType::Construct { .. } => 15,
            JobType::Furnish { .. } => 10,
            JobType::Eat => 5,
            JobType::Drink => 3,
            JobType::Sleep => 30,
            JobType::Socialize => 10,
            JobType::Fight { .. } => 1, // Combat is per-round
            JobType::Patrol { .. } => 20,
            JobType::Guard { .. } => 50,
            JobType::Heal { .. } => 15,
            JobType::Clean { .. } => 5,
        }
    }

    /// Get priority (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            JobType::Fight { .. } => 0,    // Combat is urgent
            JobType::Heal { .. } => 1,     // Healing injured
            JobType::Drink => 2,           // Thirst is dangerous
            JobType::Eat => 3,
            JobType::Sleep => 4,
            JobType::Guard { .. } => 5,
            JobType::Patrol { .. } => 6,
            JobType::Mine { .. } => 10,
            JobType::Chop { .. } => 10,
            JobType::Farm { .. } => 10,
            JobType::Craft { .. } => 15,
            JobType::Build { .. } => 15,
            JobType::Construct { .. } => 15,
            JobType::Haul { .. } => 20,
            JobType::Store { .. } => 20,
            _ => 30,
        }
    }

    /// Description for UI
    pub fn description(&self) -> String {
        match self {
            JobType::Mine { x, y, z } => format!("Mining at ({},{},{})", x, y, z),
            JobType::Chop { x, y } => format!("Chopping tree at ({},{})", x, y),
            JobType::Farm { x, y, z } => format!("Farming at ({},{},{})", x, y, z),
            JobType::Gather { x, y, z } => format!("Gathering at ({},{},{})", x, y, z),
            JobType::Haul { item, .. } => format!("Hauling {}", item),
            JobType::Store { item, .. } => format!("Storing {}", item),
            JobType::Craft { recipe, .. } => format!("Crafting {}", recipe),
            JobType::Build { building_type, .. } => format!("Building {}", building_type),
            JobType::Construct { wall, .. } => {
                if *wall { "Constructing wall".to_string() }
                else { "Constructing floor".to_string() }
            }
            JobType::Furnish { furniture, .. } => format!("Placing {}", furniture),
            JobType::Eat => "Eating".to_string(),
            JobType::Drink => "Drinking".to_string(),
            JobType::Sleep => "Sleeping".to_string(),
            JobType::Socialize => "Socializing".to_string(),
            JobType::Fight { .. } => "Fighting!".to_string(),
            JobType::Patrol { .. } => "Patrolling".to_string(),
            JobType::Guard { .. } => "Guarding".to_string(),
            JobType::Heal { .. } => "Healing".to_string(),
            JobType::Clean { .. } => "Cleaning".to_string(),
        }
    }
}

/// A single job instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u32,
    pub job_type: JobType,
    pub assigned_to: Option<u32>,  // Dwarf ID
    pub progress: u32,             // 0 to work_time
    pub work_time: u32,            // Total ticks needed
    pub created_at: i64,           // Game tick when created
    pub status: JobStatus,
    pub priority_override: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Blocked,  // Missing resources or access
}

impl Job {
    pub fn new(id: u32, job_type: JobType, created_at: i64) -> Self {
        let work_time = job_type.base_work_time();
        Self {
            id,
            job_type,
            assigned_to: None,
            progress: 0,
            work_time,
            created_at,
            status: JobStatus::Pending,
            priority_override: None,
        }
    }

    /// Get effective priority
    pub fn priority(&self) -> u8 {
        self.priority_override.unwrap_or_else(|| self.job_type.priority())
    }

    /// Advance progress by given ticks
    pub fn work(&mut self, ticks: u32) -> bool {
        self.progress = (self.progress + ticks).min(self.work_time);
        if self.progress >= self.work_time {
            self.status = JobStatus::Completed;
            true
        } else {
            false
        }
    }

    /// Get progress percentage
    pub fn progress_percent(&self) -> u8 {
        if self.work_time == 0 {
            return 100;
        }
        ((self.progress as u32 * 100) / self.work_time as u32) as u8
    }

    /// Check if job can be assigned
    pub fn can_assign(&self) -> bool {
        self.status == JobStatus::Pending && self.assigned_to.is_none()
    }

    /// Assign to a dwarf
    pub fn assign(&mut self, dwarf_id: u32) {
        self.assigned_to = Some(dwarf_id);
        self.status = JobStatus::InProgress;
    }

    /// Unassign from current dwarf
    pub fn unassign(&mut self) {
        self.assigned_to = None;
        self.status = JobStatus::Pending;
        // Keep progress for resumable jobs
    }
}

/// Work order - a repeating or batch job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: u32,
    pub name: String,
    pub recipe: String,
    pub workshop_id: u32,
    pub quantity: u32,
    pub completed: u32,
    pub repeat: bool,
    pub paused: bool,
    pub created_at: i64,
}

impl WorkOrder {
    pub fn new(id: u32, name: String, recipe: String, workshop_id: u32, quantity: u32, repeat: bool, created_at: i64) -> Self {
        Self {
            id,
            name,
            recipe,
            workshop_id,
            quantity,
            completed: 0,
            repeat,
            paused: false,
            created_at,
        }
    }

    /// Check if order is complete
    pub fn is_complete(&self) -> bool {
        !self.repeat && self.completed >= self.quantity
    }

    /// Get remaining quantity
    pub fn remaining(&self) -> u32 {
        if self.repeat {
            self.quantity // Always return full quantity for repeating
        } else {
            self.quantity.saturating_sub(self.completed)
        }
    }
}

/// Job queue manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobQueue {
    jobs: Vec<Job>,
    work_orders: Vec<WorkOrder>,
    next_job_id: u32,
    next_order_id: u32,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            work_orders: Vec::new(),
            next_job_id: 1,
            next_order_id: 1,
        }
    }

    /// Add a new job
    pub fn add_job(&mut self, job_type: JobType, current_tick: i64) -> u32 {
        let id = self.next_job_id;
        self.next_job_id += 1;

        let job = Job::new(id, job_type, current_tick);
        self.jobs.push(job);
        id
    }

    /// Add a work order
    pub fn add_work_order(&mut self, name: String, recipe: String, workshop_id: u32, quantity: u32, repeat: bool, current_tick: i64) -> u32 {
        let id = self.next_order_id;
        self.next_order_id += 1;

        let order = WorkOrder::new(id, name, recipe, workshop_id, quantity, repeat, current_tick);
        self.work_orders.push(order);
        id
    }

    /// Get a job by ID
    pub fn get_job(&self, id: u32) -> Option<&Job> {
        self.jobs.iter().find(|j| j.id == id)
    }

    /// Get mutable job by ID
    pub fn get_job_mut(&mut self, id: u32) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    /// Get next available job for a dwarf with given skills
    pub fn get_available_job(&self, skill_levels: &[(String, u8)]) -> Option<&Job> {
        // Sort by priority
        let mut available: Vec<_> = self.jobs.iter()
            .filter(|j| j.can_assign())
            .collect();

        available.sort_by_key(|j| j.priority());

        // Find first job the dwarf can do
        for job in available {
            if let Some(required) = job.job_type.required_skill() {
                let has_skill = skill_levels.iter()
                    .any(|(s, _)| s == required);
                if has_skill {
                    return Some(job);
                }
            } else {
                // No skill required
                return Some(job);
            }
        }

        None
    }

    /// Get all pending jobs
    pub fn pending_jobs(&self) -> Vec<&Job> {
        self.jobs.iter()
            .filter(|j| j.status == JobStatus::Pending)
            .collect()
    }

    /// Get all in-progress jobs
    pub fn in_progress_jobs(&self) -> Vec<&Job> {
        self.jobs.iter()
            .filter(|j| j.status == JobStatus::InProgress)
            .collect()
    }

    /// Get jobs assigned to a specific dwarf
    pub fn dwarf_jobs(&self, dwarf_id: u32) -> Vec<&Job> {
        self.jobs.iter()
            .filter(|j| j.assigned_to == Some(dwarf_id))
            .collect()
    }

    /// Remove completed jobs
    pub fn cleanup_completed(&mut self) {
        self.jobs.retain(|j| j.status != JobStatus::Completed);
    }

    /// Cancel a job
    pub fn cancel_job(&mut self, id: u32) -> bool {
        if let Some(job) = self.get_job_mut(id) {
            job.status = JobStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// Get active work orders
    pub fn active_work_orders(&self) -> Vec<&WorkOrder> {
        self.work_orders.iter()
            .filter(|o| !o.paused && !o.is_complete())
            .collect()
    }

    /// Get work order by ID
    pub fn get_work_order(&self, id: u32) -> Option<&WorkOrder> {
        self.work_orders.iter().find(|o| o.id == id)
    }

    /// Get mutable work order by ID
    pub fn get_work_order_mut(&mut self, id: u32) -> Option<&mut WorkOrder> {
        self.work_orders.iter_mut().find(|o| o.id == id)
    }

    /// Mark one unit completed on a work order
    pub fn complete_work_order_unit(&mut self, order_id: u32) {
        if let Some(order) = self.get_work_order_mut(order_id) {
            order.completed += 1;
        }
    }

    /// Toggle work order pause
    pub fn toggle_work_order_pause(&mut self, id: u32) -> bool {
        if let Some(order) = self.get_work_order_mut(id) {
            order.paused = !order.paused;
            true
        } else {
            false
        }
    }

    /// Count jobs by status
    pub fn job_counts(&self) -> (u32, u32, u32) {
        let pending = self.jobs.iter().filter(|j| j.status == JobStatus::Pending).count() as u32;
        let in_progress = self.jobs.iter().filter(|j| j.status == JobStatus::InProgress).count() as u32;
        let blocked = self.jobs.iter().filter(|j| j.status == JobStatus::Blocked).count() as u32;

        (pending, in_progress, blocked)
    }

    /// Create jobs from work orders
    pub fn process_work_orders(&mut self, current_tick: i64) {
        // Collect jobs to add first to avoid borrow conflict
        let jobs_to_add: Vec<JobType> = self.work_orders.iter()
            .filter(|order| !order.paused && !order.is_complete() && order.remaining() > 0)
            .filter_map(|order| {
                // Check if there's already a job for this workshop/recipe combo
                let existing = self.jobs.iter()
                    .filter(|j| j.status == JobStatus::Pending || j.status == JobStatus::InProgress)
                    .any(|j| {
                        if let JobType::Craft { workshop_id, recipe } = &j.job_type {
                            *workshop_id == order.workshop_id && *recipe == order.recipe
                        } else {
                            false
                        }
                    });

                if !existing {
                    Some(JobType::Craft {
                        workshop_id: order.workshop_id,
                        recipe: order.recipe.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Now add the jobs
        for job_type in jobs_to_add {
            self.add_job(job_type, current_tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::new(1, JobType::Mine { x: 5, y: 5, z: 1 }, 0);
        assert_eq!(job.id, 1);
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.can_assign());
    }

    #[test]
    fn test_job_progress() {
        let mut job = Job::new(1, JobType::Mine { x: 5, y: 5, z: 1 }, 0);
        job.assign(1);

        assert_eq!(job.status, JobStatus::InProgress);

        // Work on it
        let completed = job.work(5);
        assert!(!completed);

        // Complete it
        let completed = job.work(10);
        assert!(completed);
        assert_eq!(job.status, JobStatus::Completed);
    }

    #[test]
    fn test_job_queue() {
        let mut queue = JobQueue::new();

        let id1 = queue.add_job(JobType::Mine { x: 0, y: 0, z: 1 }, 0);
        let _id2 = queue.add_job(JobType::Haul { from: (0,0,0), to: (1,1,1), item: "stone".to_string() }, 0);

        assert_eq!(queue.pending_jobs().len(), 2);

        // Get available job with mining skill
        let skills = vec![("mining".to_string(), 5u8)];
        let job = queue.get_available_job(&skills);
        assert!(job.is_some());
        assert_eq!(job.unwrap().id, id1);
    }

    #[test]
    fn test_work_order() {
        let mut queue = JobQueue::new();

        queue.add_work_order(
            "Make Weapons".to_string(),
            "forge_weapon".to_string(),
            1,
            5,
            false,
            0,
        );

        let orders = queue.active_work_orders();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].remaining(), 5);
    }

    #[test]
    fn test_job_priority() {
        let fight = JobType::Fight { enemy_id: 1 };
        let mine = JobType::Mine { x: 0, y: 0, z: 0 };
        let haul = JobType::Haul { from: (0,0,0), to: (1,1,1), item: "x".to_string() };

        assert!(fight.priority() < mine.priority());
        assert!(mine.priority() < haul.priority());
    }
}
