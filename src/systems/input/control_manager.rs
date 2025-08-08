use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

use crate::game_state::GameState;
use crate::components::{Car, ActiveEntity, Player, Helicopter, F16, NPC, SuperCarSpecs};
use super::input_config::InputAction;
use super::input_manager::InputManager;
use super::vehicle_control_config::{VehicleType as ConfigVehicleType, VehicleControlConfig as ExistingVehicleControlConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlAction {
    // Movement
    Accelerate,
    Brake,
    Steer,
    
    // Vertical (aircraft/helicopter)
    Pitch,
    Roll,
    Yaw,
    Throttle,
    
    // Special
    Turbo,
    Afterburner,
    EmergencyBrake,
    
    // AI/NPC Actions
    NPCMove,
    NPCTurn,
    NPCWander,
    
    // Interaction
    Interact,
    DebugInfo,
    EmergencyReset,
}

/// Trait for entities that can be controlled
pub trait Controllable {
    /// Get the entity type for control mapping
    fn get_control_type(&self) -> ControlEntityType;
    
    /// Apply control actions to the entity
    fn apply_controls(&mut self, controls: &HashMap<ControlAction, f32>, dt: f32);
}

/// Types of entities that can be controlled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlEntityType {
    Player,
    Vehicle,
    SuperVehicle,
    Helicopter,
    Aircraft,
    NPC,
}

/// AI control decision for NPCs
#[derive(Debug, Clone)]
pub struct AIControlDecision {
    pub movement_direction: Vec3,
    pub rotation_target: f32,
    pub speed_factor: f32,
    pub action_priority: f32,
}

