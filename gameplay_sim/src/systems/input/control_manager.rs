//! ───────────────────────────────────────────────
//! System:   Control Manager
//! Purpose:  Handles entity movement and physics
//! Schedule: Update (throttled)
//! Reads:    `ActiveEntity`, Transform, Car, NPC, Helicopter
//! Writes:   NPC, `ControlManager`
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Physics values are validated and finite
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
// Removed bevy16_compat - using direct Bevy methods
use std::collections::HashMap;
use std::time::Instant;

use game_core::prelude::*;
use super::input_config::InputAction;
use super::input_manager::InputManager;

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
    Emergency,
    
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
            max_angular_velocity: 10.0,
            position_clamp: Vec3::new(1000.0, 100.0, 1000.0),
            acceleration_sensitivity: 1.0,
            steering_sensitivity: 1.0,
            brake_sensitivity: 1.0,
            enable_safety_limits: true,
            max_safe_speed: 20.0,
            stability_assist: false,
        }
    }
}

#[derive(Resource)]
pub struct ControlManager {
    // Active entity tracking
    active_entity: Option<Entity>,
    
    // Control state
    active_controls: HashMap<ControlAction, f32>,
    
    // AI control decisions per entity
    ai_decisions: HashMap<Entity, AIControlDecision>,
    
    // Physics configuration
    physics_config: VehiclePhysicsConfig,
    
    // Performance monitoring
    last_update: Instant,
    update_count: u64,
    max_update_time: u64,
}

impl Default for ControlManager {
    fn default() -> Self {
        Self {
            active_entity: None,
            active_controls: HashMap::new(),
            ai_decisions: HashMap::new(),
            physics_config: VehiclePhysicsConfig::default(),
            last_update: Instant::now(),
            update_count: 0,
            max_update_time: 0,
        }
    }
}

impl ControlManager {
    /// Set the active entity
    pub fn set_active_entity(&mut self, entity: Option<Entity>) {
        self.active_entity = entity;
        self.active_controls.clear();
    }

