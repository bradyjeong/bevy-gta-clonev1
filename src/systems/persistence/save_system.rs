use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

use crate::components::*;
use crate::game_state::GameState;
use crate::systems::floating_origin::WorldOffset;
use super::aircraft_persistence::SerializableAircraftFlight;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // Quaternion as [x, y, z, w]
    pub scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
        }
    }
}

impl Into<Transform> for SerializableTransform {
    fn into(self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_xyzw(
                self.rotation[0],
                self.rotation[1],
                self.rotation[2],
                self.rotation[3],
            ),
            scale: Vec3::from_array(self.scale),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVelocity {
    pub linvel: [f32; 3],
    pub angvel: [f32; 3],
}

impl From<Velocity> for SerializableVelocity {
    fn from(velocity: Velocity) -> Self {
        Self {
            linvel: velocity.linvel.to_array(),
            angvel: velocity.angvel.to_array(),
        }
    }
}

impl Into<Velocity> for SerializableVelocity {
    fn into(self) -> Velocity {
        Velocity {
            linvel: Vec3::from_array(self.linvel),
            angvel: Vec3::from_array(self.angvel),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializablePlayer {
    pub entity_id: u32,
    pub transform: SerializableTransform,
    pub velocity: SerializableVelocity,
    pub is_active: bool,
    pub in_vehicle: Option<u32>,
    pub visibility: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVehicle {
    pub entity_id: u32,
    pub vehicle_type: VehicleType,
    pub transform: SerializableTransform,
    pub velocity: SerializableVelocity,
    pub is_active: bool,
    pub vehicle_state: SerializableVehicleState,

    pub aircraft_flight_data: Option<SerializableAircraftFlight>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableVehicleState {
    pub vehicle_type: VehicleType,
    pub color: [f32; 4], // RGBA
    pub max_speed: f32,
    pub acceleration: f32,
    pub damage: f32,
    pub fuel: f32,
}

impl From<VehicleState> for SerializableVehicleState {
    fn from(state: VehicleState) -> Self {
        let srgba = state.color.to_srgba();
        Self {
            vehicle_type: state.vehicle_type,
            color: [srgba.red, srgba.green, srgba.blue, srgba.alpha],
            max_speed: state.max_speed,
            acceleration: state.acceleration,
            damage: state.damage,
            fuel: state.fuel,
        }
    }
}

impl Into<VehicleState> for SerializableVehicleState {
    fn into(self) -> VehicleState {
        VehicleState {
            vehicle_type: self.vehicle_type,
            color: Color::srgba(self.color[0], self.color[1], self.color[2], self.color[3]),
            max_speed: self.max_speed,
            acceleration: self.acceleration,
            damage: self.damage,
            fuel: self.fuel,
            current_lod: crate::components::VehicleLOD::StateOnly,
            last_lod_check: 0.0,
        }
    }
}





// Legacy serialization code removed - keeping only essential components

// Use the new simplified aircraft persistence module

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveGameState {
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub game_state: GameState,
    pub active_entity_id: Option<u32>,
    pub player: SerializablePlayer,
    pub vehicles: Vec<SerializableVehicle>,
    pub world_seed: Option<u64>,
    pub play_time: f64,
    #[serde(default)]
    pub world_offset: [f32; 3], // WorldOffset for floating origin system
}

impl SaveGameState {
    pub fn validate(&self) -> Result<(), String> {
        // Version compatibility check
        if self.version > SAVE_VERSION {
            return Err(format!("Save version {} is too new (current: {})", self.version, SAVE_VERSION));
        }

        // ActiveEntity validation
        if let Some(active_id) = self.active_entity_id {
            let found = self.player.entity_id == active_id || 
                       self.vehicles.iter().any(|v| v.entity_id == active_id);
            if !found {
                return Err("ActiveEntity reference not found in saved entities".to_string());
            }
        }

        // GameState synchronization check
        match self.game_state {
            GameState::Walking => {
                if !self.player.is_active {
                    return Err("Walking state requires active player".to_string());
                }
            }
            GameState::Driving | GameState::Flying | GameState::Jetting => {
                if self.player.in_vehicle.is_none() {
                    return Err("Vehicle state requires player in vehicle".to_string());
                }
                let vehicle_active = self.vehicles.iter().any(|v| v.is_active);
                if !vehicle_active {
                    return Err("Vehicle state requires active vehicle".to_string());
                }
            }
        }

        // Physics bounds validation
        for vehicle in &self.vehicles {
            let pos = &vehicle.transform.translation;
            if pos[0].abs() > 10000.0 || pos[1].abs() > 10000.0 || pos[2].abs() > 10000.0 {
                return Err("Invalid vehicle position detected".to_string());
            }
            
            let vel = &vehicle.velocity.linvel;
            if vel[0].abs() > 1000.0 || vel[1].abs() > 1000.0 || vel[2].abs() > 1000.0 {
                return Err("Invalid vehicle velocity detected".to_string());
            }
        }

        Ok(())
    }
}

const SAVE_VERSION: u32 = 1;
const MAX_BACKUPS: usize = 3;

pub fn save_game_system(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    current_state: Res<State<GameState>>,
    world_offset: Res<WorldOffset>,
    player_query: Query<(Entity, &Transform, &Velocity, Option<&InCar>), With<Player>>,
    active_query: Query<Entity, With<ActiveEntity>>,
    car_query: Query<(Entity, &Transform, &Velocity, &VehicleState), With<Car>>,

    helicopter_query: Query<(Entity, &Transform, &Velocity, &VehicleState), With<Helicopter>>,
    f16_query: Query<(Entity, &Transform, &Velocity, &VehicleState, &AircraftFlight), With<F16>>,
) {
    if !input.just_pressed(KeyCode::F5) {
        return;
    }

    info!("Starting save operation...");

    // Get player data
    let Ok((player_entity, player_transform, player_velocity, in_car)) = player_query.single() else {
        error!("Failed to find player for save operation");
        return;
    };

    // Check if player is active
    let player_is_active = active_query.get(player_entity).is_ok();

    let serializable_player = SerializablePlayer {
        entity_id: player_entity.index(),
        transform: (*player_transform).into(),
        velocity: (*player_velocity).into(),
        is_active: player_is_active,
        in_vehicle: in_car.map(|ic| ic.0.index()),
        visibility: true, // We'll determine this from state
    };

    // Collect all vehicles
    let mut vehicles = Vec::new();

    // Cars
    for (entity, transform, velocity, vehicle_state) in car_query.iter() {
        let is_active = active_query.get(entity).is_ok();
        

        
        vehicles.push(SerializableVehicle {
            entity_id: entity.index(),
            vehicle_type: vehicle_state.vehicle_type,
            transform: (*transform).into(),
            velocity: (*velocity).into(),
            is_active,
            vehicle_state: vehicle_state.clone().into(),

            aircraft_flight_data: None,
        });
    }

    // Helicopters
    for (entity, transform, velocity, vehicle_state) in helicopter_query.iter() {
        let is_active = active_query.get(entity).is_ok();
        vehicles.push(SerializableVehicle {
            entity_id: entity.index(),
            vehicle_type: vehicle_state.vehicle_type,
            transform: (*transform).into(),
            velocity: (*velocity).into(),
            is_active,
            vehicle_state: vehicle_state.clone().into(),
            aircraft_flight_data: None,
        });
    }

    // F16s
    for (entity, transform, velocity, vehicle_state, aircraft_flight) in f16_query.iter() {
        let is_active = active_query.get(entity).is_ok();
        vehicles.push(SerializableVehicle {
            entity_id: entity.index(),
            vehicle_type: vehicle_state.vehicle_type,
            transform: (*transform).into(),
            velocity: (*velocity).into(),
            is_active,
            vehicle_state: vehicle_state.clone().into(),
            aircraft_flight_data: Some((&*aircraft_flight).into()),
        });
    }

    // Get active entity
    let active_entity_id = active_query.single().ok().map(|e| e.index());

    // Create save state
    let save_state = SaveGameState {
        version: SAVE_VERSION,
        timestamp: Utc::now(),
        game_state: current_state.clone(),
        active_entity_id,
        player: serializable_player,
        vehicles,
        world_seed: None, // TODO: Add world generation seed if needed
        play_time: time.elapsed_secs_f64(),
        world_offset: world_offset.offset.to_array(),
    };

    // Validate save state
    if let Err(err) = save_state.validate() {
        error!("Save validation failed: {}", err);
        return;
    }

    // Create saves directory
    if let Err(err) = fs::create_dir_all("saves") {
        error!("Failed to create saves directory: {}", err);
        return;
    }

    // Backup existing saves
    backup_saves();

    // Serialize and save
    let ron_string = match ron::to_string(&save_state) {
        Ok(s) => s,
        Err(err) => {
            error!("Failed to serialize save state: {}", err);
            return;
        }
    };

    let save_path = "saves/savegame.ron";
    if let Err(err) = fs::write(save_path, ron_string) {
        error!("Failed to write save file: {}", err);
        return;
    }

    info!("Game saved successfully to {}", save_path);
    info!("Active entity: {:?}, Game state: {:?}", active_entity_id, **current_state);
}

fn backup_saves() {
    let save_path = Path::new("saves/savegame.ron");
    if !save_path.exists() {
        return;
    }

    // Shift existing backups
    for i in (1..MAX_BACKUPS).rev() {
        let old_backup = format!("saves/savegame.backup.{}.ron", i);
        let new_backup = format!("saves/savegame.backup.{}.ron", i + 1);
        let _ = fs::rename(&old_backup, &new_backup);
    }

    // Create new backup
    let _ = fs::copy(save_path, "saves/savegame.backup.1.ron");
    info!("Created backup of existing save");
}
