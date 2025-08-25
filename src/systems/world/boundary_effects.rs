#![allow(clippy::type_complexity, clippy::too_many_arguments)]
use bevy::prelude::*;
use rand::Rng;

use crate::components::player::ActiveEntity;
use crate::components::vehicles::{VehicleHealth, VehicleState};
use crate::components::world::{BoundaryEffects, BoundaryVehicleType, BoundaryZone, WorldBounds};
use crate::resources::WorldRng;

/// GTA-style context-aware boundary system
/// Each vehicle type gets natural-feeling boundary effects instead of invisible walls
pub fn boundary_effects_system(
    mut commands: Commands,
    world_bounds: Res<WorldBounds>,
    mut vehicle_query: Query<
        (
            Entity,
            &mut Transform,
            &mut VehicleState,
            &mut BoundaryEffects,
            Option<&mut VehicleHealth>,
        ),
        With<ActiveEntity>,
    >,
    mut world_rng: ResMut<WorldRng>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut vehicle_state, mut boundary_effects, vehicle_health) in
        &mut vehicle_query
    {
        let position = transform.translation;

        // Determine vehicle type from VehicleState
        let vehicle_type = get_boundary_vehicle_type(&vehicle_state);

        // Get boundary zone and update effects component
        let boundary_zone = world_bounds.get_boundary_zone(position, vehicle_type);
        boundary_effects.current_zone = boundary_zone;
        boundary_effects.vehicle_type = vehicle_type;

        // Calculate effect intensity based on distance to edge
        let distance_to_edge = world_bounds.distance_to_nearest_edge(position);
        boundary_effects.effect_intensity = match boundary_zone {
            BoundaryZone::Safe => 0.0,
            BoundaryZone::Warning => 1.0 - (distance_to_edge / world_bounds.warning_zone_size),
            BoundaryZone::Critical => 1.0 - (distance_to_edge / world_bounds.critical_zone_size),
            BoundaryZone::OutOfBounds => 1.0,
        };

        // Apply context-specific boundary effects
        match vehicle_type {
            BoundaryVehicleType::Aircraft => {
                apply_aircraft_boundary_effects(
                    entity,
                    &mut commands,
                    &mut transform,
                    &mut vehicle_state,
                    &boundary_effects,
                    vehicle_health,
                    &time,
                    &mut world_rng,
                );
            }
            BoundaryVehicleType::Boat => {
                apply_boat_boundary_effects(
                    entity,
                    &mut commands,
                    &mut transform,
                    &mut vehicle_state,
                    &boundary_effects,
                    vehicle_health,
                    &time,
                    &mut world_rng,
                );
            }
            BoundaryVehicleType::OnFoot => {
                apply_onfoot_boundary_effects(
                    entity,
                    &mut commands,
                    &mut transform,
                    &mut vehicle_state,
                    &boundary_effects,
                    vehicle_health,
                    &time,
                    &mut world_rng,
                );
            }
            BoundaryVehicleType::GroundVehicle => {
                apply_ground_vehicle_boundary_effects(
                    entity,
                    &mut commands,
                    &mut transform,
                    &mut vehicle_state,
                    &boundary_effects,
                    vehicle_health,
                    &time,
                    &mut world_rng,
                );
            }
            BoundaryVehicleType::Submarine => {
                apply_submarine_boundary_effects(
                    entity,
                    &mut commands,
                    &mut transform,
                    &mut vehicle_state,
                    &boundary_effects,
                    vehicle_health,
                    &time,
                    &mut world_rng,
                );
            }
        }

        // Handle out-of-bounds teleport as last resort
        if matches!(boundary_zone, BoundaryZone::OutOfBounds) {
            // Emergency teleport to safe zone
            let safe_pos = world_bounds.safe_respawn_position();
            transform.translation = safe_pos;
            info!(
                "Emergency teleport: {} moved from {:?} to safe zone {:?}",
                get_vehicle_type_name(vehicle_type),
                position,
                safe_pos
            );
        }
    }
}

