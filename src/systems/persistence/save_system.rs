use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

use crate::components::*;
use crate::game_state::GameState;

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





#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableTurboSystem {
    pub turbo_boost: bool,
    pub pressure: f32,
    pub lag: f32,
    pub cooldown: f32,
    pub max_time: f32,
    pub current_time: f32,
    pub stage: u8,
    pub pressure_buildup: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableEngineState {
    pub rpm: f32,
    pub max_rpm: f32,
    pub idle_rpm: f32,
    pub power_band_start: f32,
    pub power_band_end: f32,
    pub temperature: f32,
    pub oil_pressure: f32,
    pub fuel_consumption_rate: f32,
    pub rev_limiter_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableTransmission {
    pub gear: u8,
    pub gear_ratios: Vec<f32>,
    pub shift_rpm: f32,
    pub downshift_rpm: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableDrivingModes {
    pub mode: DrivingMode,
    pub launch_control: bool,
    pub launch_control_engaged: bool,
    pub launch_rpm_limit: f32,
    pub sport_mode_active: bool,
    pub track_mode_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableAerodynamicsSystem {
    pub downforce: f32,
    pub active_aero: bool,
    pub rear_wing_angle: f32,
    pub front_splitter_level: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializablePerformanceMetrics {
    pub g_force_lateral: f32,
    pub g_force_longitudinal: f32,
    pub performance_timer: f32,
    pub zero_to_sixty_time: f32,
    pub is_timing_launch: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableExhaustSystem {
    pub note_mode: ExhaustMode,
    pub engine_note_intensity: f32,
    pub turbo_whistle_intensity: f32,
    pub backfire_timer: f32,
    pub pops_and_bangs: bool,
}

// Component conversion implementations

impl From<TurboSystem> for SerializableTurboSystem {
    fn from(turbo: TurboSystem) -> Self {
        Self {
            turbo_boost: turbo.turbo_boost,
            pressure: turbo.pressure,
            lag: turbo.lag,
            cooldown: turbo.cooldown,
            max_time: turbo.max_time,
            current_time: turbo.current_time,
            stage: turbo.stage,
            pressure_buildup: turbo.pressure_buildup,
        }
    }
}

impl Into<TurboSystem> for SerializableTurboSystem {
    fn into(self) -> TurboSystem {
        TurboSystem {
            turbo_boost: self.turbo_boost,
            pressure: self.pressure,
            lag: self.lag,
            cooldown: self.cooldown,
            max_time: self.max_time,
            current_time: self.current_time,
            stage: self.stage,
            pressure_buildup: self.pressure_buildup,
        }
    }
}

impl From<EngineState> for SerializableEngineState {
    fn from(engine: EngineState) -> Self {
        Self {
            rpm: engine.rpm,
            max_rpm: engine.max_rpm,
            idle_rpm: engine.idle_rpm,
            power_band_start: engine.power_band_start,
            power_band_end: engine.power_band_end,
            temperature: engine.temperature,
            oil_pressure: engine.oil_pressure,
            fuel_consumption_rate: engine.fuel_consumption_rate,
            rev_limiter_active: engine.rev_limiter_active,
        }
    }
}

impl Into<EngineState> for SerializableEngineState {
    fn into(self) -> EngineState {
        EngineState {
            rpm: self.rpm,
            max_rpm: self.max_rpm,
            idle_rpm: self.idle_rpm,
            power_band_start: self.power_band_start,
            power_band_end: self.power_band_end,
            temperature: self.temperature,
            oil_pressure: self.oil_pressure,
            fuel_consumption_rate: self.fuel_consumption_rate,
            rev_limiter_active: self.rev_limiter_active,
        }
    }
}

impl From<Transmission> for SerializableTransmission {
    fn from(transmission: Transmission) -> Self {
        Self {
            gear: transmission.gear,
            gear_ratios: transmission.gear_ratios,
            shift_rpm: transmission.shift_rpm,
            downshift_rpm: transmission.downshift_rpm,
        }
    }
}

impl Into<Transmission> for SerializableTransmission {
    fn into(self) -> Transmission {
        Transmission {
            gear: self.gear,
            gear_ratios: self.gear_ratios,
            shift_rpm: self.shift_rpm,
            downshift_rpm: self.downshift_rpm,
        }
    }
}

impl From<DrivingModes> for SerializableDrivingModes {
    fn from(modes: DrivingModes) -> Self {
        Self {
            mode: modes.mode,
            launch_control: modes.launch_control,
            launch_control_engaged: modes.launch_control_engaged,
            launch_rpm_limit: modes.launch_rpm_limit,
            sport_mode_active: modes.sport_mode_active,
            track_mode_active: modes.track_mode_active,
        }
    }
}

impl Into<DrivingModes> for SerializableDrivingModes {
    fn into(self) -> DrivingModes {
        DrivingModes {
            mode: self.mode,
            launch_control: self.launch_control,
            launch_control_engaged: self.launch_control_engaged,
            launch_rpm_limit: self.launch_rpm_limit,
            sport_mode_active: self.sport_mode_active,
            track_mode_active: self.track_mode_active,
        }
    }
}

impl From<AerodynamicsSystem> for SerializableAerodynamicsSystem {
    fn from(aero: AerodynamicsSystem) -> Self {
        Self {
            downforce: aero.downforce,
            active_aero: aero.active_aero,
            rear_wing_angle: aero.rear_wing_angle,
            front_splitter_level: aero.front_splitter_level,
        }
    }
}

impl Into<AerodynamicsSystem> for SerializableAerodynamicsSystem {
    fn into(self) -> AerodynamicsSystem {
        AerodynamicsSystem {
            downforce: self.downforce,
            active_aero: self.active_aero,
            rear_wing_angle: self.rear_wing_angle,
            front_splitter_level: self.front_splitter_level,
        }
    }
}

impl From<PerformanceMetrics> for SerializablePerformanceMetrics {
    fn from(metrics: PerformanceMetrics) -> Self {
        Self {
            g_force_lateral: metrics.g_force_lateral,
            g_force_longitudinal: metrics.g_force_longitudinal,
            performance_timer: metrics.performance_timer,
            zero_to_sixty_time: metrics.zero_to_sixty_time,
            is_timing_launch: metrics.is_timing_launch,
        }
    }
}

impl Into<PerformanceMetrics> for SerializablePerformanceMetrics {
    fn into(self) -> PerformanceMetrics {
        PerformanceMetrics {
            g_force_lateral: self.g_force_lateral,
            g_force_longitudinal: self.g_force_longitudinal,
            performance_timer: self.performance_timer,
            zero_to_sixty_time: self.zero_to_sixty_time,
            is_timing_launch: self.is_timing_launch,
        }
    }
}

impl From<ExhaustSystem> for SerializableExhaustSystem {
    fn from(exhaust: ExhaustSystem) -> Self {
        Self {
            note_mode: exhaust.note_mode,
            engine_note_intensity: exhaust.engine_note_intensity,
            turbo_whistle_intensity: exhaust.turbo_whistle_intensity,
            backfire_timer: exhaust.backfire_timer,
            pops_and_bangs: exhaust.pops_and_bangs,
        }
    }
}

impl Into<ExhaustSystem> for SerializableExhaustSystem {
    fn into(self) -> ExhaustSystem {
        ExhaustSystem {
            note_mode: self.note_mode,
            engine_note_intensity: self.engine_note_intensity,
            turbo_whistle_intensity: self.turbo_whistle_intensity,
            backfire_timer: self.backfire_timer,
            pops_and_bangs: self.pops_and_bangs,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableAircraftFlight {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
    pub throttle: f32,
    pub airspeed: f32,
    pub afterburner: bool,
    pub current_thrust: f32,
}

impl From<AircraftFlight> for SerializableAircraftFlight {
    fn from(flight: AircraftFlight) -> Self { 
        Self {
            pitch: flight.pitch,
            roll: flight.roll,
            yaw: flight.yaw,
            throttle: flight.throttle,
            airspeed: flight.airspeed,
            afterburner: flight.afterburner_active,
            current_thrust: flight.current_thrust,
        }
    }
}

impl Into<AircraftFlight> for SerializableAircraftFlight {
    fn into(self) -> AircraftFlight {
        AircraftFlight {
            pitch: self.pitch,
            roll: self.roll,
            yaw: self.yaw,
            throttle: self.throttle,
            airspeed: self.airspeed,
            afterburner_active: self.afterburner, // Use saved afterburner state
            current_thrust: self.current_thrust,
        }
    }
}

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
            aircraft_flight_data: Some(aircraft_flight.clone().into()),
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
