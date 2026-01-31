//! Tick system for Fortress simulation
//!
//! Implements hybrid tick: background for active, catchup for inactive.

use super::state::{GameState, Enemy, Invasion};
use super::dwarves::DwarfStatus;
use super::jobs::JobType;
use super::terrain::TileType;
use super::data::{get_recipe, get_enemy};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Result of processing a tick
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    pub tick: i64,
    pub resources_gathered: Vec<(String, u32)>,
    pub items_crafted: Vec<(String, u32)>,
    pub jobs_completed: u32,
    pub dwarves_fed: u32,
    pub invasion_started: bool,
    pub invasion_repelled: bool,
    pub dwarves_died: Vec<String>,
    pub migrants_arrived: u32,
}

/// Process a single game tick
pub fn process_tick(state: &mut GameState) -> TickResult {
    let mut result = TickResult::default();
    result.tick = state.tick;

    let mut rng = StdRng::seed_from_u64(state.embark_seed.wrapping_add(state.tick as u64));

    // 1. Process dwarf needs
    process_dwarf_needs(state, &mut result);

    // 2. Process jobs
    process_jobs(state, &mut result, &mut rng);

    // 3. Assign new jobs to idle dwarves
    assign_jobs(state);

    // 4. Process work orders
    state.job_queue.process_work_orders(state.tick);

    // 5. Process combat
    process_combat(state, &mut result, &mut rng);

    // 6. Check for invasions
    maybe_trigger_invasion(state, &mut result, &mut rng);

    // 7. Check for migrations
    maybe_trigger_migration(state, &mut result, &mut rng);

    // 8. Farm production
    process_farms(state, &mut result, &mut rng);

    // 9. Update time
    state.tick += 1;

    // Season change every 100 ticks
    if state.tick % 100 == 0 {
        state.season = ((state.season + 1) % 4) as u8;
        if state.season == 0 {
            state.year += 1;
            state.notify(format!("Year {} has begun!", state.year));
        }
    }

    // 10. Cleanup
    state.job_queue.cleanup_completed();

    result
}

/// Process multiple ticks (catchup for inactive players)
pub fn process_catchup(state: &mut GameState, ticks: u32) -> TickResult {
    let mut combined = TickResult::default();

    for _ in 0..ticks {
        let result = process_tick(state);

        combined.tick = result.tick;
        for (res, amt) in result.resources_gathered {
            if let Some((_, existing)) = combined.resources_gathered.iter_mut()
                .find(|(r, _)| *r == res) {
                *existing += amt;
            } else {
                combined.resources_gathered.push((res, amt));
            }
        }
        for (item, amt) in result.items_crafted {
            if let Some((_, existing)) = combined.items_crafted.iter_mut()
                .find(|(i, _)| *i == item) {
                *existing += amt;
            } else {
                combined.items_crafted.push((item, amt));
            }
        }
        combined.jobs_completed += result.jobs_completed;
        combined.dwarves_fed += result.dwarves_fed;
        combined.invasion_started |= result.invasion_started;
        combined.invasion_repelled |= result.invasion_repelled;
        combined.dwarves_died.extend(result.dwarves_died);
        combined.migrants_arrived += result.migrants_arrived;
    }

    combined
}

fn process_dwarf_needs(state: &mut GameState, result: &mut TickResult) {
    for dwarf in &mut state.dwarves {
        if dwarf.status == DwarfStatus::Dead {
            continue;
        }

        // Tick dwarf (decay needs)
        dwarf.tick();

        // Handle critical needs
        if dwarf.needs.hunger < 20 && dwarf.status != DwarfStatus::Eating {
            // Try to eat
            if state.resources.meal > 0 || state.resources.meat > 0 {
                if state.resources.meal > 0 {
                    state.resources.meal -= 1;
                    dwarf.eat(15); // Good quality meal
                } else if state.resources.meat > 0 {
                    state.resources.meat -= 1;
                    dwarf.eat(10);
                }
                result.dwarves_fed += 1;
                state.stats.food_consumed += 1;
            }
        }

        if dwarf.needs.thirst < 20 && dwarf.status != DwarfStatus::Drinking {
            // Try to drink (prefer alcohol!)
            if state.resources.ale > 0 {
                state.resources.ale -= 1;
                dwarf.drink(20); // Dwarves love ale
                state.stats.drinks_consumed += 1;
            } else if state.resources.water > 0 {
                state.resources.water -= 1;
                dwarf.drink(10);
                state.stats.drinks_consumed += 1;
            }
        }

        if dwarf.needs.rest < 20 && dwarf.status != DwarfStatus::Sleeping {
            // Find a bed
            let has_bed = dwarf.room_id.is_some();
            dwarf.sleep(if has_bed { 15 } else { 8 });
        }

        // Check for death
        if dwarf.status == DwarfStatus::Dead {
            result.dwarves_died.push(dwarf.name.clone());
            state.stats.dwarves_lost += 1;
        }
    }
}

