use crate::components::player::ActiveEntity;
use bevy::prelude::*;

/// Minimal simulation LOD system - only handles expensive CPU work throttling
/// Rendering LOD is handled by Bevy's VisibilityRange for optimal performance
///
/// This system follows AGENT.md "simplicity first" - single responsibility only

#[derive(Component, Default, PartialEq, Eq, Debug)]
pub enum SimulationLOD {
    #[default]
    High, // Full simulation - AI, physics, detailed behavior
    Low, // Throttled simulation - minimal updates only
}

#[derive(Component, Debug)]
pub struct SimulationLODRadius(pub f32);

impl SimulationLODRadius {
    pub fn new(radius: f32) -> Self {
        Self(radius)
    }

    // Predefined distances for different entity types
    pub const NPC_RADIUS: f32 = 120.0; // NPCs get full AI within 120m
    pub const VEHICLE_RADIUS: f32 = 150.0; // Vehicles get full physics within 150m  
    pub const BUILDING_RADIUS: f32 = 200.0; // Building systems within 200m
    pub const VEGETATION_RADIUS: f32 = 100.0; // Vegetation animations within 100m
}

/// Timer resource for SimulationLOD updates - replaces unsafe static mut
#[derive(Resource)]
pub struct SimulationLODTimer(pub Timer);

impl Default for SimulationLODTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.25, TimerMode::Repeating))
    }
}

/// Lightweight simulation LOD update system
/// Runs every 0.25s (not every frame) for optimal performance
/// Only updates simulation level, never touches rendering or meshes
pub fn update_simulation_lod(
    mut simulation_query: Query<(&Transform, &SimulationLODRadius, &mut SimulationLOD)>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut timer: ResMut<SimulationLODTimer>,
    time: Res<Time>,
) {
    // Only update every 250ms to reduce CPU overhead
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let active_pos = active_transform.translation;

    // Update simulation LOD for all entities
    for (transform, radius, mut sim_lod) in &mut simulation_query {
        let distance = active_pos.distance(transform.translation);

        let desired_lod = if distance <= radius.0 {
            SimulationLOD::High
        } else {
            SimulationLOD::Low
        };

        // Only trigger change detection if LOD actually changed
        if *sim_lod != desired_lod {
            *sim_lod = desired_lod;
        }
    }
}

/// Helper function to check if entity should run expensive simulation
pub fn should_simulate_high_detail(sim_lod: &SimulationLOD) -> bool {
    matches!(sim_lod, SimulationLOD::High)
}

/// System to demonstrate how other systems can use SimulationLOD
/// This is an example - actual systems would check SimulationLOD before expensive work
pub fn example_ai_system(query: Query<(Entity, &Transform, &SimulationLOD)>) {
    for (_entity, _transform, sim_lod) in &query {
        match sim_lod {
            SimulationLOD::High => {
                // Full AI processing, pathfinding, behavior trees, etc.
                // This is where you'd normally do expensive AI work
            }
            SimulationLOD::Low => {
                // Minimal updates only - maybe just basic state preservation
                // Skip expensive AI, physics, animation calculations
            }
        }
    }
}

/// Resource to track simulation performance (optional)
#[derive(Resource, Default)]
pub struct SimulationLODStats {
    pub high_detail_entities: usize,
    pub low_detail_entities: usize,
    pub total_entities: usize,
    pub last_update: f32,
}

/// Optional system to track simulation LOD statistics
pub fn track_simulation_lod_stats(
    mut stats: ResMut<SimulationLODStats>,
    query: Query<&SimulationLOD>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Update stats every second
    if current_time - stats.last_update < 1.0 {
        return;
    }

    let mut high_count = 0;
    let mut low_count = 0;

    for sim_lod in &query {
        match sim_lod {
            SimulationLOD::High => high_count += 1,
            SimulationLOD::Low => low_count += 1,
        }
    }

    stats.high_detail_entities = high_count;
    stats.low_detail_entities = low_count;
    stats.total_entities = high_count + low_count;
    stats.last_update = current_time;
}
