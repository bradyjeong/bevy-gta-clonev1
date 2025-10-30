use crate::components::world::EntityLimits;
use crate::components::{Building, Car, F16, Helicopter, NPCState, Yacht};
use bevy::log::info;
#[cfg(feature = "debug-ui")]
use bevy::log::warn;
use bevy::prelude::*;

type VehicleFilter = Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>)>;

/// System to enforce entity limits with FIFO cleanup
/// Replaces deleted EntityLimitManager service
pub fn enforce_entity_limits(
    mut commands: Commands,
    mut entity_limits: ResMut<EntityLimits>,
    time: Res<Time>,
    vehicle_query: Query<Entity, VehicleFilter>,
    building_query: Query<Entity, With<Building>>,
    npc_query: Query<Entity, With<NPCState>>,
    _tree_query: Query<Entity, With<crate::components::world::DynamicContent>>,
) {
    let current_time = time.elapsed_secs();

    // ONE-TIME INITIALIZATION: Populate tracking lists from existing entities
    // This handles entities spawned during world generation before this system runs
    if entity_limits.vehicle_entities.is_empty() && vehicle_query.iter().count() > 0 {
        #[cfg(feature = "debug-ui")]
        warn!("Initializing entity tracking from existing world entities...");

        // Populate vehicle tracking
        for entity in vehicle_query.iter() {
            entity_limits.vehicle_entities.push((entity, current_time));
        }

        // Populate building tracking
        for entity in building_query.iter() {
            entity_limits.building_entities.push((entity, current_time));
        }

        // Populate NPC tracking
        for entity in npc_query.iter() {
            entity_limits.npc_entities.push((entity, current_time));
        }

        #[cfg(feature = "debug-ui")]
        warn!(
            "Entity tracking initialized: {} vehicles, {} buildings, {} NPCs",
            entity_limits.vehicle_entities.len(),
            entity_limits.building_entities.len(),
            entity_limits.npc_entities.len()
        );
    }

    // Throttle limit checks to once per second (avoid frame-rate spam)
    const CHECK_INTERVAL: f32 = 1.0;
    if (current_time % CHECK_INTERVAL) >= (CHECK_INTERVAL - 0.016) {
        return; // Skip this frame
    }

    // Check and enforce vehicle limits
    let vehicle_count = vehicle_query.iter().count();
    if vehicle_count > entity_limits.max_vehicles {
        let excess = vehicle_count - entity_limits.max_vehicles;
        info!(
            "Vehicle limit exceeded: {vehicle_count}/{} (removing {excess} oldest)",
            entity_limits.max_vehicles
        );

        // Sort by spawn time (oldest first)
        entity_limits
            .vehicle_entities
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Despawn oldest excess vehicles (automatically recursive in Bevy 0.16)
        for i in 0..excess.min(entity_limits.vehicle_entities.len()) {
            if let Some((entity, _)) = entity_limits.vehicle_entities.get(i) {
                commands.entity(*entity).despawn();
            }
        }

        // Remove despawned entities from tracking
        let drain_count = excess.min(entity_limits.vehicle_entities.len());
        entity_limits.vehicle_entities.drain(0..drain_count);
    }

    // Check and enforce building limits
    let building_count = building_query.iter().count();
    if building_count > entity_limits.max_buildings {
        let excess = building_count - entity_limits.max_buildings;
        info!(
            "Building limit exceeded: {building_count}/{} (removing {excess} oldest)",
            entity_limits.max_buildings
        );

        entity_limits
            .building_entities
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for i in 0..excess.min(entity_limits.building_entities.len()) {
            if let Some((entity, _)) = entity_limits.building_entities.get(i) {
                commands.entity(*entity).despawn();
            }
        }

        let drain_count = excess.min(entity_limits.building_entities.len());
        entity_limits.building_entities.drain(0..drain_count);
    }

    // Check and enforce NPC limits
    let npc_count = npc_query.iter().count();
    if npc_count > entity_limits.max_npcs {
        let excess = npc_count - entity_limits.max_npcs;
        info!(
            "NPC limit exceeded: {npc_count}/{} (removing {excess} oldest)",
            entity_limits.max_npcs
        );

        entity_limits
            .npc_entities
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for i in 0..excess.min(entity_limits.npc_entities.len()) {
            if let Some((entity, _)) = entity_limits.npc_entities.get(i) {
                commands.entity(*entity).despawn();
            }
        }

        let drain_count = excess.min(entity_limits.npc_entities.len());
        entity_limits.npc_entities.drain(0..drain_count);
    }

    // Periodic cleanup: Remove invalid entities from tracking lists
    if (current_time % 30.0) < 0.1 {
        // Clean up every 30 seconds
        entity_limits
            .vehicle_entities
            .retain(|(entity, _)| vehicle_query.get(*entity).is_ok());
        entity_limits
            .building_entities
            .retain(|(entity, _)| building_query.get(*entity).is_ok());
        entity_limits
            .npc_entities
            .retain(|(entity, _)| npc_query.get(*entity).is_ok());
    }
}

/// AUTO-REGISTRATION SYSTEM (Bug #6 Fix)
/// Automatically tracks newly spawned entities without manual calls
#[allow(clippy::too_many_arguments)]
pub fn auto_register_spawned_entities(
    mut entity_limits: ResMut<EntityLimits>,
    time: Res<Time>,
    new_cars: Query<Entity, Added<Car>>,
    new_helicopters: Query<Entity, Added<Helicopter>>,
    new_f16s: Query<Entity, Added<F16>>,
    new_yachts: Query<Entity, Added<Yacht>>,
    new_buildings: Query<Entity, Added<Building>>,
    new_npcs: Query<Entity, Added<NPCState>>,
) {
    let current_time = time.elapsed_secs();

    for entity in new_cars
        .iter()
        .chain(new_helicopters.iter())
        .chain(new_f16s.iter())
        .chain(new_yachts.iter())
    {
        entity_limits.vehicle_entities.push((entity, current_time));
    }

    for entity in new_buildings.iter() {
        entity_limits.building_entities.push((entity, current_time));
    }

    for entity in new_npcs.iter() {
        entity_limits.npc_entities.push((entity, current_time));
    }
}

#[deprecated(note = "Use auto_register_spawned_entities system instead")]
pub fn track_spawned_entity(
    entity_limits: &mut EntityLimits,
    entity: Entity,
    spawn_time: f32,
    entity_type: TrackedEntityType,
) {
    match entity_type {
        TrackedEntityType::Vehicle => entity_limits.vehicle_entities.push((entity, spawn_time)),
        TrackedEntityType::Building => entity_limits.building_entities.push((entity, spawn_time)),
        TrackedEntityType::NPC => entity_limits.npc_entities.push((entity, spawn_time)),
        TrackedEntityType::Tree => entity_limits.tree_entities.push((entity, spawn_time)),
    }
}

#[allow(dead_code)]
pub enum TrackedEntityType {
    Vehicle,
    Building,
    NPC,
    Tree,
}