fn process_jobs(state: &mut GameState, result: &mut TickResult, rng: &mut StdRng) {
    let in_progress: Vec<u32> = state.job_queue.in_progress_jobs()
        .iter()
        .map(|j| j.id)
        .collect();

    for job_id in in_progress {
        // Get dwarf assigned to job
        let dwarf_id = if let Some(job) = state.job_queue.get_job(job_id) {
            job.assigned_to
        } else {
            continue;
        };

        let Some(dwarf_id) = dwarf_id else { continue };

        // Get dwarf efficiency
        let efficiency = state.get_dwarf(dwarf_id)
            .map(|d| d.work_efficiency())
            .unwrap_or(0);

        if efficiency == 0 {
            continue; // Dwarf can't work right now
        }

        // Work on job
        let work_ticks = (efficiency as u32).max(1) / 10;
        let completed = if let Some(job) = state.job_queue.get_job_mut(job_id) {
            job.work(work_ticks.max(1))
        } else {
            false
        };

        if completed {
            result.jobs_completed += 1;

            // Get job info first to avoid borrow conflict
            let (job_type, required_skill) = if let Some(job) = state.job_queue.get_job(job_id) {
                (job.job_type.clone(), job.job_type.required_skill())
            } else {
                continue;
            };

            // Handle job completion
            complete_job(state, &job_type, result, rng);

            // Update dwarf
            if let Some(skill) = required_skill {
                if let Some(dwarf) = state.get_dwarf_mut(dwarf_id) {
                    dwarf.complete_job(skill);
                }
            }
        }
    }
}

fn complete_job(state: &mut GameState, job_type: &JobType, result: &mut TickResult, _rng: &mut StdRng) {
    match job_type {
        JobType::Mine { x, y, z } => {
            if let Some(resources) = state.terrain.dig(*x, *y, *z) {
                for (resource, amount) in resources {
                    state.resources.add(resource, amount);
                    result.resources_gathered.push((resource.to_string(), amount));
                }
                state.stats.tiles_mined += 1;
            }
        }

        JobType::Chop { x, y } => {
            // Check if there's a tree
            if let Some(tile) = state.terrain.get(*x, *y, 0) {
                if tile.tile_type == TileType::Tree {
                    if let Some(tile) = state.terrain.get_mut(*x, *y, 0) {
                        tile.tile_type = TileType::Grass;
                    }
                    state.resources.add("wood", 5);
                    result.resources_gathered.push(("wood".to_string(), 5));
                    state.stats.trees_chopped += 1;
                }
            }
        }

        JobType::Farm { x: _, y: _, z: _ } => {
            // Farming produces food (handled in process_farms)
        }

        JobType::Craft { workshop_id: _, recipe } => {
            if let Some(recipe_def) = get_recipe(recipe) {
                // Check inputs
                let inputs: Vec<(&str, u32)> = recipe_def.inputs.iter()
                    .map(|(r, a)| (*r, *a))
                    .collect();

                if state.resources.consume(&inputs) {
                    // Produce outputs
                    for (output, amount) in recipe_def.outputs {
                        state.resources.add(output, *amount);
                        result.items_crafted.push((output.to_string(), *amount));
                    }
                    state.stats.items_crafted += 1;
                }
            }
        }

        JobType::Build { x, y, z, building_type } => {
            state.buildings.push(super::state::Building {
                id: state.next_building_id,
                building_type: building_type.clone(),
                x: *x,
                y: *y,
                z: *z,
                width: 1,
                height: 1,
            });
            state.next_building_id += 1;
        }

        JobType::Construct { x, y, z, wall } => {
            if *wall {
                state.terrain.build_wall(*x, *y, *z);
            }
        }

        _ => {}
    }
}

