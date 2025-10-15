#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::water::Yacht;
use crate::components::{
    ActiveEntity, Car, ControlState, F16, Helicopter, InCar, PendingPhysicsEnable, Player,
    PlayerControlled, VehicleControlType,
};
use crate::game_state::GameState;
use crate::systems::safe_active_entity::queue_active_transfer;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
        ),
        (
            With<Player>,
            Without<Car>,
            Without<Helicopter>,
            Without<F16>,
            Without<Yacht>,
        ),
    >,
    car_query: Query<(Entity, &GlobalTransform, Option<&Velocity>), (With<Car>, Without<Player>)>,
    helicopter_query: Query<
        (Entity, &GlobalTransform, Option<&Velocity>),
        (With<Helicopter>, Without<Player>),
    >,
    f16_query: Query<(Entity, &GlobalTransform, Option<&Velocity>), (With<F16>, Without<Player>)>,
    yacht_query: Query<
        (Entity, &GlobalTransform, Option<&Velocity>),
        (With<Yacht>, Without<Player>),
    >,
    _vehicle_control_query: Query<
        (
            Option<&ControlState>,
            Option<&PlayerControlled>,
            Option<&VehicleControlType>,
        ),
        (
            Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>)>,
            Without<Player>,
        ),
    >,
    active_query: Query<Entity, With<ActiveEntity>>,
) {
    // Check for F key press directly from keyboard input
    let interact_pressed = keyboard_input.just_pressed(KeyCode::KeyF);

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
            )) = player_query.single_mut()
            else {
                warn!("Failed to get player entity!");
                return;
            };

            // Check for cars first
            for (car_entity, car_gt, _) in car_query.iter() {
                let distance = player_transform.translation.distance(car_gt.translation());
                if distance < 3.0 {
                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, car_entity);

                    // Remove control components from player and hide them
                    // CRITICAL: Disable player physics to prevent corruption while in vehicle
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .remove::<VehicleControlType>()
                        .insert(Visibility::Hidden)
                        .insert(RigidBodyDisabled);

                    // Make player a child of the car
                    commands.entity(player_entity).insert(ChildOf(car_entity));

                    // Add control components to car (ActiveEntity handled by transfer system)
                    let mut car_commands = commands.entity(car_entity);

                    // Transfer control components to car with appropriate vehicle type
                    if let Some(control_state) = control_state {
                        car_commands.insert(control_state.clone());
                    } else {
                        car_commands.insert(ControlState::default());
                    }

                    if player_controlled.is_some() {
                        car_commands.insert(PlayerControlled);
                    }

                    // Set appropriate vehicle control type for cars
                    car_commands.insert(VehicleControlType::Car);

                    // Store which car the player is in
                    commands.entity(player_entity).insert(InCar(car_entity));

                    // Switch to driving state
                    state.set(GameState::Driving);
                    info!(
                        "ActiveEntity transferred from Player({:?}) to Car({:?})",
                        player_entity, car_entity
                    );
                    return;
                }
            }

            // Check for helicopters
            for (helicopter_entity, helicopter_gt, _) in helicopter_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(helicopter_gt.translation());
                if distance < 5.0 {
                    // Larger range for helicopters

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, helicopter_entity);

                    // Remove control components from player and hide them
                    // CRITICAL: Disable player physics to prevent corruption while in vehicle
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .remove::<VehicleControlType>()
                        .insert(Visibility::Hidden)
                        .insert(RigidBodyDisabled);

                    // Make player a child of the helicopter
                    commands
                        .entity(player_entity)
                        .insert(ChildOf(helicopter_entity));

                    // Add control components to helicopter (ActiveEntity handled by transfer system)
                    let mut helicopter_commands = commands.entity(helicopter_entity);

                    // Transfer control components to helicopter with appropriate vehicle type
                    if let Some(control_state) = control_state {
                        helicopter_commands.insert(control_state.clone());
                    } else {
                        helicopter_commands.insert(ControlState::default());
                    }

                    if player_controlled.is_some() {
                        helicopter_commands.insert(PlayerControlled);
                    }

                    // Set appropriate vehicle control type for helicopters
                    helicopter_commands.insert(VehicleControlType::Helicopter);

                    // Store which helicopter the player is in
                    commands
                        .entity(player_entity)
                        .insert(InCar(helicopter_entity)); // Reuse InCar for vehicles

                    // Switch to flying state
                    state.set(GameState::Flying);
                    info!("Entered helicopter!");
                    return;
                }
            }

            // Check for F16s
            for (f16_entity, f16_gt, _) in f16_query.iter() {
                let distance = player_transform.translation.distance(f16_gt.translation());
                if distance < 8.0 {
                    // Larger range for F16s

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, f16_entity);

                    // Remove control components from player and hide them
                    // CRITICAL: Disable player physics to prevent corruption while in vehicle
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .remove::<VehicleControlType>()
                        .insert(Visibility::Hidden)
                        .insert(RigidBodyDisabled);

                    // Make player a child of the F16
                    commands.entity(player_entity).insert(ChildOf(f16_entity));

                    // Add control components to F16 (ActiveEntity handled by transfer system)
                    let mut f16_commands = commands.entity(f16_entity);

                    // Transfer control components to F16 with appropriate vehicle type
                    if let Some(control_state) = control_state {
                        f16_commands.insert(control_state.clone());
                    } else {
                        f16_commands.insert(ControlState::default());
                    }

                    if player_controlled.is_some() {
                        f16_commands.insert(PlayerControlled);
                    }

                    // Set appropriate vehicle control type for F16s
                    f16_commands.insert(VehicleControlType::F16);

                    // Store which F16 the player is in
                    commands.entity(player_entity).insert(InCar(f16_entity)); // Reuse InCar for vehicles

                    // Switch to jetting state
                    state.set(GameState::Jetting);
                    info!("Entered F16 Fighter Jet!");
                    return;
                }
            }

            // Check for yachts
            for (yacht_entity, yacht_gt, _) in yacht_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(yacht_gt.translation());
                if distance < 35.0 {
                    info!(
                        "Player at {:?}, Yacht at {:?}, Distance: {:.1}m - Boarding yacht!",
                        player_transform.translation,
                        yacht_gt.translation(),
                        distance
                    );

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, yacht_entity);

                    // Remove control components from player and hide them
                    // CRITICAL: Disable player physics to prevent corruption while in vehicle
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .remove::<VehicleControlType>()
                        .insert(Visibility::Hidden)
                        .insert(RigidBodyDisabled);

                    // Make player a child of the yacht
                    commands.entity(player_entity).insert(ChildOf(yacht_entity));

                    // Add control components to yacht (ActiveEntity handled by transfer system)
                    let mut yacht_commands = commands.entity(yacht_entity);

                    // Transfer control components to yacht with appropriate vehicle type
                    if let Some(control_state) = control_state {
                        yacht_commands.insert(control_state.clone());
                    } else {
                        yacht_commands.insert(ControlState::default());
                    }

                    if player_controlled.is_some() {
                        yacht_commands.insert(PlayerControlled);
                    }

                    // Set appropriate vehicle control type for yachts
                    yacht_commands.insert(VehicleControlType::Yacht);

                    // Store which yacht the player is in
                    commands.entity(player_entity).insert(InCar(yacht_entity)); // Reuse InCar for vehicles

                    // Switch to driving state (reuse for yachts)
                    state.set(GameState::Driving);
                    info!("Boarded Superyacht!");
                    return;
                } else if distance < 100.0 {
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
            )) = player_query.single_mut()
            else {
                warn!("Failed to get player entity!");
                return;
            };

            // Check for yachts
            for (yacht_entity, yacht_gt, _) in yacht_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(yacht_gt.translation());

                if distance < 35.0 {
                    info!(
                        "Player at {:?}, Yacht at {:?}, Distance: {:.1}m - Boarding yacht!",
                        player_transform.translation,
                        yacht_gt.translation(),
                        distance
                    );

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, yacht_entity);

                    // Remove control components from player and hide them
                    // CRITICAL: Disable player physics to prevent corruption while in vehicle
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .remove::<VehicleControlType>()
                        .insert(Visibility::Hidden)
                        .insert(RigidBodyDisabled);

                    // Make player a child of the yacht
                    commands.entity(player_entity).insert(ChildOf(yacht_entity));

                    // Add control components to yacht (ActiveEntity handled by transfer system)
                    let mut yacht_commands = commands.entity(yacht_entity);

                    // Transfer control components to yacht with appropriate vehicle type
                    if let Some(control_state) = control_state {
                        yacht_commands.insert(control_state.clone());
                    } else {
                        yacht_commands.insert(ControlState::default());
                    }

                    if player_controlled.is_some() {
                        yacht_commands.insert(PlayerControlled);
                    }

                    // Set appropriate vehicle control type for yachts
                    yacht_commands.insert(VehicleControlType::Yacht);

                    // Store which yacht the player is in
                    commands.entity(player_entity).insert(InCar(yacht_entity));

                    // Switch to driving state (reuse for yachts)
                    state.set(GameState::Driving);
                    info!("Climbed aboard Superyacht from water!");
                    return;
                } else if distance < 100.0 {
                    info!(
                        "Yacht too far! Distance: {:.1}m (need < 35m). Swim closer and press F.",
                        distance
                    );
                }
            }
        }
        GameState::Driving => {
            // Exit car
            if let Ok(active_car) = active_query.single() {
                // Get the specific active car's global transform and velocity
                if let Ok((_, car_gt, car_vel)) = car_query.get(active_car) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
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
                    }

                    // Switch to walking state
                    state.set(GameState::Walking);
                }
            }
        }
        GameState::Flying => {
            // Exit helicopter
            if let Ok(active_helicopter) = active_query.single() {
                // Get the specific active helicopter's global transform and velocity
                if let Ok((_, helicopter_gt, helicopter_vel)) =
                    helicopter_query.get(active_helicopter)
                {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
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
                // Get the specific active F16's global transform and velocity
                if let Ok((_, f16_gt, f16_vel)) = f16_query.get(active_f16) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
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
