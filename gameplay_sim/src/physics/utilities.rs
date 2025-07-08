use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::GameConfig;
use crate::constants::{STATIC_GROUP, VEHICLE_GROUP, CHARACTER_GROUP};
use crate::systems::input::{ControlManager, ControlAction};
use tracing::warn;

/// Unified physics utilities for consistent physics behavior across all movement systems
#[derive(Default)]
pub struct PhysicsUtilities;

impl PhysicsUtilities {
    /// Validate and clamp velocity to safe ranges for physics stability
    pub fn validate_velocity(velocity: &mut Velocity, config: &GameConfig) {
        // Clamp linear velocity to prevent physics instability
        velocity.linvel = velocity.linvel.clamp_length_max(config.physics.max_velocity);
        velocity.angvel = velocity.angvel.clamp_length_max(config.physics.max_angular_velocity);
        
        // Ensure all values are finite
        if !velocity.linvel.is_finite() {
            velocity.linvel = Vec3::ZERO;
        }
        if !velocity.angvel.is_finite() {
            velocity.angvel = Vec3::ZERO;
        }
    }
    
    /// Validate and clamp external force to safe ranges (for vehicles)
    pub fn validate_external_force(force: &mut ExternalForce, max_force: f32) {
        // Clamp force magnitude
        force.force = force.force.clamp_length_max(max_force);
        force.torque = force.torque.clamp_length_max(max_force);
        
        // Ensure values are finite
        if !force.force.is_finite() {
            force.force = Vec3::ZERO;
        }
        if !force.torque.is_finite() {
            force.torque = Vec3::ZERO;
        }
    }
    
    /// Apply force through physics with safety validation
    pub fn apply_force_safe(
        velocity: &mut Velocity, 
        force: Vec3, 
        torque: Vec3, 
        dt: f32,
        max_force: f32
    ) {
        // Validate input forces
        let safe_force = force.clamp_length_max(max_force);
        let safe_torque = torque.clamp_length_max(max_force);
        
        if safe_force.is_finite() && safe_force.length() > 0.01 {
            velocity.linvel += safe_force * dt;
        }
        
        if safe_torque.is_finite() && safe_torque.length() > 0.01 {
            velocity.angvel += safe_torque * dt;
        }
    }
    
    /// Apply external force with validation for vehicle force-based systems
    pub fn apply_external_force_safe(
        external_force: &mut ExternalForce,
        force: Vec3,
        torque: Vec3,
        max_force: f32
    ) {
        // Validate and apply force
        let safe_force = force.clamp_length_max(max_force);
        let safe_torque = torque.clamp_length_max(max_force);
        
        if safe_force.is_finite() {
            external_force.force = safe_force;
        }
        
        if safe_torque.is_finite() {
            external_force.torque = safe_torque;
        }
    }
    
    /// Prevent entity from going underground with soft collision
    pub fn apply_ground_collision(
        velocity: &mut Velocity,
        transform: &Transform,
        min_height: f32,
        bounce_force: f32
    ) {
        if transform.translation.y < min_height {
            // Stop downward movement
            if velocity.linvel.y < 0.0 {
                velocity.linvel.y = 0.0;
            }
            // Add upward force to keep entity above ground
            velocity.linvel.y += bounce_force;
        }
    }
    
    /// Apply world bounds using velocity-based physics (no direct Transform manipulation)
    pub fn apply_world_bounds(
        velocity: &mut Velocity,
        transform: &Transform,
        config: &GameConfig
    ) {
        let bounds = config.physics.max_world_coord;
        
        // Check and apply velocity corrections for X bounds
        if transform.translation.x > bounds {
            velocity.linvel.x = velocity.linvel.x.min(0.0);
        } else if transform.translation.x < -bounds {
            velocity.linvel.x = velocity.linvel.x.max(0.0);
        }
        
        // Check and apply velocity corrections for Z bounds
        if transform.translation.z > bounds {
            velocity.linvel.z = velocity.linvel.z.min(0.0);
        } else if transform.translation.z < -bounds {
            velocity.linvel.z = velocity.linvel.z.max(0.0);
        }
    }
    
    /// Calculate drag force for aerodynamic resistance
    pub fn calculate_drag_force(
        velocity: &Velocity,
        drag_coefficient: f32,
        air_density: f32,
        frontal_area: f32
    ) -> Vec3 {
        let speed_squared = velocity.linvel.length_squared();
        -velocity.linvel.normalize_or_zero() * 
            0.5 * air_density * drag_coefficient * frontal_area * speed_squared
    }
    
    /// Apply realistic deceleration when no input is provided
    pub fn apply_natural_deceleration(
        velocity: &mut Velocity,
        linear_damping: f32,
        angular_damping: f32,
        dt: f32
    ) {
        // Natural deceleration with exponential decay
        let linear_decay = 1.0 - (linear_damping * dt).clamp(0.0, 0.99);
        let angular_decay = 1.0 - (angular_damping * dt).clamp(0.0, 0.99);
        
        velocity.linvel *= linear_decay;
        velocity.angvel *= angular_decay;
    }
    
    /// Interpolate velocity smoothly to target for smooth movement
    pub fn smooth_velocity_transition(
        current_velocity: &mut Velocity,
        target_velocity: Vec3,
        smoothing_factor: f32,
        dt: f32
    ) {
        let lerp_factor = (smoothing_factor * dt).clamp(0.0, 1.0);
        current_velocity.linvel = current_velocity.linvel.lerp(target_velocity, lerp_factor);
    }
}

/// Collision group management utilities
pub struct CollisionGroupHelper;