fn assign_jobs(state: &mut GameState) {
    // Get idle dwarves
    let idle_dwarves: Vec<u32> = state.dwarves.iter()
        .filter(|d| d.can_work())
        .map(|d| d.id)
        .collect();

    for dwarf_id in idle_dwarves {
        // Get dwarf skills
        let skills: Vec<(String, u8)> = if let Some(dwarf) = state.get_dwarf(dwarf_id) {
            vec![
                ("mining".to_string(), dwarf.skills.mining),
                ("woodcutting".to_string(), dwarf.skills.woodcutting),
                ("farming".to_string(), dwarf.skills.farming),
                ("crafting".to_string(), dwarf.skills.crafting),
                ("cooking".to_string(), dwarf.skills.cooking),
                ("building".to_string(), dwarf.skills.building),
                ("combat".to_string(), dwarf.skills.combat),
                ("hauling".to_string(), dwarf.skills.hauling),
                ("masonry".to_string(), dwarf.skills.masonry),
                ("smithing".to_string(), dwarf.skills.smithing),
                ("brewing".to_string(), dwarf.skills.brewing),
                ("healing".to_string(), dwarf.skills.healing),
            ]
        } else {
            continue;
        };

        // Find available job
        if let Some(job) = state.job_queue.get_available_job(&skills) {
            let job_id = job.id;
            if let Some(job) = state.job_queue.get_job_mut(job_id) {
                job.assign(dwarf_id);
            }
            if let Some(dwarf) = state.get_dwarf_mut(dwarf_id) {
                dwarf.assign_job(job_id);
            }
        }
    }
}