impl Default for AIControlDecision {
    fn default() -> Self {
        Self {
            movement_direction: Vec3::ZERO,
            rotation_target: 0.0,
            speed_factor: 1.0,
            action_priority: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VehiclePhysicsConfig {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turn_speed: f32,
    pub brake_force: f32,
    
    // Physics constraints
    pub max_acceleration: f32,
    pub max_angular_velocity: f32,
    pub position_clamp: Vec3,
    
    // Input sensitivity
    pub acceleration_sensitivity: f32,
    pub steering_sensitivity: f32,
    pub brake_sensitivity: f32,
    
    // Safety limits
    pub enable_safety_limits: bool,
    pub max_safe_speed: f32,
    pub stability_assist: bool,
}

impl Default for VehiclePhysicsConfig {
    fn default() -> Self {
        Self {
            max_speed: 25.0,
            acceleration: 20.0,
            turn_speed: 2.0,
            brake_force: 30.0,
            max_acceleration: 50.0,
            max_angular_velocity: 5.0,
            position_clamp: Vec3::new(500.0, 100.0, 500.0),
            acceleration_sensitivity: 1.0,
            steering_sensitivity: 1.0,
            brake_sensitivity: 1.0,
            enable_safety_limits: true,
            max_safe_speed: 100.0,
            stability_assist: true,
        }
    }
}

#[derive(Resource)]
pub struct ControlManager {
    // Vehicle physics configurations
    physics_configs: HashMap<ConfigVehicleType, VehiclePhysicsConfig>,
    
    // Current control state - maps action to normalized value
    active_controls: HashMap<ControlAction, f32>,
    
    // AI control decisions for NPCs
    ai_decisions: HashMap<Entity, AIControlDecision>,
    
    // Entity type tracking
    entity_types: HashMap<Entity, ControlEntityType>,
    
    // Performance monitoring
    last_update_time: Option<Instant>,
    max_update_time_us: u128,
    validation_failures: u32,
    
    // Safety systems
    emergency_brake_active: bool,
    stability_intervention: bool,
    safety_override: bool,
}

impl Default for ControlManager {
    fn default() -> Self {
        let mut control_manager = ControlManager {
            physics_configs: HashMap::new(),
            active_controls: HashMap::new(),
            ai_decisions: HashMap::new(),
            entity_types: HashMap::new(),
            last_update_time: None,
            max_update_time_us: 0,
            validation_failures: 0,
            emergency_brake_active: false,
            stability_intervention: false,
            safety_override: false,
        };
        
        // Initialize default physics configurations
        control_manager.init_default_physics_configs();
        control_manager
    }
}

impl ControlManager {
    fn init_default_physics_configs(&mut self) {
        // Player walking physics
        self.physics_configs.insert(ConfigVehicleType::Walking, VehiclePhysicsConfig {
            max_speed: 20.0,
            acceleration: 40.0,
            turn_speed: 3.0,
            brake_force: 40.0,
            max_acceleration: 30.0,
            max_angular_velocity: 4.0,
            position_clamp: Vec3::new(500.0, 100.0, 500.0),
            acceleration_sensitivity: 1.0,
            steering_sensitivity: 1.0,
            brake_sensitivity: 1.0,
            enable_safety_limits: true,
            max_safe_speed: 20.0,
            stability_assist: false,
        });
        
        // Regular car physics
        self.physics_configs.insert(ConfigVehicleType::Car, VehiclePhysicsConfig {
            max_speed: 25.0,
            acceleration: 20.0,
            turn_speed: 2.0,
            brake_force: 35.0,
            max_acceleration: 40.0,
            max_angular_velocity: 3.0,
            position_clamp: Vec3::new(1000.0, 50.0, 1000.0),
            acceleration_sensitivity: 1.0,
            steering_sensitivity: 1.0,
            brake_sensitivity: 1.0,
            enable_safety_limits: true,
            max_safe_speed: 50.0,
            stability_assist: true,
        });
        
        // SuperCar physics
        self.physics_configs.insert(ConfigVehicleType::SuperCar, VehiclePhysicsConfig {
            max_speed: 80.0,
            acceleration: 40.0,
            turn_speed: 4.5,
            brake_force: 60.0,
            max_acceleration: 80.0,
            max_angular_velocity: 5.0,
            position_clamp: Vec3::new(2000.0, 50.0, 2000.0),
            acceleration_sensitivity: 0.8,
            steering_sensitivity: 0.9,
            brake_sensitivity: 1.2,
            enable_safety_limits: true,
            max_safe_speed: 120.0,
            stability_assist: true,
        });
        
        // Helicopter physics
        self.physics_configs.insert(ConfigVehicleType::Helicopter, VehiclePhysicsConfig {
            max_speed: 50.0,
            acceleration: 15.0,
            turn_speed: 2.5,
            brake_force: 20.0,
            max_acceleration: 30.0,
            max_angular_velocity: 2.5,
            position_clamp: Vec3::new(2000.0, 500.0, 2000.0),
            acceleration_sensitivity: 0.7,
            steering_sensitivity: 0.8,
            brake_sensitivity: 1.0,
            enable_safety_limits: true,
            max_safe_speed: 70.0,
            stability_assist: true,
        });
        
        // F16 physics
        self.physics_configs.insert(ConfigVehicleType::F16, VehiclePhysicsConfig {
            max_speed: 200.0,
            acceleration: 100.0,
            turn_speed: 8.0,
            brake_force: 80.0,
            max_acceleration: 150.0,
            max_angular_velocity: 10.0,
            position_clamp: Vec3::new(5000.0, 2000.0, 5000.0),
            acceleration_sensitivity: 0.6,
            steering_sensitivity: 0.7,
            brake_sensitivity: 0.8,
            enable_safety_limits: false, // F16 has fewer safety limits
            max_safe_speed: 300.0,
            stability_assist: false,
        });
    }
    
    /// Process input and update control state - PERFORMANCE CRITICAL
    pub fn update_controls(
        &mut self,
        input_manager: &InputManager,
        current_state: &GameState,
        current_velocity: &Velocity,
        current_transform: &Transform,
    ) -> Result<(), String> {
        let start_time = Instant::now();
        
        // Clear previous control state
        self.active_controls.clear();
        
        // Convert GameState to VehicleType
        let vehicle_type = ExistingVehicleControlConfig::game_state_to_vehicle_type(current_state);
        
        // Get physics configuration (clone to avoid borrowing issues)
        let physics_config = self.physics_configs.get(&vehicle_type)
            .ok_or_else(|| format!("No physics configuration for vehicle type {:?}", vehicle_type))?
            .clone();
        
        // Map input actions to control actions based on game state
        self.map_input_to_controls(input_manager, current_state, &physics_config)?;
        
        // Validate control inputs for safety
        self.validate_and_clamp_controls(&physics_config, current_velocity, current_transform)?;
        
        // Apply safety systems
        self.apply_safety_systems(&physics_config, current_velocity)?;
        
        // Performance monitoring
        let update_time = start_time.elapsed().as_micros();
        self.max_update_time_us = self.max_update_time_us.max(update_time);
        self.last_update_time = Some(start_time);
        
        if update_time > 500 { // 0.5ms limit for control updates
            warn!("Control update took {}μs (>500μs limit)", update_time);
        }
        
        Ok(())
    }
    
    fn map_input_to_controls(
        &mut self,
        input_manager: &InputManager,
        current_state: &GameState,
        physics_config: &VehiclePhysicsConfig,
    ) -> Result<(), String> {
        match current_state {
            GameState::Walking => {
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Accelerate, 1.0 * physics_config.acceleration_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Brake, 1.0 * physics_config.brake_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::TurnLeft) {
                    self.active_controls.insert(ControlAction::Steer, 1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::TurnRight) {
                    self.active_controls.insert(ControlAction::Steer, -1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::Run) {
                    self.active_controls.insert(ControlAction::Turbo, 1.0);
                }
            }
            
            GameState::Driving => {
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Accelerate, 1.0 * physics_config.acceleration_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Brake, 1.0 * physics_config.brake_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::TurnLeft) {
                    self.active_controls.insert(ControlAction::Steer, 1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::TurnRight) {
                    self.active_controls.insert(ControlAction::Steer, -1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::Turbo) {
                    self.active_controls.insert(ControlAction::Turbo, 1.0);
                }
            }
            
            GameState::Flying => {
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Accelerate, 1.0 * physics_config.acceleration_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Brake, 1.0 * physics_config.brake_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::TurnLeft) {
                    self.active_controls.insert(ControlAction::Yaw, -1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::TurnRight) {
                    self.active_controls.insert(ControlAction::Yaw, 1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::VerticalUp) {
                    self.active_controls.insert(ControlAction::Throttle, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::VerticalDown) {
                    self.active_controls.insert(ControlAction::Throttle, -1.0);
                }
            }
            
            GameState::Jetting => {
                // Flight control surfaces
                if input_manager.is_action_pressed(InputAction::PitchUp) {
                    self.active_controls.insert(ControlAction::Pitch, 1.0 * physics_config.acceleration_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::PitchDown) {
                    self.active_controls.insert(ControlAction::Pitch, -1.0 * physics_config.acceleration_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::RollLeft) {
                    self.active_controls.insert(ControlAction::Roll, -1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::RollRight) {
                    self.active_controls.insert(ControlAction::Roll, 1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::YawLeft) {
                    self.active_controls.insert(ControlAction::Yaw, 1.0 * physics_config.steering_sensitivity);
                }
                if input_manager.is_action_pressed(InputAction::YawRight) {
                    self.active_controls.insert(ControlAction::Yaw, -1.0 * physics_config.steering_sensitivity);
                }
                
                // Engine controls - both Arrow keys AND Shift/Ctrl work for throttle
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Throttle, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Throttle, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::VerticalUp) {
                    self.active_controls.insert(ControlAction::Throttle, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::VerticalDown) {
                    self.active_controls.insert(ControlAction::Throttle, -1.0);
                }
                
                // Afterburner
                if input_manager.is_action_pressed(InputAction::Afterburner) {
                    self.active_controls.insert(ControlAction::Afterburner, 1.0);
                }
            }
        }
        
        // Common actions
        if input_manager.is_action_just_pressed(InputAction::Interact) {
            self.active_controls.insert(ControlAction::Interact, 1.0);
        }
        if input_manager.is_action_just_pressed(InputAction::DebugInfo) {
            self.active_controls.insert(ControlAction::DebugInfo, 1.0);
        }
        if input_manager.is_action_just_pressed(InputAction::EmergencyReset) {
            self.active_controls.insert(ControlAction::EmergencyReset, 1.0);
        }
        
        Ok(())
    }
    
    fn validate_and_clamp_controls(
        &mut self,
        physics_config: &VehiclePhysicsConfig,
        current_velocity: &Velocity,
        current_transform: &Transform,
    ) -> Result<(), String> {
        if !physics_config.enable_safety_limits {
            return Ok(());
        }
        
        let current_speed = current_velocity.linvel.length();
        
        // Clamp position to safe bounds
        let pos = current_transform.translation;
        if pos.x.abs() > physics_config.position_clamp.x ||
           pos.y.abs() > physics_config.position_clamp.y ||
           pos.z.abs() > physics_config.position_clamp.z {
            warn!("Position {} exceeds safety bounds {:?}", pos, physics_config.position_clamp);
            self.active_controls.insert(ControlAction::EmergencyBrake, 1.0);
        }
        
        // Limit acceleration if speed is too high
        if current_speed > physics_config.max_safe_speed {
            // Reduce acceleration if present
            if let Some(accel_value) = self.active_controls.get(&ControlAction::Accelerate) {
                let reduced_accel = accel_value * 0.5;
                self.active_controls.insert(ControlAction::Accelerate, reduced_accel);
            }
            self.stability_intervention = true;
        }
        
        // Validate physics values
        for (_action, value) in &self.active_controls {
            if !value.is_finite() || value.abs() > 10.0 {
                self.validation_failures += 1;
                return Err(format!("Invalid control value: {}", value));
            }
        }
        
        Ok(())
    }
    
    fn apply_safety_systems(
        &mut self,
        physics_config: &VehiclePhysicsConfig,
        current_velocity: &Velocity,
    ) -> Result<(), String> {
        if !physics_config.enable_safety_limits {
            return Ok(());
        }
        
        let current_speed = current_velocity.linvel.length();
        
        // Emergency brake system
        if current_speed > physics_config.max_safe_speed * 1.2 {
            self.emergency_brake_active = true;
            self.active_controls.insert(ControlAction::EmergencyBrake, 1.0);
            
            // Remove acceleration and boost controls
            self.active_controls.remove(&ControlAction::Accelerate);
            self.active_controls.remove(&ControlAction::Turbo);
            self.active_controls.remove(&ControlAction::Afterburner);
        } else if current_speed < physics_config.max_safe_speed * 0.8 {
            self.emergency_brake_active = false;
        }
        
        // Stability assist
        if physics_config.stability_assist && current_speed > physics_config.max_safe_speed * 0.5 {
            // Reduce steering sensitivity at high speeds
            let speed_factor = (current_speed / physics_config.max_safe_speed).min(1.0);
            let steering_reduction = 1.0 - speed_factor * 0.3;
            
            // Apply to steering controls
            if let Some(steer_value) = self.active_controls.get(&ControlAction::Steer) {
                let reduced_value = steer_value * steering_reduction;
                self.active_controls.insert(ControlAction::Steer, reduced_value);
            }
            if let Some(yaw_value) = self.active_controls.get(&ControlAction::Yaw) {
                let reduced_value = yaw_value * steering_reduction;
                self.active_controls.insert(ControlAction::Yaw, reduced_value);
            }
        }
        
        Ok(())
    }
    
    /// Get the current control value for a specific action
    pub fn get_control_value(&self, action: ControlAction) -> f32 {
        self.active_controls.get(&action).copied().unwrap_or(0.0)
    }
    
    /// Check if a control action is active
    pub fn is_control_active(&self, action: ControlAction) -> bool {
        self.active_controls.contains_key(&action)
    }
    
    /// Get all active control actions
    pub fn get_active_controls(&self) -> &HashMap<ControlAction, f32> {
        &self.active_controls
    }
    
    /// Get physics configuration for a vehicle type
    pub fn get_physics_config(&self, vehicle_type: ConfigVehicleType) -> Option<&VehiclePhysicsConfig> {
        self.physics_configs.get(&vehicle_type)
    }
    
    /// Update physics configuration
    pub fn update_physics_config(&mut self, vehicle_type: ConfigVehicleType, config: VehiclePhysicsConfig) {
        self.physics_configs.insert(vehicle_type, config);
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> (u128, u32) {
        (self.max_update_time_us, self.validation_failures)
    }
    
    /// Reset performance statistics
    pub fn reset_performance_stats(&mut self) {
        self.max_update_time_us = 0;
        self.validation_failures = 0;
    }
    
    /// Check if emergency systems are active
    pub fn is_emergency_active(&self) -> bool {
        self.emergency_brake_active
    }
    
    /// Check if stability system is intervening
    pub fn is_stability_active(&self) -> bool {
        self.stability_intervention
    }
    
    /// Enable/disable safety override
    pub fn set_safety_override(&mut self, enabled: bool) {
        self.safety_override = enabled;
    }
    
    /// Clear all control state
    pub fn clear_all_controls(&mut self) {
        self.active_controls.clear();
        self.emergency_brake_active = false;
        self.stability_intervention = false;
    }
    
    /// Register an entity for control management
    pub fn register_entity(&mut self, entity: Entity, entity_type: ControlEntityType) {
        self.entity_types.insert(entity, entity_type);
    }
    
    /// Unregister an entity
    pub fn unregister_entity(&mut self, entity: Entity) {
        self.entity_types.remove(&entity);
        self.ai_decisions.remove(&entity);
    }
    
    /// Update AI control decision for an NPC entity
    pub fn update_ai_decision(&mut self, entity: Entity, decision: AIControlDecision) {
        self.ai_decisions.insert(entity, decision);
    }
    
    /// Get AI control decision for an entity
    pub fn get_ai_decision(&self, entity: Entity) -> Option<&AIControlDecision> {
        self.ai_decisions.get(&entity)
    }
    
    /// Convert AI decision to control actions
    pub fn ai_decision_to_controls(&self, decision: &AIControlDecision) -> HashMap<ControlAction, f32> {
        let mut controls = HashMap::new();
        
        if decision.movement_direction.length() > 0.1 {
            controls.insert(ControlAction::NPCMove, decision.speed_factor);
        }
        
        if decision.rotation_target.abs() > 0.1 {
            controls.insert(ControlAction::NPCTurn, decision.rotation_target);
        }
        
        controls
    }
    
    /// Process unified controls for any entity type
    pub fn process_entity_controls(&mut self, 
        entity: Entity,
        input_manager: Option<&InputManager>,
        current_state: Option<&GameState>,
        current_velocity: &Velocity,
        current_transform: &Transform,
    ) -> Result<HashMap<ControlAction, f32>, String> {
        
        let entity_type = self.entity_types.get(&entity)
            .copied()
            .unwrap_or(ControlEntityType::Vehicle);
            
        let mut controls = HashMap::new();
        
        match entity_type {
            ControlEntityType::Player | 
            ControlEntityType::Vehicle | 
            ControlEntityType::SuperVehicle |
            ControlEntityType::Helicopter |
            ControlEntityType::Aircraft => {
                // Use input manager for player-controlled entities
                if let (Some(input_mgr), Some(state)) = (input_manager, current_state) {
                    self.update_controls(input_mgr, state, current_velocity, current_transform)?;
                    controls = self.active_controls.clone();
                }
            }
            ControlEntityType::NPC => {
                // Use AI decision for NPCs
                if let Some(decision) = self.ai_decisions.get(&entity) {
                    controls = self.ai_decision_to_controls(decision);
                }
            }
        }
        
        Ok(controls)
    }
}

/// System to process control actions each frame - UNIFIED for all entity types
pub fn unified_control_system(
    input_manager: Res<InputManager>,
    mut control_manager: ResMut<ControlManager>,
    current_state: Res<State<GameState>>,
    // All entity queries
    active_entities: Query<(Entity, &Velocity, &Transform), With<ActiveEntity>>,
    player_query: Query<Entity, (With<Player>, With<ActiveEntity>)>,
    car_query: Query<Entity, (With<Car>, With<ActiveEntity>, Without<SuperCarSpecs>)>,
    supercar_query: Query<Entity, (With<SuperCarSpecs>, With<ActiveEntity>)>,
    helicopter_query: Query<Entity, (With<Helicopter>, With<ActiveEntity>)>,
    f16_query: Query<Entity, (With<F16>, With<ActiveEntity>)>,
) {
    // Register entity types for active entities
    for player_entity in player_query.iter() {
        control_manager.register_entity(player_entity, ControlEntityType::Player);
    }
    for car_entity in car_query.iter() {
        control_manager.register_entity(car_entity, ControlEntityType::Vehicle);
    }
    for supercar_entity in supercar_query.iter() {
        control_manager.register_entity(supercar_entity, ControlEntityType::SuperVehicle);
    }
    for heli_entity in helicopter_query.iter() {
        control_manager.register_entity(heli_entity, ControlEntityType::Helicopter);
    }
    for f16_entity in f16_query.iter() {
        control_manager.register_entity(f16_entity, ControlEntityType::Aircraft);
    }
    
    // Process controls for each active entity
    for (entity, velocity, transform) in active_entities.iter() {
        match control_manager.process_entity_controls(
            entity,
            Some(&input_manager),
            Some(&**current_state),
            velocity,
            transform,
        ) {
            Ok(_controls) => {
                // Controls successfully processed and stored in control_manager
            }
            Err(e) => {
                error!("Control processing failed for entity {:?}: {}", entity, e);
                control_manager.clear_all_controls();
            }
        }
    }
}

/// Legacy system - kept for backwards compatibility but now uses unified control manager
pub fn control_action_system(
    input_manager: Res<InputManager>,
    mut control_manager: ResMut<ControlManager>,
    current_state: Res<State<GameState>>,
    // Player query
    player_query: Query<(&Velocity, &Transform), (With<ActiveEntity>, Without<Car>)>,
    // Car queries
    car_query: Query<(&Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCarSpecs>)>,
    supercar_query: Query<(&Velocity, &Transform), (With<SuperCarSpecs>, With<ActiveEntity>)>,
) {
    // Determine current active entity and get velocity/transform
    let (velocity, transform) = if let Ok((vel, trans)) = player_query.single() {
        (vel, trans)
    } else if let Ok((vel, trans)) = supercar_query.single() {
        (vel, trans)
    } else if let Ok((vel, trans)) = car_query.single() {
        (vel, trans)
    } else {
        return; // No active entity found
    };
    
    // Update control state using legacy method
    if let Err(e) = control_manager.update_controls(
        &input_manager,
        &**current_state,
        velocity,
        transform,
    ) {
        error!("Control update failed: {}", e);
        control_manager.clear_all_controls();
    }
}

/// System to validate control inputs
pub fn control_validation_system(
    mut control_manager: ResMut<ControlManager>,
    current_state: Res<State<GameState>>,
    player_query: Query<(&Velocity, &Transform), (With<ActiveEntity>, Without<Car>)>,
    car_query: Query<(&Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCarSpecs>)>,
    supercar_query: Query<(&Velocity, &Transform), (With<SuperCarSpecs>, With<ActiveEntity>)>,
) {
    // Get current active entity velocity/transform
    let (velocity, transform) = if let Ok((vel, trans)) = player_query.single() {
        (vel, trans)
    } else if let Ok((vel, trans)) = supercar_query.single() {
        (vel, trans)
    } else if let Ok((vel, trans)) = car_query.single() {
        (vel, trans)
    } else {
        return;
    };
    
    // Convert GameState to VehicleType and get physics config
    let vehicle_type = ExistingVehicleControlConfig::game_state_to_vehicle_type(&**current_state);
    let physics_config = if let Some(cfg) = control_manager.get_physics_config(vehicle_type) {
        cfg.clone()
    } else {
        return;
    };
    
    // Validate current control state
    if let Err(e) = control_manager.validate_and_clamp_controls(&physics_config, velocity, transform) {
        warn!("Control validation failed: {}", e);
        control_manager.clear_all_controls();
    }
}

/// Helper functions for easy access to control values
pub fn get_control_value(control_manager: &ControlManager, action: ControlAction) -> f32 {
    control_manager.get_control_value(action)
}

pub fn is_accelerating(control_manager: &ControlManager) -> bool {
    control_manager.is_control_active(ControlAction::Accelerate)
}

pub fn is_braking(control_manager: &ControlManager) -> bool {
    control_manager.is_control_active(ControlAction::Brake)
}

pub fn get_steering_input(control_manager: &ControlManager) -> f32 {
    control_manager.get_control_value(ControlAction::Steer)
}

pub fn is_turbo_active(control_manager: &ControlManager) -> bool {
    control_manager.is_control_active(ControlAction::Turbo)
}

pub fn is_afterburner_active(control_manager: &ControlManager) -> bool {
    control_manager.is_control_active(ControlAction::Afterburner)
}

pub fn get_pitch_input(control_manager: &ControlManager) -> f32 {
    control_manager.get_control_value(ControlAction::Pitch)
}

pub fn get_roll_input(control_manager: &ControlManager) -> f32 {
    control_manager.get_control_value(ControlAction::Roll)
}

pub fn get_yaw_input(control_manager: &ControlManager) -> f32 {
    control_manager.get_control_value(ControlAction::Yaw)
}

pub fn get_throttle_input(control_manager: &ControlManager) -> f32 {
    control_manager.get_control_value(ControlAction::Throttle)
}

/// System to process NPC AI decisions and convert them to control actions
pub fn npc_ai_control_system(
    time: Res<Time>,
    mut control_manager: ResMut<ControlManager>,
    mut npc_query: Query<(Entity, &Transform, &mut NPC), Without<ActiveEntity>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPC>)>,
) {
    let current_time = time.elapsed_secs();
    
    // Get player position for AI decision making
    let player_pos = if let Ok(active_transform) = active_query.single() {
        active_transform.translation
    } else {
        Vec3::ZERO
    };
    
    for (entity, transform, mut npc) in npc_query.iter_mut() {
        // Register NPC entity
        control_manager.register_entity(entity, ControlEntityType::NPC);
        
        // Only update NPCs at their specific intervals (staggered updates)
        if current_time - npc.last_update < npc.update_interval {
            continue;
        }
        npc.last_update = current_time;
        
        let current_pos = transform.translation;
        let target_pos = npc.target_position;
        
        // Calculate distance to target
        let distance = current_pos.distance(target_pos);
        
        // Reduce update frequency for distant NPCs
        let distance_to_player = current_pos.distance(player_pos);
        if distance_to_player > 100.0 {
            npc.update_interval = 0.5; // Very slow updates for distant NPCs
        } else if distance_to_player > 50.0 {
            npc.update_interval = 0.2; // Slower updates for far NPCs
        } else {
            npc.update_interval = 0.05; // Normal updates for close NPCs
        }
        
        // Create AI decision
        let mut ai_decision = AIControlDecision::default();
        
        // If close to target, pick a new random target
        if distance < 5.0 {
            npc.target_position = Vec3::new(
                rand::random::<f32>() * 1800.0 - 900.0,
                1.0,
                rand::random::<f32>() * 1800.0 - 900.0,
            );
            ai_decision.action_priority = 0.1; // Low priority when choosing new target
        } else {
            // Move towards target
            let direction = (target_pos - current_pos).normalize();
            ai_decision.movement_direction = direction;
            ai_decision.speed_factor = 1.0;
            
            // Calculate rotation needed
            let current_forward = transform.forward();
            let dot_product = current_forward.dot(direction);
            let cross_product = current_forward.cross(direction);
            
            if dot_product < 0.99 { // Not already facing the right direction
                ai_decision.rotation_target = if cross_product.y > 0.0 { 1.0 } else { -1.0 };
            }
            
            ai_decision.action_priority = distance / 100.0; // Higher priority for closer targets
        }
        
        // Update AI decision in control manager
        control_manager.update_ai_decision(entity, ai_decision);
    }
}