impl CollisionGroupHelper {
    /// Get collision groups for static objects (buildings, terrain)
    pub fn static_groups() -> CollisionGroups {
        CollisionGroups::new(STATIC_GROUP, Group::ALL)
    }
    
    /// Get collision groups for vehicles (cars, aircraft)
    pub fn vehicle_groups() -> CollisionGroups {
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP)
    }
    
    /// Get collision groups for characters (player, NPCs)
    pub fn character_groups() -> CollisionGroups {
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP)
    }
    
    /// Get collision groups from config for flexible assignment
    pub fn from_config(
        entity_group: Group,
        collision_mask: Group
    ) -> CollisionGroups {
        CollisionGroups::new(entity_group, collision_mask)
    }
}

/// Physics body setup utilities for consistent physics configuration
pub struct PhysicsBodySetup;

impl PhysicsBodySetup {
    /// Create a dynamic physics body for moving entities
    pub fn create_dynamic_body(
        collision_groups: CollisionGroups,
        linear_damping: f32,
        angular_damping: f32
    ) -> (RigidBody, CollisionGroups, Damping) {
        (
            RigidBody::Dynamic,
            collision_groups,
            Damping {
                linear_damping,
                angular_damping,
            }
        )
    }
    
    /// Create a static physics body for immovable objects
    pub fn create_static_body(collision_groups: CollisionGroups) -> (RigidBody, CollisionGroups) {
        (RigidBody::Fixed, collision_groups)
    }
    
    /// Create collider with validation
    pub fn create_collider_safe(
        shape: Collider,
        config: &GameConfig
    ) -> Option<Collider> {
        // Validate collider size using raw collider access
        let aabb = shape.raw.compute_local_aabb();
        let size = aabb.half_extents().magnitude();
        
        if size > config.physics.max_collider_size || size < config.physics.min_collider_size {
            warn!("Collider size {} outside safe range [{}, {}]", 
                  size, config.physics.min_collider_size, config.physics.max_collider_size);
            return None;
        }
        
        Some(shape)
    }
    
    /// Validate and create mass properties
    pub fn create_mass_properties(mass: f32, config: &GameConfig) -> Option<AdditionalMassProperties> {
        let clamped_mass = mass.clamp(config.physics.min_mass, config.physics.max_mass);
        
        if !clamped_mass.is_finite() || clamped_mass <= 0.0 {
            return None;
        }
        
        Some(AdditionalMassProperties::Mass(clamped_mass))
    }
}

/// Unified input processing results for all movement systems
#[derive(Clone, Copy, Debug, Default)]
pub struct MovementInputs {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
    pub thrust: f32,
    pub forward: f32,
    pub backward: f32,
    pub left: f32,
    pub right: f32,
    pub jump: f32,
}

/// Input processing utilities for consistent control handling
pub struct InputProcessor;

impl InputProcessor {
    /// Unified input processing for all movement systems
    pub fn process_unified_inputs(control_manager: &ControlManager) -> MovementInputs {
        MovementInputs {
            throttle: control_manager.get_control_value(ControlAction::Throttle),
            brake: control_manager.get_control_value(ControlAction::Brake),
            steering: control_manager.get_control_value(ControlAction::Steer),
            pitch: control_manager.get_control_value(ControlAction::Pitch),
            roll: control_manager.get_control_value(ControlAction::Roll),
            yaw: control_manager.get_control_value(ControlAction::Yaw),
            thrust: control_manager.get_control_value(ControlAction::Throttle), // Use Throttle for thrust
            forward: control_manager.get_control_value(ControlAction::Accelerate), // Use Accelerate for forward
            backward: control_manager.get_control_value(ControlAction::Brake), // Use Brake for backward
            left: 0.0, // No direct left action, use steering negative
            right: 0.0, // No direct right action, use steering positive  
            jump: 0.0, // No jump action in current ControlAction enum
        }
    }

    /// Process acceleration input with smooth ramping
    pub fn process_acceleration_input(
        current_input: f32,
        target_input: f32,
        ramp_up_rate: f32,
        ramp_down_rate: f32,
        dt: f32
    ) -> f32 {
        let rate = if target_input > current_input { ramp_up_rate } else { ramp_down_rate };
        let change = (target_input - current_input) * rate * dt;
        (current_input + change).clamp(0.0, 1.0)
    }
    
    /// Apply speed-dependent steering sensitivity
    pub fn apply_speed_dependent_steering(
        steering_input: f32,
        current_speed: f32,
        base_sensitivity: f32,
        speed_threshold: f32
    ) -> f32 {
        let speed_factor = (current_speed / speed_threshold).min(1.0);
        let sensitivity = base_sensitivity * (1.0 - speed_factor * 0.6);
        steering_input * sensitivity
    }
    
    /// Calculate force from control input with power curve
    pub fn calculate_force_from_input(
        input_value: f32,
        base_force: f32,
        power_curve: f32
    ) -> f32 {
        base_force * input_value.powf(power_curve)
    }
}

/// Comprehensive physics safety system using velocity-based approach
pub fn apply_universal_physics_safeguards(
    mut query: Query<(Entity, &mut Velocity, &Transform), With<RigidBody>>,
    config: Res<GameConfig>,
) {
    for (_entity, mut velocity, transform) in query.iter_mut() {
        // Apply all safety measures
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_world_bounds(&mut velocity, &transform, &config);
        
        // Additional safety checks for invalid positions (use velocity reset for safety)
        if !transform.translation.is_finite() {
            warn!("Entity had invalid position, stopping all movement");
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
        }
        
        if !transform.rotation.is_finite() {
            warn!("Entity had invalid rotation, stopping angular movement");
            velocity.angvel = Vec3::ZERO;
        }
    }
}
