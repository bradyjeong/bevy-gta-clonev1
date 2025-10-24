#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::water::Yacht;
use crate::components::{
    ActiveEntity, Car, ControlState, F16, Helicopter, HumanAnimation, InCar, PendingPhysicsEnable,
    Player, PlayerControlled, VehicleControlType,
};
use crate::game_state::GameState;
use crate::systems::safe_active_entity::queue_active_transfer;
use crate::systems::swimming::{ProneRotation, Swimming};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// Helper: Transfer player control to vehicle (extracted to avoid duplication)
fn transfer_to_vehicle(
    commands: &mut Commands,
    player_entity: Entity,
    vehicle_entity: Entity,
    control_state: Option<&ControlState>,
    player_controlled: Option<&PlayerControlled>,
    vehicle_type: VehicleControlType,
    vehicle_name: &str,
) {
    // Queue atomic ActiveEntity transfer
    queue_active_transfer(commands, player_entity, vehicle_entity);

    // Remove control components from player and hide them
    commands
        .entity(player_entity)
        .remove::<PlayerControlled>()
        .remove::<ControlState>()
        .remove::<VehicleControlType>()
        .insert(Visibility::Hidden)
        .insert(RigidBodyDisabled);

    // Make player a child of the vehicle
    commands
        .entity(player_entity)
        .insert(ChildOf(vehicle_entity));

    // Transfer control components to vehicle
    let mut vehicle_commands = commands.entity(vehicle_entity);
    if let Some(control_state) = control_state {
        vehicle_commands.insert(control_state.clone());
    } else {
        vehicle_commands.insert(ControlState::default());
    }

    if player_controlled.is_some() {
        vehicle_commands.insert(PlayerControlled);
    }

    vehicle_commands.insert(vehicle_type);

    // Store vehicle reference
    commands.entity(player_entity).insert(InCar(vehicle_entity));

    info!("Entered {vehicle_name}! Entity: {:?}", vehicle_entity);
}

