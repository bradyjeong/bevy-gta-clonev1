use crate::bundles::PlayerPhysicsBundle;
use crate::components::unified_water::{CurrentWaterRegion, UnifiedWaterBody, WaterBodyId};
use crate::components::{
    ActiveEntity, ControlState, HumanAnimation, HumanMovement, Player, SwimmingEvent,
    VehicleControlType,
};
use crate::util::transform_utils::horizontal_forward;
use bevy::math::EulerRot;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::game_state::GameState;

type DetectSwimmingQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Collider,
        &'static Velocity,
        Option<&'static Swimming>,
        &'static CurrentWaterRegion,
    ),
    (With<Player>, With<ActiveEntity>),
>;

type ApplySwimmingStateQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static Collider,
        Option<&'static mut ProneRotation>,
        Option<&'static mut Velocity>,
    ),
    With<Player>,
>;

type EmergencySwimExitQuery<'w, 's> =
    Query<'w, 's, (Entity, Option<&'static mut Velocity>), (With<Player>, With<Swimming>)>;
type SwimVelocityQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static mut Velocity,
        &'static Swimming,
        &'static mut HumanMovement,
        &'static ControlState,
        &'static HumanAnimation,
    ),
    (With<Player>, With<ActiveEntity>),
>;
type SwimAnimQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Swimming,
        &'static mut HumanAnimation,
        &'static HumanMovement,
        &'static ControlState,
    ),
    (With<Player>, With<ActiveEntity>),
>;
type ProneRotationQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static mut ProneRotation,
        Option<&'static ExitingSwim>,
    ),
    (With<Player>, With<ActiveEntity>),
>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SwimState {
    Surface,
    Diving,
}

#[derive(Component)]
pub struct Swimming {
    pub state: SwimState,
}

#[derive(Component)]
pub struct ProneRotation {
    pub target_pitch: f32,  // -π/2 for prone, 0 for upright
    pub current_pitch: f32, // interpolated pitch value
    pub going_prone: bool,  // true -> prone, false -> stand up
}

#[derive(Component)]
pub struct ExitingSwim;

/// Detect swimming conditions and send events (READ-ONLY)
/// Uses CurrentWaterRegion cache for O(1) lookup instead of O(N) scanning
pub fn detect_swimming_conditions(
    mut events: EventWriter<SwimmingEvent>,
    time: Res<Time<Fixed>>,
    players: DetectSwimmingQuery,
    water_regions: Query<&UnifiedWaterBody>,
) {
    let now = time.elapsed_secs();

    for (entity, transform, collider, velocity, swimming, current_region) in players.iter() {
        let pos = transform.translation;

        // O(1) cache lookup instead of O(N) iteration
        let water = current_region
            .region_entity
            .and_then(|region_entity| water_regions.get(region_entity).ok());

        let half_ext = if let Some(cuboid) = collider.as_cuboid() {
            Vec3::new(
                cuboid.half_extents().x,
                cuboid.half_extents().y,
                cuboid.half_extents().z,
            )
        } else if let Some(capsule) = collider.as_capsule() {
            let total_half_height = capsule.half_height() + capsule.radius();
            Vec3::new(capsule.radius(), total_half_height, capsule.radius())
        } else {
            Vec3::splat(0.5)
        };

        match (swimming, water) {
            (None, Some(w)) => {
                let water_level = w.get_base_water_level(now);
                let head_y = pos.y + half_ext.y;
                if head_y < water_level {
                    let depth = water_level - pos.y;
                    events.write(SwimmingEvent::EnterWater { entity, depth });
                }
            }
            (Some(_), None) => {
                events.write(SwimmingEvent::ExitWater { entity });
            }
            (Some(_), Some(w)) => {
                let entity_bottom = pos.y - half_ext.y;
                let water_level = w.get_base_water_level(now);
                let exit_margin = 0.3;
                let feet_on_ground = entity_bottom > water_level - exit_margin;

                if feet_on_ground {
                    events.write(SwimmingEvent::ExitWater { entity });
                } else {
                    let depth = water_level - pos.y;
                    events.write(SwimmingEvent::UpdateDepth {
                        entity,
                        depth,
                        velocity: velocity.linvel,
                    });
                }
            }
            _ => {}
        }
    }
}

