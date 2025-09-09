use bevy::prelude::*;
use bevy::math::EulerRot;
use bevy_rapier3d::prelude::*;
use crate::components::{Player, ActiveEntity, HumanAnimation, HumanMovement, VehicleControlType, ControlState};
use crate::components::unified_water::{UnifiedWaterBody, WaterBodyId};
use crate::util::transform_utils::horizontal_forward;
use crate::bundles::PlayerPhysicsBundle;

use crate::game_state::GameState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SwimState { Surface, Diving }

#[derive(Component)]
pub struct Swimming {
    pub state: SwimState,
}

#[derive(Component)]
pub struct ProneRotation {
    pub target_pitch: f32,      // -π/2 for prone, 0 for upright  
    pub current_pitch: f32,     // interpolated pitch value
    pub going_prone: bool,      // true -> prone, false -> stand up
}

#[derive(Component)]
pub struct ExitingSwim;

const ENTER_THRESHOLD: f32 = 0.10;  // >10% submerged → enter (DEBUG: lowered)
const EXIT_THRESHOLD: f32 = 0.05;   // <5% submerged → exit (DEBUG: lowered)

/// State machine with hysteresis - prevents flicker
pub fn swim_state_transition_system(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &Transform, &Collider, Option<&Swimming>, Option<&mut ProneRotation>, Option<&mut Velocity>), (With<Player>, With<ActiveEntity>)>,
    water_regions: Query<&UnifiedWaterBody>,
) {
    let now = time.elapsed_secs();

    // Emergency reset with F2 key
    if keyboard_input.just_pressed(KeyCode::F2) {
        for (entity, _, _, swimming, _prone_rotation, velocity) in &mut query {
            if swimming.is_some() {
                // Clamp velocity to prevent physics spikes on emergency exit
                if let Some(mut vel) = velocity {
                    if vel.linvel.y > 2.0 { vel.linvel.y = 2.0; }
                    if vel.linvel.y < -1.0 { vel.linvel.y = -1.0; }
                }
                
                commands.entity(entity)
                    .remove::<Swimming>()
                    .remove::<GravityScale>()
                    .remove::<Damping>()
                    .remove::<WaterBodyId>()
                    .insert(VehicleControlType::Walking)
                    .insert(ExitingSwim) // Signal smooth return to upright
                    .insert(PlayerPhysicsBundle::default()); // Restore clean physics state
                state.set(GameState::Walking);
                info!("Emergency exit from swimming mode with F2");
            }
        }
        return;
    }

    for (entity, transform, collider, swimming, prone_rotation, velocity) in &mut query {
        let pos = transform.translation;
        
        // DEBUG: Log player position and water regions
        if (now % 2.0) < 0.016 { // Every ~2 seconds
            info!("Player at position: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
            info!("Checking {} water regions:", water_regions.iter().count());
            for (i, region) in water_regions.iter().enumerate() {
                info!("  Region {}: '{}' bounds=({:.1}, {:.1}) to ({:.1}, {:.1})", 
                      i, region.name, region.bounds.0, region.bounds.1, region.bounds.2, region.bounds.3);
                let in_region = region.contains_point(pos.x, pos.z);
                info!("    Player in region: {}", in_region);
            }
        }
        
        let water = water_regions.iter().find(|w| w.contains_point(pos.x, pos.z));
        
        // Calculate submersion ratio first - always check submersion for hysteresis
        let half_ext = collider.as_cuboid()
            .map(|c| Vec3::new(c.half_extents().x, c.half_extents().y, c.half_extents().z))
            .unwrap_or(Vec3::splat(0.5));
            
        let submersion = if let Some(w) = water {
            // DEBUG: Found water region
            if (now % 2.0) < 0.016 {
                info!("Player is in water region: {}", w.name);
            }
            w.calculate_submersion_ratio(transform, half_ext, now)
        } else {
            // Not in any water region - treat as completely out of water
            0.0
        };
        
        // DEBUG: Log submersion ratio and water level details
        if (now % 1.0) < 0.016 { // More frequent logging
            if let Some(w) = water {
                let water_level = w.get_water_surface_level(now);
                let entity_bottom = pos.y - half_ext.y;
                let entity_top = pos.y + half_ext.y;
                info!("🌊 WATER DEBUG:");
                info!("  Water level: {:.3}, Player Y: {:.3}", water_level, pos.y);
                info!("  Player bottom: {:.3}, Player top: {:.3}", entity_bottom, entity_top);
                info!("  Player half extents: ({:.3}, {:.3}, {:.3})", half_ext.x, half_ext.y, half_ext.z);
                info!("  Submersion ratio: {:.3} (need {:.2} to enter)", submersion, ENTER_THRESHOLD);
                
                if submersion > 0.0 {
                    info!("  🏊 SUBMERSION DETECTED! Ratio: {:.3}", submersion);
                }
                if entity_bottom < water_level {
                    info!("  👣 FEET IN WATER! Bottom: {:.3} < Water: {:.3}", entity_bottom, water_level);
                }
            }
        }

        match swimming {
            None => {
                if submersion > ENTER_THRESHOLD && water.is_some() {
                    // ENTER SWIM MODE
                    commands.entity(entity)
                        .insert(Swimming { state: SwimState::Surface })
                        .insert(GravityScale(0.1))  // Light gravity to prevent floating
                        .insert(Damping { linear_damping: 6.0, angular_damping: 3.0 }) // Higher damping in water
                        .insert(WaterBodyId)  // Enable water physics
                        .insert(VehicleControlType::Swimming)  // Switch to swimming controls
                        .insert(ProneRotation { 
                            target_pitch: -std::f32::consts::FRAC_PI_2, // -90° for prone
                            current_pitch: 0.0,                         // start upright
                            going_prone: true,
                        });
                    state.set(GameState::Swimming);  // Update UI state
                    info!("Player entered swimming mode at {:.1}% submersion", submersion * 100.0);
                }
            }
            Some(swim) => {
                // Exit based on submersion hysteresis - prevents flicker
                if submersion < EXIT_THRESHOLD {
                    // Clamp velocity to prevent physics spikes on exit
                    if let Some(mut vel) = velocity {
                        if vel.linvel.y > 2.0 { vel.linvel.y = 2.0; }
                        if vel.linvel.y < -1.0 { vel.linvel.y = -1.0; }
                    }
                    
                    // Signal ProneRotation to return to upright position
                    if let Some(mut prone) = prone_rotation {
                        prone.going_prone = false; // Signal to return to upright
                        prone.target_pitch = 0.0;  // Return to upright
                    }
                    
                    commands.entity(entity)
                        .remove::<Swimming>()
                        .remove::<GravityScale>()
                        .remove::<Damping>()
                        .remove::<WaterBodyId>()
                        .insert(VehicleControlType::Walking)  // Switch back to walking controls
                        .insert(PlayerPhysicsBundle::default()); // Restore clean physics state
                    state.set(GameState::Walking);  // Update UI state
                    info!("Player exited swimming mode at {:.1}% submersion", submersion * 100.0);
                    continue;
                }

                // Surface vs Diving state (only if still in water)
                if let Some(w) = water {
                    let head_y = transform.translation.y + half_ext.y;
                    let water_level = w.get_water_surface_level(now);
                    let new_state = if head_y < water_level - 0.2 {
                        SwimState::Diving
                    } else {
                        SwimState::Surface
                    };
                    
                    if swim.state != new_state {
                        commands.entity(entity).insert(Swimming { state: new_state });
                    }
                }
            }
        }
    }
}

/// 3D swimming movement with biomechanical arm/leg contributions
pub fn swim_velocity_apply_system(
    time: Res<Time>,
    mut query: Query<(&Transform, &mut Velocity, &Swimming, &mut HumanMovement, &ControlState, &HumanAnimation), (With<Player>, With<ActiveEntity>)>,
    water_regions: Query<&UnifiedWaterBody>,
) {
    let Ok((transform, mut vel, swim, mut move_data, control_state, animation)) = query.single_mut() else {
        return;
    };

    let base_speed = match swim.state {
        SwimState::Surface => 3.0,  // Surface swimming speed
        SwimState::Diving => 2.5,   // Underwater speed
    };

    // Horizontal movement (from control state)
    let mut dir = Vec3::ZERO;
    
    // Use horizontal forward direction to handle prone swimming correctly
    let hori_fwd = horizontal_forward(transform);
    
    // Convert control state to movement direction
    if control_state.throttle > 0.0 {
        dir += hori_fwd;
    }
    if control_state.brake > 0.0 {
        dir -= hori_fwd;
    }
    
    // Apply steering to turn the player direction
    if dir.length_squared() > 0.0 {
        dir = dir.normalize();
    }

    // BIOMECHANICAL SWIMMING: Calculate arm stroke contribution
    let stroke_phase = (time.elapsed_secs() * animation.swim_stroke_frequency).sin();
    let kick_phase = (time.elapsed_secs() * animation.swim_stroke_frequency * 3.0).sin();
    
    // Arm stroke power (freestyle stroke generates forward thrust)
    let arm_power = if dir.length() > 0.0 {
        // Forward stroke: Power stroke contributes to forward movement
        let left_arm_power = (stroke_phase * 0.5).max(0.0);  // Power only on downstroke
        let right_arm_power = ((stroke_phase + std::f32::consts::PI) * 0.5).max(0.0);  // Alternating
        (left_arm_power + right_arm_power) * 0.8  // Arm contribution factor
    } else {
        0.0
    };
    
    // Leg kick power (flutter kick contributes to forward movement and vertical stability)
    let leg_power = if dir.length() > 0.0 || control_state.vertical.abs() > 0.0 {
        // Flutter kick provides propulsion and vertical control
        let left_kick_power = (kick_phase * 0.3).abs();
        let right_kick_power = ((kick_phase + std::f32::consts::PI * 0.5) * 0.3).abs();
        (left_kick_power + right_kick_power) * 0.6  // Leg contribution factor
    } else {
        0.0
    };
    
    // Total swimming efficiency from biomechanics
    let swimming_efficiency = 1.0 + arm_power + leg_power;
    let effective_speed = base_speed * swimming_efficiency;

    // IMPROVED: Smart vertical movement relative to water surface
    let pos = transform.translation;
    let water = water_regions.iter().find(|w| w.contains_point(pos.x, pos.z));
    
    let mut vertical_velocity = 0.0;
    if let Some(water) = water {
        let water_level = water.get_water_surface_level(time.elapsed_secs());
        let current_depth = water_level - pos.y;  // Positive = underwater, negative = above water
        
        // Leg kick also contributes to vertical movement
        let leg_vertical_power = leg_power * 0.5;  // Legs help with vertical movement
        
        if control_state.vertical > 0.0 {
            // W key - swim toward surface with leg assistance
            if current_depth > 0.3 {
                vertical_velocity = (2.0 + leg_vertical_power);  // Fast ascent when deep
            } else if current_depth > -0.2 {
                vertical_velocity = (0.5 + leg_vertical_power * 0.5);  // Slow ascent near surface
            } else {
                vertical_velocity = -0.5; // Gentle sink if too high above water
            }
        } else if control_state.vertical < 0.0 {
            // S key - dive down with leg assistance
            vertical_velocity = -(2.0 + leg_vertical_power);
        } else {
            // No input - gentle drift toward surface level with leg stabilization
            let stabilization = leg_vertical_power * 0.2;
            if current_depth < -0.5 {
                vertical_velocity = -1.0 + stabilization; // Sink down if floating too high
            } else if current_depth > 0.1 {
                vertical_velocity = 0.3 + stabilization;  // Slight buoyancy if too deep
            } else {
                vertical_velocity = stabilization; // Leg movements help maintain position
            }
        }
    }

    let target_velocity = Vec3::new(
        dir.x * effective_speed,
        vertical_velocity,
        dir.z * effective_speed,
    );

    // Apply 3D swimming movement with biomechanical contributions
    vel.linvel = target_velocity;
    vel.angvel = Vec3::new(0.0, control_state.steering * 1.2, 0.0);  // Simple steering only

    move_data.current_speed = target_velocity.length();
    move_data.target_velocity = target_velocity;
    
    // Debug biomechanical swimming with orientation
    if (time.elapsed_secs() % 2.0) < 0.016 {
        info!("🏊‍♂️ BIOMECH SWIM: arm_power={:.2}, leg_power={:.2}, efficiency={:.2}, speed={:.2}", 
              arm_power, leg_power, swimming_efficiency, effective_speed);
    }
}

/// Set swimming animation flags and update swimming animation timing with input responsiveness
pub fn swim_animation_flag_system(
    time: Res<Time>,
    mut query: Query<(&Swimming, &mut HumanAnimation, &HumanMovement, &ControlState), (With<Player>, With<ActiveEntity>)>,
) {
    if let Ok((swim, mut anim, movement, control_state)) = query.single_mut() {
        anim.is_swimming = true;
        anim.is_walking = false;
        anim.is_running = false;
        
        // Update swimming animation timing based on speed and state
        anim.swim_speed = movement.current_speed;
        
        // BIOMECHANICAL RESPONSIVE ANIMATION: Stroke frequency responds to input intensity
        let base_frequency = match swim.state {
            SwimState::Surface => 0.8, // Normal surface stroke rate
            SwimState::Diving => 0.6,  // Slower strokes when diving
        };
        
        // Input-based frequency adjustment (more active when player gives input)
        let input_intensity = (control_state.throttle + control_state.brake + control_state.vertical.abs()).min(1.0);
        let input_multiplier = 0.8 + (input_intensity * 0.7);  // 0.8x to 1.5x based on input
        
        // Speed-based frequency adjustment (faster when moving faster)
        let speed_multiplier = 1.0 + (movement.current_speed / 5.0).min(0.5);
        
        anim.swim_stroke_frequency = 2.0 * std::f32::consts::PI * base_frequency * input_multiplier * speed_multiplier;
        
        // Update cycle counters with responsive timing
        let dt = time.delta_secs();
        anim.swim_stroke_cycle += dt * input_multiplier;  // Faster animation with more input
        anim.swim_kick_cycle += dt * input_multiplier;
        
        // Debug biomechanical animation status
        if (time.elapsed_secs() % 2.0) < 0.016 {
            info!("🏊 BIOMECH ANIM: is_swimming={}, input_intensity={:.2}, stroke_freq={:.2}, speed={:.2}", 
                  anim.is_swimming, input_intensity, anim.swim_stroke_frequency, movement.current_speed);
        }
    }
}

/// Apply prone rotation to player transform when swimming
pub fn apply_prone_rotation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ProneRotation, Option<&ExitingSwim>), (With<Player>, With<ActiveEntity>)>,
) {
    let Ok((entity, mut transform, mut prone, exiting_swim)) = query.single_mut() else {
        return;
    };
    
    // If exiting swimming, signal return to upright
    if exiting_swim.is_some() && prone.going_prone {
        prone.going_prone = false;
        prone.target_pitch = 0.0;  // Return to upright
        commands.entity(entity).remove::<ExitingSwim>(); // Remove the signal
        info!("Signaled return to upright position after land exit");
    }

    // Interpolate only the pitch, preserve physics-driven yaw
    let rate = 5.0; // How quickly to rotate
    let dt = time.delta_secs();
    let t = 1.0 - (-rate * dt).exp(); // Maps dt to (0,1)
    
    // Interpolate pitch towards target
    prone.current_pitch = prone.current_pitch + (prone.target_pitch - prone.current_pitch) * t;
    
    // Extract current yaw (let physics handle it) and apply interpolated pitch
    let (yaw, _old_pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, prone.current_pitch, roll);
    
    // Check if we're close enough to the target when returning to upright
    if !prone.going_prone {
        let pitch_diff = (prone.current_pitch - prone.target_pitch).abs();
        if pitch_diff < 0.01 { // Close enough 
            prone.current_pitch = prone.target_pitch; // Snap to exact target
            commands.entity(entity).remove::<ProneRotation>(); // Remove component when done
            info!("Player returned to upright position");
        }
    }
}

/// Reset animation flags when on land
pub fn reset_animation_on_land_system(
    swim_query: Query<(), (With<Player>, With<Swimming>)>,
    mut anim_query: Query<&mut HumanAnimation, (With<Player>, Without<Swimming>)>,
) {
    if swim_query.is_empty() {
        if let Ok(mut anim) = anim_query.single_mut() {
            // Only reset if currently swimming to avoid spam
            if anim.is_swimming {
                anim.is_swimming = false;
                anim.swim_stroke_cycle = 0.0;
                anim.swim_kick_cycle = 0.0;
                anim.swim_speed = 0.0;
                info!("Reset swimming animation - player is on land");
            }
        }
    }
}
