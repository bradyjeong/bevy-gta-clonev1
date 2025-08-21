use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::fs;
use std::collections::HashMap;

use crate::components::*;
use crate::game_state::GameState;

use super::save_system::*;

#[derive(Resource, Default)]
pub struct LoadState {
    pub entity_mapping: HashMap<u32, Entity>,
    pub pending_load: bool,
}

pub fn load_game_system(
    input: Res<ButtonInput<KeyCode>>,
    mut load_state: ResMut<LoadState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,

    // Queries for cleanup
    player_query: Query<Entity, With<Player>>,
    car_query: Query<Entity, With<Car>>,
    helicopter_query: Query<Entity, With<Helicopter>>,
    f16_query: Query<Entity, With<F16>>,
    active_query: Query<Entity, With<ActiveEntity>>,
) {
    if !input.just_pressed(KeyCode::F9) {
        return;
    }

    info!("Starting load operation...");

    // Load save file
    let save_data = match load_save_file() {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to load save file: {}", err);
            return;
        }
    };

    // Validate loaded data
    if let Err(err) = save_data.validate() {
        error!("Loaded save validation failed: {}", err);
        return;
    }

    // Clear existing entities
    cleanup_existing_entities(&mut commands, &player_query, &car_query, &helicopter_query, &f16_query, &active_query);

    // Clear entity mapping
    load_state.entity_mapping.clear();
    
    // World offset no longer needed with finite world
    
    // Load player
    let player_entity = spawn_player(&mut commands, &save_data.player);
    load_state.entity_mapping.insert(save_data.player.entity_id, player_entity);

    // Load vehicles
    for vehicle_data in &save_data.vehicles {
        let vehicle_entity = spawn_vehicle(&mut commands, vehicle_data);
        load_state.entity_mapping.insert(vehicle_data.entity_id, vehicle_entity);
    }

    // Set up ActiveEntity and relationships
    setup_active_entity_and_relationships(
        &mut commands,
        &save_data,
        &load_state.entity_mapping,
        player_entity,
    );

    // Set game state
    let game_state = save_data.game_state.clone();
    next_state.set(game_state.clone());

    // Post-load validation
    if let Err(err) = validate_post_load(&save_data, &load_state.entity_mapping) {
        error!("Post-load validation failed: {}", err);
        return;
    }

    info!("Game loaded successfully!");
    info!("Loaded state: {:?}, Active entity: {:?}", game_state, save_data.active_entity_id);
}

fn load_save_file() -> Result<SaveGameState, String> {
    let save_path = "saves/savegame.ron";
    
    let content = fs::read_to_string(save_path)
        .map_err(|e| format!("Failed to read save file: {}", e))?;
    
    let save_data: SaveGameState = ron::from_str(&content)
        .map_err(|e| format!("Failed to parse save file: {}", e))?;

    Ok(save_data)
}