/// Apply swimming state changes (WRITE-ONLY)
pub fn apply_swimming_state(
    mut commands: Commands,
    mut events: EventReader<SwimmingEvent>,
    mut state: ResMut<NextState<GameState>>,
    mut query: ApplySwimmingStateQuery,
    water_regions: Query<&UnifiedWaterBody>,
    time: Res<Time<Fixed>>,
) {
    let now = time.elapsed_secs();

    for event in events.read() {
        match event {
            SwimmingEvent::EnterWater { entity, depth } => {
                commands
                    .entity(*entity)
                    .insert(Swimming {
                        state: SwimState::Surface,
                    })
                    .insert(GravityScale(0.1))
                    .insert(Damping {
                        linear_damping: 6.0,
                        angular_damping: 3.0,
                    })
                    .insert(WaterBodyId)
                    .insert(VehicleControlType::Swimming)
                    .insert(ProneRotation {
                        target_pitch: -std::f32::consts::FRAC_PI_2,
                        current_pitch: 0.0,
                        going_prone: true,
                    });
                state.set(GameState::Swimming);
                debug!("Player entered swimming mode (depth: {:.1})", depth);
            }
            SwimmingEvent::ExitWater { entity } => {
                if let Ok((_, _, prone_rotation, velocity)) = query.get_mut(*entity) {
                    if let Some(mut vel) = velocity {
                        vel.linvel.y = vel.linvel.y.clamp(-1.0, 2.0);
                    }

                    if let Some(mut prone) = prone_rotation {
                        prone.going_prone = false;
                        prone.target_pitch = 0.0;
                    }

                    commands
                        .entity(*entity)
                        .remove::<Swimming>()
                        .remove::<GravityScale>()
                        .remove::<Damping>()
                        .remove::<WaterBodyId>()
                        .insert(VehicleControlType::Walking)
                        .insert(PlayerPhysicsBundle::default());
                    state.set(GameState::Walking);
                    debug!("Player exited swimming mode");
                }
            }
            SwimmingEvent::UpdateDepth { entity, .. } => {
                if let Ok((transform, collider, _, _)) = query.get(*entity) {
                    let half_ext = if let Some(cuboid) = collider.as_cuboid() {
                        Vec3::new(
                            cuboid.half_extents().x,
                            cuboid.half_extents().y,
                            cuboid.half_extents().z,
                        )
                    } else if let Some(capsule) = collider.as_capsule() {
                        let total_half_height = capsule.half_height() + capsule.radius();
                        Vec3::new(capsule.radius(), total_half_height, capsule.radius())
                    } else {
                        Vec3::splat(0.5)
                    };

                    let pos = transform.translation;
                    let water = water_regions
                        .iter()
                        .find(|w| w.contains_point(pos.x, pos.z));

                    if let Some(w) = water {
                        let head_y = pos.y + half_ext.y;
                        let water_level = w.get_base_water_level(now);
                        let new_state = if head_y < water_level - 0.2 {
                            SwimState::Diving
                        } else {
                            SwimState::Surface
                        };
                        commands
                            .entity(*entity)
                            .insert(Swimming { state: new_state });
                    }
                }
            }
        }
    }
}

/// Emergency reset with F2 key
pub fn emergency_swim_exit_system(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: EmergencySwimExitQuery,
) {
    if keyboard_input.just_pressed(KeyCode::F2) {
        for (entity, velocity) in &mut query {
            if let Some(mut vel) = velocity {
                vel.linvel.y = vel.linvel.y.clamp(-1.0, 2.0);
            }

            commands
                .entity(entity)
                .remove::<Swimming>()
                .remove::<GravityScale>()
                .remove::<Damping>()
                .remove::<WaterBodyId>()
                .insert(VehicleControlType::Walking)
                .insert(ExitingSwim)
                .insert(PlayerPhysicsBundle::default());
            state.set(GameState::Walking);
            debug!("Emergency exit from swimming mode with F2");
        }
    }
}