/// Aircraft boundary effects - mechanical failure and wing damage
fn apply_aircraft_boundary_effects(
    _entity: Entity,
    _commands: &mut Commands,
    transform: &mut Transform,
    vehicle_state: &mut VehicleState,
    boundary_effects: &BoundaryEffects,
    mut vehicle_health: Option<Mut<VehicleHealth>>,
    time: &Res<Time>,
    world_rng: &mut WorldRng,
) {
    let intensity = boundary_effects.effect_intensity;

    match boundary_effects.current_zone {
        BoundaryZone::Warning => {
            // Engine performance degradation - reduce max speed
            let original_speed = vehicle_state.max_speed;
            vehicle_state.max_speed = original_speed * (1.0 - intensity * 0.3); // Up to 30% speed loss

            // Occasional warning messages
            if world_rng.global().gen_bool(0.01 * intensity as f64) {
                info!("Aircraft warning: Engine performance degrading near world boundary");
            }
        }
        BoundaryZone::Critical => {
            // Severe mechanical failure - significant speed and acceleration loss
            let original_speed = vehicle_state.max_speed;
            let original_acceleration = vehicle_state.acceleration;
            vehicle_state.max_speed = original_speed * (1.0 - intensity * 0.7); // Up to 70% speed loss
            vehicle_state.acceleration = original_acceleration * (1.0 - intensity * 0.5); // Reduced acceleration

            // Increase damage over time
            vehicle_state.damage += 10.0 * intensity * time.delta_secs();

            // Damage VehicleHealth if available
            if let Some(ref mut health) = vehicle_health {
                health.current -= 10.0 * intensity * time.delta_secs();
                health.current = health.current.max(1.0); // Don't kill immediately
            }

            // Random complete failures
            if world_rng.global().gen_bool(0.02 * intensity as f64) {
                vehicle_state.max_speed *= 0.1; // Near-complete engine failure
                info!("Aircraft critical: Engine failure at world boundary!");
            }
        }
        BoundaryZone::OutOfBounds => {
            // Complete system failure - aircraft becomes nearly uncontrollable
            vehicle_state.max_speed *= 0.1;
            vehicle_state.acceleration *= 0.1;
            vehicle_state.damage += 50.0 * time.delta_secs(); // Rapid damage accumulation

            // Force descent
            let descent_force = Vec3::new(0.0, -50.0 * intensity, 0.0);
            transform.translation += descent_force * time.delta_secs();
        }
        _ => {}
    }
}

/// Boat boundary effects - rough seas and weather
fn apply_boat_boundary_effects(
    _entity: Entity,
    _commands: &mut Commands,
    transform: &mut Transform,
    vehicle_state: &mut VehicleState,
    boundary_effects: &BoundaryEffects,
    _vehicle_health: Option<Mut<VehicleHealth>>,
    time: &Res<Time>,
    world_rng: &mut WorldRng,
) {
    let intensity = boundary_effects.effect_intensity;

    match boundary_effects.current_zone {
        BoundaryZone::Warning => {
            // Rough seas - boat becomes harder to control
            let wave_force = Vec3::new(
                (time.elapsed_secs() * 2.0).sin() * intensity * 5.0,
                (time.elapsed_secs() * 3.0).cos() * intensity * 2.0,
                (time.elapsed_secs() * 1.5).sin() * intensity * 5.0,
            );
            transform.translation += wave_force * time.delta_secs();

            // Reduced engine efficiency in rough seas
            vehicle_state.max_speed *= 1.0 - (intensity * 0.2);
        }
        BoundaryZone::Critical => {
            // Storm conditions - severe waves and reduced visibility
            let storm_force = Vec3::new(
                (time.elapsed_secs() * 4.0).sin() * intensity * 15.0,
                (time.elapsed_secs() * 5.0).cos() * intensity * 8.0,
                (time.elapsed_secs() * 3.0).sin() * intensity * 15.0,
            );
            transform.translation += storm_force * time.delta_secs();

            // Engine struggles in storm
            vehicle_state.max_speed *= 1.0 - (intensity * 0.6);
            vehicle_state.acceleration *= 1.0 - (intensity * 0.4);

            if world_rng.global().gen_bool((0.01 * intensity) as f64) {
                info!("Boat warning: Storm conditions at world boundary!");
            }
        }
        BoundaryZone::OutOfBounds => {
            // Tsunami/massive waves - boat is overwhelmed
            vehicle_state.max_speed *= 0.1; // Engine nearly unusable

            // Massive wave pushes boat back toward safe zone
            let push_back_force = transform.translation.normalize_or_zero() * -100.0;
            transform.translation += push_back_force * time.delta_secs();
        }
        _ => {}
    }
}

/// On-foot boundary effects - increasing hostility and danger
fn apply_onfoot_boundary_effects(
    _entity: Entity,
    _commands: &mut Commands,
    _transform: &mut Transform,
    vehicle_state: &mut VehicleState,
    boundary_effects: &BoundaryEffects,
    mut vehicle_health: Option<Mut<VehicleHealth>>,
    time: &Res<Time>,
    world_rng: &mut WorldRng,
) {
    let intensity = boundary_effects.effect_intensity;

    match boundary_effects.current_zone {
        BoundaryZone::Warning => {
            // Environmental hazards - reduced movement speed
            vehicle_state.max_speed *= 1.0 - (intensity * 0.2);

            // Occasional warning about dangerous area
            if world_rng.global().gen_bool((0.005 * intensity) as f64) {
                info!("Warning: Dangerous area ahead - hostile wildlife detected");
            }
        }
        BoundaryZone::Critical => {
            // Hostile environment - significant movement penalty
            vehicle_state.max_speed *= 1.0 - (intensity * 0.5);

            // Environmental damage over time
            if let Some(ref mut health) = vehicle_health {
                health.current -= 5.0 * intensity * time.delta_secs();
                health.current = health.current.max(1.0);
            }

            // Spawn hostile creatures/environmental hazards
            if world_rng.global().gen_bool((0.01 * intensity) as f64) {
                // TODO: Spawn hostile NPCs or environmental hazards
                info!("Hostile encounter at world boundary!");
            }
        }
        BoundaryZone::OutOfBounds => {
            // Extreme hostile environment
            vehicle_state.max_speed *= 0.2; // Very slow movement

            // Continuous damage
            if let Some(ref mut health) = vehicle_health {
                health.current -= 20.0 * time.delta_secs();
            }
        }
        _ => {}
    }
}