fn cleanup_existing_entities(
    commands: &mut Commands,
    player_query: &Query<Entity, With<Player>>,
    car_query: &Query<Entity, With<Car>>,
    helicopter_query: &Query<Entity, With<Helicopter>>,
    f16_query: &Query<Entity, With<F16>>,
    active_query: &Query<Entity, With<ActiveEntity>>,
) {
    info!("Cleaning up existing entities...");

    // Remove ActiveEntity from all entities first
    for entity in active_query.iter() {
        commands.entity(entity).remove::<ActiveEntity>();
    }

    // Despawn all existing entities
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in car_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in helicopter_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in f16_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_player(commands: &mut Commands, player_data: &SerializablePlayer) -> Entity {
    info!("Spawning player...");

    let transform: Transform = player_data.transform.clone().into();
    let velocity: Velocity = player_data.velocity.clone().into();
    
    let mut entity_commands = commands.spawn((
        Player,
        transform,
        velocity,
        HumanMovement::default(),
        HumanAnimation::default(),
        HumanBehavior::default(),
        PlayerBody::default(),
        RigidBody::Dynamic,
        Collider::capsule_y(0.9, 0.4),
        ColliderMassProperties::Density(1.0),
        Ccd::enabled(),
        LockedAxes::ROTATION_LOCKED,
        Name::new("Player"),
    ));

    // Set visibility
    if player_data.visibility {
        entity_commands.insert(Visibility::Visible);
    } else {
        entity_commands.insert(Visibility::Hidden);
    }

    let entity = entity_commands.id();
    info!("Player entity spawned: {:?}", entity);
    entity
}

fn spawn_vehicle(commands: &mut Commands, vehicle_data: &SerializableVehicle) -> Entity {
    info!("Spawning vehicle: {:?}", vehicle_data.vehicle_type);

    let vehicle_state: VehicleState = vehicle_data.vehicle_state.clone().into();
    let transform: Transform = vehicle_data.transform.clone().into();
    let velocity: Velocity = vehicle_data.velocity.clone().into();

    let mut entity_commands = commands.spawn((
        transform,
        velocity,
        vehicle_state,
        RigidBody::Dynamic,
        Ccd::enabled(),
        Name::new(format!("{:?}", vehicle_data.vehicle_type)),
    ));

    match vehicle_data.vehicle_type {
        VehicleType::BasicCar => {
            entity_commands.insert((
                Car,
                Collider::cuboid(2.0, 1.0, 4.5),
                ColliderMassProperties::Density(0.8),
            ));
        }

        VehicleType::Helicopter => {
            entity_commands.insert((
                Helicopter,
                Collider::cuboid(2.0, 1.5, 8.0),
                ColliderMassProperties::Density(0.6),
            ));
        }
        VehicleType::F16 => {
            let aircraft_flight_data: AircraftFlight = vehicle_data.aircraft_flight_data.as_ref()
                .map(|af| af.clone().into())
                .unwrap_or_default();
            
            entity_commands.insert((
                F16,
                aircraft_flight_data,
                Collider::cuboid(1.5, 1.0, 6.0),
                ColliderMassProperties::Density(0.7),
            ));
        }
    }

    let entity = entity_commands.id();
    info!("Vehicle entity spawned: {:?} -> {:?}", vehicle_data.vehicle_type, entity);
    entity
}

fn setup_active_entity_and_relationships(
    commands: &mut Commands,
    save_data: &SaveGameState,
    entity_mapping: &HashMap<u32, Entity>,
    player_entity: Entity,
) {
    info!("Setting up ActiveEntity and relationships...");

    // Set up ActiveEntity
    if let Some(active_id) = save_data.active_entity_id {
        if let Some(&active_entity) = entity_mapping.get(&active_id) {
            commands.entity(active_entity).insert(ActiveEntity);
            info!("ActiveEntity assigned to: {:?}", active_entity);
        }
    }

    // Set up player-vehicle relationships
    if let Some(vehicle_id) = save_data.player.in_vehicle {
        if let Some(&vehicle_entity) = entity_mapping.get(&vehicle_id) {
            // Set up parent-child relationship
            commands.entity(player_entity).insert(ChildOf(vehicle_entity));
            commands.entity(player_entity).insert(InCar(vehicle_entity));
            info!("Player assigned to vehicle: {:?}", vehicle_entity);
        }
    }

    // Validate state consistency
    match save_data.game_state {
        GameState::Walking => {
            if save_data.player.is_active {
                commands.entity(player_entity).insert(ActiveEntity);
            }
        }
        GameState::Driving | GameState::Flying | GameState::Jetting => {
            // Vehicle should be active, player should be hidden
            if !save_data.player.visibility {
                commands.entity(player_entity).insert(Visibility::Hidden);
            }
        }
    }
}

fn validate_post_load(
    save_data: &SaveGameState,
    entity_mapping: &HashMap<u32, Entity>,
) -> Result<(), String> {
    info!("Running post-load validation...");

    // Check that all saved entities were recreated
    if !entity_mapping.contains_key(&save_data.player.entity_id) {
        return Err("Player entity not found in mapping".to_string());
    }

    for vehicle in &save_data.vehicles {
        if !entity_mapping.contains_key(&vehicle.entity_id) {
            return Err(format!("Vehicle entity {} not found in mapping", vehicle.entity_id));
        }
    }

    // Check ActiveEntity consistency
    if let Some(active_id) = save_data.active_entity_id {
        if !entity_mapping.contains_key(&active_id) {
            return Err("ActiveEntity not found in mapping".to_string());
        }
    }

    // Check GameState consistency
    match save_data.game_state {
        GameState::Walking => {
            if !save_data.player.is_active && save_data.active_entity_id.is_none() {
                return Err("Walking state should have active player".to_string());
            }
        }
        GameState::Driving | GameState::Flying | GameState::Jetting => {
            if save_data.player.in_vehicle.is_none() {
                return Err("Vehicle state requires player in vehicle".to_string());
            }
        }
    }

    info!("Post-load validation completed successfully");
    Ok(())
}