/// 3D swimming movement with biomechanical arm/leg contributions
pub fn swim_velocity_apply_system(
    time: Res<Time>,
    mut query: SwimVelocityQuery,
    water_regions: Query<&UnifiedWaterBody>,
) {
    let Ok((transform, mut vel, swim, mut move_data, control_state, animation)) =
        query.single_mut()
    else {
        return;
    };

    let base_speed = match swim.state {
        SwimState::Surface => 3.0, // Surface swimming speed
        SwimState::Diving => 2.5,  // Underwater speed
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
        let left_arm_power = (stroke_phase * 0.5).max(0.0); // Power only on downstroke
        let right_arm_power = ((stroke_phase + std::f32::consts::PI) * 0.5).max(0.0); // Alternating
        (left_arm_power + right_arm_power) * 0.8 // Arm contribution factor
    } else {
        0.0
    };

    // Leg kick power (flutter kick contributes to forward movement and vertical stability)
    let leg_power = if dir.length() > 0.0 || control_state.vertical.abs() > 0.0 {
        // Flutter kick provides propulsion and vertical control
        let left_kick_power = (kick_phase * 0.3).abs();
        let right_kick_power = ((kick_phase + std::f32::consts::PI * 0.5) * 0.3).abs();
        (left_kick_power + right_kick_power) * 0.6 // Leg contribution factor
    } else {
        0.0
    };

    // Total swimming efficiency from biomechanics
    let swimming_efficiency = 1.0 + arm_power + leg_power;
    let effective_speed = base_speed * swimming_efficiency;

    // IMPROVED: Smart vertical movement relative to water surface
    let pos = transform.translation;
    let water = water_regions
        .iter()
        .find(|w| w.contains_point(pos.x, pos.z));

    let mut vertical_velocity = 0.0;
    if let Some(water) = water {
        let water_level = water.get_water_surface_level(time.elapsed_secs());
        let current_depth = water_level - pos.y; // Positive = underwater, negative = above water

        // Leg kick also contributes to vertical movement
        let leg_vertical_power = leg_power * 0.5; // Legs help with vertical movement

        if control_state.vertical > 0.0 {
            // W key - swim toward surface with leg assistance
            if current_depth > 0.3 {
                vertical_velocity = 2.0 + leg_vertical_power; // Fast ascent when deep
            } else if current_depth > -0.2 {
                vertical_velocity = 0.5 + leg_vertical_power * 0.5; // Slow ascent near surface
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
                vertical_velocity = 0.3 + stabilization; // Slight buoyancy if too deep
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
    vel.angvel = Vec3::new(0.0, control_state.steering * 1.2, 0.0); // Simple steering only

    move_data.current_speed = target_velocity.length();
    move_data.target_velocity = target_velocity;

    // Debug biomechanical swimming with orientation
    #[cfg(feature = "debug-ui")]
    if (time.elapsed_secs() % 2.0) < 0.016 {
        debug!(
            "BIOMECH SWIM: arm_power={:.2}, leg_power={:.2}, efficiency={:.2}, speed={:.2}",
            arm_power, leg_power, swimming_efficiency, effective_speed
        );
    }
}

/// Set swimming animation flags and update swimming animation timing with input responsiveness
pub fn swim_animation_flag_system(time: Res<Time>, mut query: SwimAnimQuery) {
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
        let input_intensity =
            (control_state.throttle + control_state.brake + control_state.vertical.abs()).min(1.0);
        let input_multiplier = 0.8 + (input_intensity * 0.7); // 0.8x to 1.5x based on input

        // Speed-based frequency adjustment (faster when moving faster)
        let speed_multiplier = 1.0 + (movement.current_speed / 5.0).min(0.5);

        anim.swim_stroke_frequency =
            2.0 * std::f32::consts::PI * base_frequency * input_multiplier * speed_multiplier;

        // Update cycle counters with responsive timing
        let dt = time.delta_secs();
        anim.swim_stroke_cycle += dt * input_multiplier; // Faster animation with more input
        anim.swim_kick_cycle += dt * input_multiplier;

        // Debug biomechanical animation status
        #[cfg(feature = "debug-ui")]
        if (time.elapsed_secs() % 2.0) < 0.016 {
            debug!(
                "BIOMECH ANIM: is_swimming={}, input_intensity={:.2}, stroke_freq={:.2}, speed={:.2}",
                anim.is_swimming,
                input_intensity,
                anim.swim_stroke_frequency,
                movement.current_speed
            );
        }
    }
}

/// Apply prone rotation to player transform when swimming
pub fn apply_prone_rotation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: ProneRotationQuery,
) {
    let Ok((entity, mut transform, mut prone, exiting_swim)) = query.single_mut() else {
        return;
    };

    // If exiting swimming, signal return to upright
    if exiting_swim.is_some() && prone.going_prone {
        prone.going_prone = false;
        prone.target_pitch = 0.0; // Return to upright
        commands.entity(entity).remove::<ExitingSwim>(); // Remove the signal
        debug!("Signaled return to upright position after land exit");
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
        if pitch_diff < 0.01 {
            // Close enough
            prone.current_pitch = prone.target_pitch; // Snap to exact target
            commands.entity(entity).remove::<ProneRotation>(); // Remove component when done
            debug!("Player returned to upright position");
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
                debug!("Reset swimming animation - player is on land");
            }
        }
    }
}