pub fn interaction_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            Option<&ControlState>,
            Option<&PlayerControlled>,
            Option<&VehicleControlType>,
            Option<&mut HumanAnimation>,
        ),
        (
            With<Player>,
            Without<Car>,
            Without<Helicopter>,
            Without<F16>,
            Without<Yacht>,
        ),
    >,
    active_control_query: Query<&ControlState, With<ActiveEntity>>,
    // Unified vehicle query: single pass to check all vehicle types with Optional markers
    unified_vehicle_query: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&Car>,
            Option<&Helicopter>,
            Option<&F16>,
            Option<&Yacht>,
        ),
        (
            Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>)>,
            Without<Player>,
        ),
    >,
    // Legacy queries kept for exit logic (needed to fetch specific active vehicle transforms)
    car_query: Query<(Entity, &GlobalTransform, Option<&Velocity>), (With<Car>, Without<Player>)>,
    helicopter_query: Query<
        (Entity, &GlobalTransform, Option<&Velocity>),
        (With<Helicopter>, Without<Player>),
    >,
    f16_query: Query<(Entity, &GlobalTransform, Option<&Velocity>), (With<F16>, Without<Player>)>,
    active_query: Query<Entity, With<ActiveEntity>>,
    just_controlled: Query<Entity, Added<PlayerControlled>>,
) {
    // Check for interact action from ControlState (unified input source)
    // Use active entity's ControlState if available, fallback to keyboard for Walking/Swimming
    let interact_pressed = if let Ok(control_state) = active_control_query.single() {
        control_state.interact
    } else {
        // Fallback for Walking/Swimming states where player is ActiveEntity
        keyboard_input.just_pressed(KeyCode::KeyF)
    };

    if !interact_pressed {
        return;
    }

    match **current_state {
        GameState::Walking => {
            // Try to enter vehicle (car or helicopter)
            let Ok((
                player_entity,
                player_transform,
                _,
                control_state,
                player_controlled,
                _vehicle_control_type,
                _human_animation,
            )) = player_query.single_mut()
            else {
                warn!("Failed to get player entity!");
                return;
            };

            // OPTIMIZED: Priority-based collection with distance_squared (3-4x faster)
            // Collect best match for each vehicle type, then check priority order
            // Using squared distances avoids expensive sqrt: 3.0² = 9.0, 5.0² = 25.0, 8.0² = 64.0, 35.0² = 1225.0
            let mut best_car: Option<(Entity, f32)> = None;
            let mut best_helicopter: Option<(Entity, f32)> = None;
            let mut best_f16: Option<(Entity, f32)> = None;
            let mut best_yacht: Option<(Entity, f32)> = None;
            let mut nearest_yacht_for_debug: Option<f32> = None;

            for (entity, gt, car, helicopter, f16, yacht) in unified_vehicle_query.iter() {
                let dist_sq = player_transform
                    .translation
                    .distance_squared(gt.translation());

                // Track nearest yacht for debug message
                if yacht.is_some() && dist_sq < 10000.0 {
                    nearest_yacht_for_debug =
                        Some(nearest_yacht_for_debug.map_or(dist_sq, |d| d.min(dist_sq)));
                }

                // Collect best match for each vehicle type
                if car.is_some() && dist_sq < 9.0 {
                    if best_car.is_none_or(|(_, d)| dist_sq < d) {
                        best_car = Some((entity, dist_sq));
                    }
                } else if helicopter.is_some() && dist_sq < 25.0 {
                    if best_helicopter.is_none_or(|(_, d)| dist_sq < d) {
                        best_helicopter = Some((entity, dist_sq));
                    }
                } else if f16.is_some() && dist_sq < 64.0 {
                    if best_f16.is_none_or(|(_, d)| dist_sq < d) {
                        best_f16 = Some((entity, dist_sq));
                    }
                } else if yacht.is_some()
                    && dist_sq < 1225.0
                    && best_yacht.is_none_or(|(_, d)| dist_sq < d)
                {
                    best_yacht = Some((entity, dist_sq));
                }
            }

            // Check priority order: car > helicopter > f16 > yacht
            if let Some((entity, _)) = best_car {
                transfer_to_vehicle(
                    &mut commands,
                    player_entity,
                    entity,
                    control_state,
                    player_controlled,
                    VehicleControlType::Car,
                    "Car",
                );
                state.set(GameState::Driving);
                return;
            } else if let Some((entity, _)) = best_helicopter {
                transfer_to_vehicle(
                    &mut commands,
                    player_entity,
                    entity,
                    control_state,
                    player_controlled,
                    VehicleControlType::Helicopter,
                    "Helicopter",
                );
                state.set(GameState::Flying);
                return;
            } else if let Some((entity, _)) = best_f16 {
                transfer_to_vehicle(
                    &mut commands,
                    player_entity,
                    entity,
                    control_state,
                    player_controlled,
                    VehicleControlType::F16,
                    "F16",
                );
                state.set(GameState::Jetting);
                return;
            } else if let Some((yacht_entity, dist_sq)) = best_yacht {
                let distance = dist_sq.sqrt(); // Only sqrt for logging
                // Get yacht position for logging
                if let Ok((_, yacht_gt, _, _, _, _)) = unified_vehicle_query.get(yacht_entity) {
                    info!(
                        "Player at {:?}, Yacht at {:?}, Distance: {:.1}m - Boarding yacht!",
                        player_transform.translation,
                        yacht_gt.translation(),
                        distance
                    );
                }
                transfer_to_vehicle(
                    &mut commands,
                    player_entity,
                    yacht_entity,
                    control_state,
                    player_controlled,
                    VehicleControlType::Yacht,
                    "Yacht",
                );
                state.set(GameState::Driving);
                return;
            }

            // Debug message for yacht outside range
            if let Some(dist_sq) = nearest_yacht_for_debug {
                if dist_sq >= 1225.0 {
                    let distance = dist_sq.sqrt();
                    info!(
                        "Yacht too far! Distance: {:.1}m (need < 35m). Swim closer and press F.",
                        distance
                    );
                }
            }
        }
        GameState::Swimming => {
            // Try to enter yacht from swimming
            let Ok((
                player_entity,
                player_transform,
                _,
                control_state,
                player_controlled,
                _vehicle_control_type,
                human_animation,
            )) = player_query.single_mut()
            else {
                warn!("Failed to get player entity!");
                return;
            };

            // OPTIMIZED: Use unified query with distance_squared for yachts
            for (entity, gt, _car, _helicopter, _f16, yacht) in unified_vehicle_query.iter() {
                if yacht.is_none() {
                    continue;
                }

                let dist_sq = player_transform
                    .translation
                    .distance_squared(gt.translation());

                if dist_sq < 1225.0 {
                    // 35.0²
                    let distance = dist_sq.sqrt(); // Only sqrt for logging
                    info!(
                        "Player at {:?}, Yacht at {:?}, Distance: {:.1}m - Boarding yacht!",
                        player_transform.translation,
                        gt.translation(),
                        distance
                    );

                    // Queue atomic ActiveEntity transfer
                    queue_active_transfer(&mut commands, player_entity, entity);

                    // CRITICAL: Clean up swimming state components when entering from water
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .remove::<VehicleControlType>()
                        .remove::<Swimming>()
                        .remove::<ProneRotation>()
                        .remove::<GravityScale>()
                        .remove::<Damping>()
                        .insert(Visibility::Hidden)
                        .insert(RigidBodyDisabled);

                    // Reset swim animation flag
                    if let Some(mut anim) = human_animation {
                        anim.is_swimming = false;
                    }

                    // Make player a child of the yacht
                    commands.entity(player_entity).insert(ChildOf(entity));

                    // Transfer control components to yacht
                    let mut yacht_commands = commands.entity(entity);
                    if let Some(control_state) = control_state {
                        yacht_commands.insert(control_state.clone());
                    } else {
                        yacht_commands.insert(ControlState::default());
                    }

                    if player_controlled.is_some() {
                        yacht_commands.insert(PlayerControlled);
                    }

                    yacht_commands.insert(VehicleControlType::Yacht);
                    commands.entity(player_entity).insert(InCar(entity));

                    state.set(GameState::Driving);
                    info!("Climbed aboard Superyacht from water!");
                    return;
                } else if dist_sq < 10000.0 {
                    // 100.0²
                    let distance = dist_sq.sqrt();
                    info!(
                        "Yacht too far! Distance: {:.1}m (need < 35m). Swim closer and press F.",
                        distance
                    );
                }
            }
        }
        GameState::Driving => {
            // Exit car (NOT yacht - yacht is handled by yacht_exit_system)
            if let Ok(active_car) = active_query.single() {
                // Skip one frame after control transfer to prevent immediate exit when F is held
                if just_controlled.get(active_car).is_ok() {
                    return;
                }

                // CRITICAL: Only handle car exits here, yachts use yacht_exit_system
                // Get the specific active car's global transform and velocity
                if let Ok((_, car_gt, car_vel)) = car_query.get(active_car) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _, _)) = player_query.single_mut() {
                        // Queue atomic ActiveEntity transfer back to player
                        queue_active_transfer(&mut commands, active_car, player_entity);

                        // Calculate exit position in WORLD SPACE using GlobalTransform
                        // Use horizontal-only right vector to avoid extreme teleportation from vehicle rotation
                        let right_horizontal =
                            Vec3::new(car_gt.right().x, 0.0, car_gt.right().z).normalize_or_zero();
                        let exit_position = car_gt.translation() + right_horizontal * 3.0;
                        let inherited_vel = car_vel.cloned().unwrap_or(Velocity::zero());

                        // Preserve vehicle's Y rotation so player faces same direction
                        let (vehicle_yaw, _, _) = car_gt
                            .to_scale_rotation_translation()
                            .1
                            .to_euler(EulerRot::YXZ);
                        let exit_rotation = Quat::from_rotation_y(vehicle_yaw);

                        // Phase A: Set pose and keep physics disabled this frame
                        commands
                            .entity(player_entity)
                            .remove::<InCar>()
                            .remove::<ChildOf>()
                            .insert(
                                Transform::from_translation(exit_position)
                                    .with_rotation(exit_rotation),
                            )
                            .insert(inherited_vel)
                            .insert(PlayerControlled)
                            .insert(ControlState::default())
                            .insert(VehicleControlType::Walking)
                            .insert(Visibility::Visible)
                            .insert(PendingPhysicsEnable);
                        // DO NOT remove RigidBodyDisabled here; will be done next frame

                        info!(
                            "ActiveEntity transferred from Car({:?}) back to Player({:?})",
                            active_car, player_entity
                        );

                        // ONLY set Walking state if we actually exited a car
                        state.set(GameState::Walking);
                    }
                }
                // If active_car exists but isn't a Car (e.g., it's a Yacht), do nothing here
                // Let yacht_exit_system handle it
            }
        }
        GameState::Flying => {
            // Exit helicopter
            if let Ok(active_helicopter) = active_query.single() {
                // Skip one frame after control transfer to prevent immediate exit when F is held
                if just_controlled.get(active_helicopter).is_ok() {
                    return;
                }

                // Get the specific active helicopter's global transform and velocity
                if let Ok((_, helicopter_gt, helicopter_vel)) =
                    helicopter_query.get(active_helicopter)
                {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _, _)) = player_query.single_mut() {
                        // Queue atomic ActiveEntity transfer back to player
                        queue_active_transfer(&mut commands, active_helicopter, player_entity);

                        // Calculate exit position in WORLD SPACE using GlobalTransform
                        // Use horizontal-only right vector to avoid extreme teleportation from aircraft rotation
                        let right_horizontal =
                            Vec3::new(helicopter_gt.right().x, 0.0, helicopter_gt.right().z)
                                .normalize_or_zero();
                        let exit_position = helicopter_gt.translation()
                            + right_horizontal * 4.0  // Horizontal offset only
                            + Vec3::new(0.0, -1.0, 0.0); // Drop to ground level

                        // Inherit only horizontal velocity for realistic free-fall
                        let mut inherited_vel = helicopter_vel.cloned().unwrap_or(Velocity::zero());
                        inherited_vel.linvel.y = 0.0; // Zero out upward velocity for realistic gravity

                        // Preserve vehicle's Y rotation so player faces same direction
                        let (vehicle_yaw, _, _) = helicopter_gt
                            .to_scale_rotation_translation()
                            .1
                            .to_euler(EulerRot::YXZ);
                        let exit_rotation = Quat::from_rotation_y(vehicle_yaw);

                        // Phase A: Set pose and keep physics disabled this frame
                        commands
                            .entity(player_entity)
                            .remove::<InCar>()
                            .remove::<ChildOf>()
                            .insert(
                                Transform::from_translation(exit_position)
                                    .with_rotation(exit_rotation),
                            )
                            .insert(inherited_vel)
                            .insert(PlayerControlled)
                            .insert(ControlState::default())
                            .insert(VehicleControlType::Walking)
                            .insert(Visibility::Visible)
                            .insert(PendingPhysicsEnable);
                        // DO NOT remove RigidBodyDisabled here; will be done next frame

                        info!("Exited helicopter at position: {:?}", exit_position);
                    }

                    // Switch to walking state
                    state.set(GameState::Walking);
                    info!("Exited helicopter!");
                }
            }
        }
        GameState::Jetting => {
            // Exit F16
            if let Ok(active_f16) = active_query.single() {
                // Skip one frame after control transfer to prevent immediate exit when F is held
                if just_controlled.get(active_f16).is_ok() {
                    return;
                }

                // Get the specific active F16's global transform and velocity
                if let Ok((_, f16_gt, f16_vel)) = f16_query.get(active_f16) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _, _)) = player_query.single_mut() {
                        // Queue atomic ActiveEntity transfer back to player
                        queue_active_transfer(&mut commands, active_f16, player_entity);

                        // Calculate exit position in WORLD SPACE using GlobalTransform
                        // Use horizontal-only right vector to avoid extreme teleportation from aircraft rotation
                        let right_horizontal =
                            Vec3::new(f16_gt.right().x, 0.0, f16_gt.right().z).normalize_or_zero();
                        let exit_position = f16_gt.translation()
                            + right_horizontal * 6.0  // Horizontal offset only
                            + Vec3::new(0.0, -2.0, 0.0); // Drop to ground level

                        // Inherit only horizontal velocity for realistic free-fall
                        let mut inherited_vel = f16_vel.cloned().unwrap_or(Velocity::zero());
                        inherited_vel.linvel.y = 0.0; // Zero out upward velocity for realistic gravity

                        // Preserve vehicle's Y rotation so player faces same direction
                        let (vehicle_yaw, _, _) = f16_gt
                            .to_scale_rotation_translation()
                            .1
                            .to_euler(EulerRot::YXZ);
                        let exit_rotation = Quat::from_rotation_y(vehicle_yaw);

                        // Phase A: Set pose and keep physics disabled this frame
                        commands
                            .entity(player_entity)
                            .remove::<InCar>()
                            .remove::<ChildOf>()
                            .insert(
                                Transform::from_translation(exit_position)
                                    .with_rotation(exit_rotation),
                            )
                            .insert(inherited_vel)
                            .insert(PlayerControlled)
                            .insert(ControlState::default())
                            .insert(VehicleControlType::Walking)
                            .insert(Visibility::Visible)
                            .insert(PendingPhysicsEnable);
                        // DO NOT remove RigidBodyDisabled here; will be done next frame

                        info!("Exited F16 at position: {:?}", exit_position);
                    }

                    // Switch to walking state
                    state.set(GameState::Walking);
                    info!("Exited F16!");
                }
            }
        }
    }
}