/// Ground vehicle boundary effects - terrain and mechanical issues
fn apply_ground_vehicle_boundary_effects(
    _entity: Entity,
    _commands: &mut Commands,
    _transform: &mut Transform,
    vehicle_state: &mut VehicleState,
    boundary_effects: &BoundaryEffects,
    _vehicle_health: Option<Mut<VehicleHealth>>,
    _time: &Res<Time>,
    world_rng: &mut WorldRng,
) {
    let intensity = boundary_effects.effect_intensity;

    match boundary_effects.current_zone {
        BoundaryZone::Warning => {
            // Rough terrain - reduced traction and speed
            vehicle_state.max_speed *= 1.0 - (intensity * 0.3);
            vehicle_state.acceleration *= 1.0 - (intensity * 0.2);
        }
        BoundaryZone::Critical => {
            // Very rough terrain - significant performance loss
            vehicle_state.max_speed *= 1.0 - (intensity * 0.6);
            vehicle_state.acceleration *= 1.0 - (intensity * 0.5);

            // Random tire punctures or mechanical issues
            if world_rng.global().gen_bool((0.005 * intensity) as f64) {
                vehicle_state.max_speed *= 0.5; // Engine trouble
                info!("Vehicle trouble: Mechanical failure in rough terrain!");
            }
        }
        BoundaryZone::OutOfBounds => {
            // Impassable terrain
            vehicle_state.max_speed *= 0.1;
            vehicle_state.acceleration *= 0.2;
        }
        _ => {}
    }
}

/// Submarine boundary effects - pressure and oxygen limits
fn apply_submarine_boundary_effects(
    _entity: Entity,
    _commands: &mut Commands,
    transform: &mut Transform,
    vehicle_state: &mut VehicleState,
    boundary_effects: &BoundaryEffects,
    mut vehicle_health: Option<Mut<VehicleHealth>>,
    time: &Res<Time>,
    world_rng: &mut WorldRng,
) {
    let intensity = boundary_effects.effect_intensity;

    match boundary_effects.current_zone {
        BoundaryZone::Warning => {
            // Increasing pressure - slight performance loss
            vehicle_state.max_speed *= 1.0 - (intensity * 0.2);

            if world_rng.global().gen_bool((0.01 * intensity) as f64) {
                info!("Submarine warning: Pressure increasing at depth limits");
            }
        }
        BoundaryZone::Critical => {
            // High pressure - significant systems strain
            vehicle_state.max_speed *= 1.0 - (intensity * 0.5);
            vehicle_state.acceleration *= 1.0 - (intensity * 0.3);

            // Pressure damage
            if let Some(ref mut health) = vehicle_health {
                health.current -= 8.0 * intensity * time.delta_secs();
                health.current = health.current.max(1.0);
            }
        }
        BoundaryZone::OutOfBounds => {
            // Crush depth - severe pressure damage
            if let Some(ref mut health) = vehicle_health {
                health.current -= 50.0 * time.delta_secs(); // Rapid hull failure
            }

            // Force emergency ascent
            let ascent_force = Vec3::new(0.0, 20.0, 0.0);
            transform.translation += ascent_force * time.delta_secs();
        }
        _ => {}
    }
}

/// Determine boundary vehicle type from VehicleState
fn get_boundary_vehicle_type(vehicle_state: &VehicleState) -> BoundaryVehicleType {
    // This would need to be implemented based on your VehicleState structure
    // For now, return a default
    match vehicle_state.vehicle_type {
        crate::components::vehicles::VehicleType::F16 => BoundaryVehicleType::Aircraft,
        crate::components::vehicles::VehicleType::Helicopter => BoundaryVehicleType::Aircraft,
        crate::components::vehicles::VehicleType::SuperCar => BoundaryVehicleType::GroundVehicle,
        crate::components::vehicles::VehicleType::Yacht => BoundaryVehicleType::GroundVehicle,
    }
}

/// Get human-readable vehicle type name
fn get_vehicle_type_name(vehicle_type: BoundaryVehicleType) -> &'static str {
    match vehicle_type {
        BoundaryVehicleType::OnFoot => "Player",
        BoundaryVehicleType::GroundVehicle => "Ground Vehicle",
        BoundaryVehicleType::Aircraft => "Aircraft",
        BoundaryVehicleType::Boat => "Boat",
        BoundaryVehicleType::Submarine => "Submarine",
    }
}