fn process_combat(state: &mut GameState, result: &mut TickResult, _rng: &mut StdRng) {
    // Get soldiers and their combat power
    let soldiers: Vec<(u32, u32)> = state.dwarves.iter()
        .filter(|d| d.status == DwarfStatus::Fighting && d.health > 0)
        .map(|d| (d.id, d.combat_power()))
        .collect();

    // Collect combat actions to apply later
    let mut dwarf_damages: Vec<(u32, u32)> = Vec::new();
    let mut enemies_killed = 0u32;
    let mut loot_drops: Vec<(&'static str, u32)> = Vec::new();

    for invasion in &mut state.invasions {
        for enemy in &mut invasion.enemies {
            if enemy.health == 0 {
                continue;
            }

            // Find a soldier to fight
            if let Some(&(soldier_id, dwarf_power)) = soldiers.first() {
                let enemy_def = get_enemy(&enemy.enemy_type);
                let enemy_attack = enemy_def.map(|e| e.attack).unwrap_or(5);
                let enemy_defense = enemy_def.map(|e| e.defense).unwrap_or(2);

                // Combat round
                let dwarf_damage = (dwarf_power as i32 - enemy_defense as i32).max(1) as u32;
                let enemy_damage = (enemy_attack as i32 - 2).max(1) as u32; // Armor provides some protection

                enemy.health = enemy.health.saturating_sub(dwarf_damage);
                dwarf_damages.push((soldier_id, enemy_damage));

                if enemy.health == 0 {
                    enemies_killed += 1;

                    // Drop loot
                    if let Some(enemy_def) = get_enemy(&enemy.enemy_type) {
                        for &(resource, amount) in enemy_def.loot {
                            loot_drops.push((resource, amount));
                        }
                    }
                }
            }
        }

        // Remove dead enemies
        invasion.enemies.retain(|e| e.health > 0);
    }

    // Apply dwarf damages
    for (soldier_id, damage) in dwarf_damages {
        if let Some(dwarf) = state.get_dwarf_mut(soldier_id) {
            dwarf.take_damage(damage);

            if dwarf.status == DwarfStatus::Dead {
                result.dwarves_died.push(dwarf.name.clone());
                state.stats.dwarves_lost += 1;
            }
        }
    }

    // Apply enemy kills and loot
    state.stats.enemies_slain += enemies_killed;
    for (resource, amount) in loot_drops {
        state.resources.add(&resource, amount);
    }

    // Check if invasions are repelled
    let repelled: Vec<u32> = state.invasions.iter()
        .filter(|i| i.enemies.is_empty() && i.waves_remaining == 0)
        .map(|i| i.id)
        .collect();

    for id in repelled {
        state.invasions.retain(|i| i.id != id);
        result.invasion_repelled = true;
        state.stats.invasions_repelled += 1;
        state.notify("The invasion has been repelled!".to_string());
    }
}

fn maybe_trigger_invasion(state: &mut GameState, result: &mut TickResult, rng: &mut StdRng) {
    // Don't trigger if already under siege
    if state.under_siege() {
        return;
    }

    // Invasion chance based on wealth and time
    let wealth_factor = (state.fortress_value() / 10000) as u32;
    let time_factor = state.tick as u32 / 500;
    let invasion_chance = wealth_factor + time_factor;

    if rng.gen_range(0..1000) < invasion_chance {
        // Determine invasion type based on threat level
        let threat = (wealth_factor / 5).min(4) + 1;
        let (enemy_type, count) = match threat {
            1 => ("goblin", rng.gen_range(3..8)),
            2 => ("goblin_warrior", rng.gen_range(5..12)),
            3 => ("troll", rng.gen_range(2..5)),
            4 => ("forgotten_beast", 1),
            _ => ("goblin", 5),
        };

        let enemy_def = get_enemy(enemy_type).unwrap();
        let mut enemies = Vec::new();

        for _ in 0..count {
            enemies.push(Enemy {
                id: state.next_enemy_id,
                enemy_type: enemy_type.to_string(),
                health: enemy_def.health,
                max_health: enemy_def.health,
                x: 0,
                y: 0,
                z: 0,
            });
            state.next_enemy_id += 1;
        }

        state.invasions.push(Invasion {
            id: state.next_enemy_id,
            enemy_type: enemy_type.to_string(),
            enemies,
            started_at: state.tick,
            waves_remaining: 0,
        });
        state.next_enemy_id += 1;

        result.invasion_started = true;
        state.notify(format!("{}s are attacking!", enemy_def.name));

        // Set soldiers to fighting
        for dwarf in &mut state.dwarves {
            if dwarf.skills.combat >= 3 && dwarf.status == DwarfStatus::Idle {
                dwarf.status = DwarfStatus::Fighting;
            }
        }
    }
}

fn maybe_trigger_migration(state: &mut GameState, result: &mut TickResult, rng: &mut StdRng) {
    // Migrations happen seasonally based on fortress conditions
    if state.tick % 100 != 50 {
        return; // Check mid-season
    }

    // Good conditions attract migrants
    let food_score = state.resources.meal + state.resources.meat;
    let drink_score = state.resources.ale + state.resources.wine + state.resources.water;
    let wealth_score = (state.fortress_value() / 1000) as u32;

    let migration_chance = food_score + drink_score + wealth_score;

    if rng.gen_range(0..500) < migration_chance {
        let migrant_count = rng.gen_range(1..4);
        for _ in 0..migrant_count {
            state.add_dwarf();
        }
        result.migrants_arrived = migrant_count;
        state.notify(format!("{} migrants have arrived!", migrant_count));
    }
}

fn process_farms(state: &mut GameState, result: &mut TickResult, rng: &mut StdRng) {
    // Count farm tiles
    let farm_count = state.terrain.count_tiles(0, TileType::Farm);

    if farm_count == 0 {
        return;
    }

    // Produce food based on farms and season
    let season_modifier = match state.season {
        0 => 50,  // Spring - planting
        1 => 100, // Summer - growing
        2 => 150, // Autumn - harvest
        3 => 20,  // Winter - minimal
        _ => 100,
    };

    let base_production = farm_count;
    let production = (base_production * season_modifier) / 100;

    if production > 0 && rng.gen_bool(0.5) {
        let food_type = match rng.gen_range(0..4) {
            0 => "vegetable",
            1 => "grain",
            2 => "plump_helmet",
            _ => "plant_fiber",
        };

        state.resources.add(food_type, production);
        result.resources_gathered.push((food_type.to_string(), production));
    }
}

/// Calculate ticks since last update
pub fn ticks_since_last_update(state: &GameState, current_time: i64) -> u32 {
    // Assume 1 tick per second in real time when offline
    let elapsed = current_time - state.tick;
    elapsed.max(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tick() {
        let mut state = GameState::new("Test".to_string(), 42);
        let initial_tick = state.tick;

        let result = process_tick(&mut state);

        assert_eq!(state.tick, initial_tick + 1);
        assert_eq!(result.tick, initial_tick);
    }

    #[test]
    fn test_catchup_ticks() {
        let mut state = GameState::new("Test".to_string(), 42);
        let initial_tick = state.tick;

        let result = process_catchup(&mut state, 10);

        assert_eq!(state.tick, initial_tick + 10);
        assert_eq!(result.tick, initial_tick + 9);
    }

    #[test]
    fn test_dwarf_feeding() {
        let mut state = GameState::new("Test".to_string(), 42);

        // Make dwarf hungry
        state.dwarves[0].needs.hunger = 10;
        state.resources.meal = 5;

        let result = process_tick(&mut state);

        assert!(result.dwarves_fed > 0 || state.dwarves[0].needs.hunger > 10);
    }

    #[test]
    fn test_season_change() {
        let mut state = GameState::new("Test".to_string(), 42);
        state.tick = 99;

        process_tick(&mut state);

        assert_eq!(state.tick, 100);
        assert_eq!(state.season, 1); // Should be Summer now
    }

    #[test]
    fn test_year_change() {
        let mut state = GameState::new("Test".to_string(), 42);
        state.tick = 399;
        state.season = 3;

        process_tick(&mut state);

        assert_eq!(state.year, 2);
        assert_eq!(state.season, 0);
    }
}