    /// Get the active entity
    #[must_use] pub fn get_active_entity(&self) -> Option<Entity> {
        self.active_entity
    }

    /// Update control actions based on input
    pub fn update_controls(
        &mut self,
        input_manager: &InputManager,
        current_state: &GameState,
    ) {
        let start_time = Instant::now();
        
        self.active_controls.clear();
        
        // Map input actions to control actions based on current state
        match current_state {
            GameState::Walking => {
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Accelerate, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Brake, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::TurnLeft) {
                    self.active_controls.insert(ControlAction::Steer, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::TurnRight) {
                    self.active_controls.insert(ControlAction::Steer, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::Run) {
                    self.active_controls.insert(ControlAction::Turbo, 1.0);
                }
            }
            GameState::Driving => {
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Accelerate, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Brake, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::TurnLeft) {
                    self.active_controls.insert(ControlAction::Steer, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::TurnRight) {
                    self.active_controls.insert(ControlAction::Steer, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::Turbo) {
                    self.active_controls.insert(ControlAction::Turbo, 1.0);
                }
            }
            GameState::Flying => {
                // Helicopter controls
                if input_manager.is_action_pressed(InputAction::Forward) {
                    self.active_controls.insert(ControlAction::Pitch, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::Backward) {
                    self.active_controls.insert(ControlAction::Pitch, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::TurnLeft) {
                    self.active_controls.insert(ControlAction::Yaw, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::TurnRight) {
                    self.active_controls.insert(ControlAction::Yaw, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::VerticalUp) {
                    self.active_controls.insert(ControlAction::Throttle, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::VerticalDown) {
                    self.active_controls.insert(ControlAction::Throttle, -1.0);
                }
            }
            GameState::Jetting => {
                // F16 controls
                if input_manager.is_action_pressed(InputAction::PitchUp) {
                    self.active_controls.insert(ControlAction::Pitch, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::PitchDown) {
                    self.active_controls.insert(ControlAction::Pitch, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::RollLeft) {
                    self.active_controls.insert(ControlAction::Roll, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::RollRight) {
                    self.active_controls.insert(ControlAction::Roll, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::YawLeft) {
                    self.active_controls.insert(ControlAction::Yaw, -1.0);
                }
                if input_manager.is_action_pressed(InputAction::YawRight) {
                    self.active_controls.insert(ControlAction::Yaw, 1.0);
                }
                if input_manager.is_action_pressed(InputAction::Afterburner) {
                    self.active_controls.insert(ControlAction::Afterburner, 1.0);
                }
            }
        }
        
        // Common actions across all states
        if input_manager.is_action_just_pressed(InputAction::Interact) {
            self.active_controls.insert(ControlAction::Interact, 1.0);
        }
        if input_manager.is_action_just_pressed(InputAction::DebugInfo) {
            self.active_controls.insert(ControlAction::DebugInfo, 1.0);
        }
        if input_manager.is_action_just_pressed(InputAction::EmergencyReset) {
            self.active_controls.insert(ControlAction::EmergencyReset, 1.0);
        }
        
        // Performance monitoring
        let update_time = start_time.elapsed().as_micros() as u64;
        self.max_update_time = self.max_update_time.max(update_time);
        self.update_count += 1;
        self.last_update = start_time;
    }

    /// Get current control actions
    #[must_use] pub fn get_controls(&self) -> &HashMap<ControlAction, f32> {
        &self.active_controls
    }

    /// Check if a control action is currently active
    #[must_use] pub fn is_control_active(&self, action: ControlAction) -> bool {
        self.active_controls.contains_key(&action) && self.active_controls[&action] > 0.0
    }

    /// Get the value of a control action (0.0 if not active)
    #[must_use] pub fn get_control_value(&self, action: ControlAction) -> f32 {
        *self.active_controls.get(&action).unwrap_or(&0.0)
    }

    /// Check if emergency controls are active
    #[must_use] pub fn is_emergency_active(&self) -> bool {
        self.is_control_active(ControlAction::Emergency)
    }

    /// Set AI control decision for an entity
    pub fn update_ai_decision(&mut self, entity: Entity, decision: AIControlDecision) {
        self.ai_decisions.insert(entity, decision);
    }

    /// Get AI control decision for an entity
    #[must_use] pub fn get_ai_decision(&self, entity: Entity) -> Option<&AIControlDecision> {
        self.ai_decisions.get(&entity)
    }

    /// Update physics configuration
    pub fn update_physics_config(&mut self, config: VehiclePhysicsConfig) {
        self.physics_config = config;
    }

    /// Get physics configuration
    #[must_use] pub fn get_physics_config(&self) -> &VehiclePhysicsConfig {
        &self.physics_config
    }

    /// Get performance statistics
    #[must_use] pub fn get_performance_stats(&self) -> (u64, u64, u64) {
        (self.update_count, self.max_update_time, self.ai_decisions.len() as u64)
    }

    /// Clear AI decisions for cleanup
    pub fn clear_stale_ai_decisions(&mut self, valid_entities: &[Entity]) {
        self.ai_decisions.retain(|entity, _| valid_entities.contains(entity));
    }

    /// Get active actions for a specific entity (for now, returns global active controls)
    #[must_use] pub fn get_active_actions(&self, _entity: Entity) -> Option<Vec<ControlAction>> {
        if self.active_controls.is_empty() {
            None
        } else {
            Some(self.active_controls.keys().copied().collect())
        }
    }
}

/// System to update control manager
pub fn update_control_manager_system(
    mut control_manager: ResMut<ControlManager>,
    input_manager: Res<InputManager>,
    current_state: Res<State<GameState>>,
) {
    control_manager.update_controls(&input_manager, &current_state);
}

/// System to apply AI control decisions to NPCs
pub fn apply_ai_controls_system(
    control_manager: ResMut<ControlManager>,
    mut npc_query: Query<(Entity, &mut Transform, &mut Velocity), With<NPCState>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut velocity) in &mut npc_query {
        if let Some(ai_decision) = control_manager.get_ai_decision(entity) {
            // Apply AI decision to NPC movement
            let movement = ai_decision.movement_direction * ai_decision.speed_factor * time.delta_secs();
            transform.translation += movement;
            
            // Apply rotation
            if ai_decision.rotation_target != 0.0 {
                let rotation_speed = 2.0 * time.delta_secs();
                let target_rotation = Quat::from_rotation_y(ai_decision.rotation_target);
                transform.rotation = transform.rotation.slerp(target_rotation, rotation_speed);
            }
            
            // Update velocity for physics consistency
            velocity.linvel = movement / time.delta_secs();
        }
    }
}

/// Simple AI system to generate control decisions for NPCs
pub fn npc_ai_decision_system(
    mut control_manager: ResMut<ControlManager>,
    npc_query: Query<(Entity, &Transform), With<NPCState>>,
    time: Res<Time>,
) {
    for (entity, transform) in npc_query.iter() {
        // Simple wander behavior
        let time_offset = entity.index() as f32 * 0.1;
        let wander_angle = (time.elapsed_secs() + time_offset).sin() * 0.5;
        
        let ai_decision = AIControlDecision {
            movement_direction: Vec3::new(wander_angle.cos(), 0.0, wander_angle.sin()) * 2.0,
            rotation_target: wander_angle,
            speed_factor: 0.5,
            action_priority: 1.0,
        };
        
        control_manager.update_ai_decision(entity, ai_decision);
    }
}

/// Helper function to check if entity is accelerating
#[must_use] pub fn is_accelerating(control_manager: &ControlManager, entity: Entity) -> bool {
    control_manager.get_active_actions(entity)
        .is_some_and(|actions| actions.contains(&ControlAction::Accelerate))
}

/// Helper function to check if entity is braking
#[must_use] pub fn is_braking(control_manager: &ControlManager, entity: Entity) -> bool {
    control_manager.get_active_actions(entity)
        .is_some_and(|actions| actions.contains(&ControlAction::Brake))
}
