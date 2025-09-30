#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::bundles::PlayerPhysicsBundle;
use crate::components::{
    ActiveEntity, Car, ControlState, F16, Helicopter, InCar, Player, PlayerControlled,
    VehicleControlType,
};
use crate::game_state::GameState;
use crate::systems::queue_active_transfer;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Enhanced vehicle exit helper with physics restoration
fn transfer_player_from_vehicle_with_physics(
    commands: &mut Commands,
    player: Entity,
    vehicle: Entity,
    exit_position: Vec3,
    exit_rotation: Quat,
) {
    // Remove control from vehicle
    commands
        .entity(vehicle)
        .remove::<ControlState>()
        .remove::<PlayerControlled>()
        .remove::<VehicleControlType>();

    // Restore player control, physics, and visibility
    commands
        .entity(player)
        .remove::<InCar>()
        .remove::<ChildOf>()
        .insert(PlayerControlled)
        .insert(ControlState::default())
        .insert(VehicleControlType::Walking)
        .insert(Visibility::Visible)
        .insert(Transform::from_translation(exit_position).with_rotation(exit_rotation))
        .insert(PlayerPhysicsBundle::default());
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
        ),
        (
            With<Player>,
            Without<Car>,
            Without<Helicopter>,
            Without<F16>,
        ),
    >,
    car_query: Query<(Entity, &Transform), (With<Car>, Without<Player>)>,
    helicopter_query: Query<(Entity, &Transform), (With<Helicopter>, Without<Player>)>,
    f16_query: Query<(Entity, &Transform), (With<F16>, Without<Player>)>,
    _vehicle_control_query: Query<
        (
            Option<&ControlState>,
            Option<&PlayerControlled>,
            Option<&VehicleControlType>,
        ),
        (
            Or<(With<Car>, With<Helicopter>, With<F16>)>,
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
            for (car_entity, car_transform) in car_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(car_transform.translation);
                if distance < 3.0 {
                    // Disable vehicle physics temporarily to prevent explosions
                    commands.entity(car_entity).insert(RigidBodyDisabled);

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, car_entity);

                    // Remove control components from player and hide them
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .insert(Visibility::Hidden);

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

                    // Re-enable vehicle physics after setup is complete (prevents physics explosions)
                    commands.entity(car_entity).remove::<RigidBodyDisabled>();

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
            for (helicopter_entity, helicopter_transform) in helicopter_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(helicopter_transform.translation);
                if distance < 5.0 {
                    // Larger range for helicopters
                    // Disable helicopter physics temporarily to prevent explosions
                    commands.entity(helicopter_entity).insert(RigidBodyDisabled);

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, helicopter_entity);

                    // Remove control components from player and hide them
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .insert(Visibility::Hidden);

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

                    // Re-enable helicopter physics after setup is complete (prevents physics explosions)
                    commands
                        .entity(helicopter_entity)
                        .remove::<RigidBodyDisabled>();

                    // Switch to flying state
                    state.set(GameState::Flying);
                    info!("Entered helicopter!");
                    return;
                }
            }

            // Check for F16s
            for (f16_entity, f16_transform) in f16_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(f16_transform.translation);
                if distance < 8.0 {
                    // Larger range for F16s
                    // Disable F16 physics temporarily to prevent explosions
                    commands.entity(f16_entity).insert(RigidBodyDisabled);

                    // Queue atomic ActiveEntity transfer (prevents gaps)
                    queue_active_transfer(&mut commands, player_entity, f16_entity);

                    // Remove control components from player and hide them
                    commands
                        .entity(player_entity)
                        .remove::<PlayerControlled>()
                        .remove::<ControlState>()
                        .insert(Visibility::Hidden);

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

                    // Re-enable F16 physics after setup is complete (prevents physics explosions)
                    commands.entity(f16_entity).remove::<RigidBodyDisabled>();

                    // Switch to jetting state
                    state.set(GameState::Jetting);
                    info!("Entered F16 Fighter Jet!");
                    return;
                }
            }
        }
        GameState::Swimming => {
            // Swimming doesn't have vehicle interaction - same as walking
            // Could add underwater vehicle interaction here in the future
        }
        GameState::Driving => {
            // Exit car
            if let Ok(active_car) = active_query.single() {
                // Get the specific active car's transform and control components
                if let Ok((_, car_transform)) = car_query.get(active_car) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the car
                        let exit_position = car_transform.translation + car_transform.right() * 3.0;

                        // Queue atomic ActiveEntity transfer back to player
                        queue_active_transfer(&mut commands, active_car, player_entity);

                        // Use centralized physics restoration helper
                        transfer_player_from_vehicle_with_physics(
                            &mut commands,
                            player_entity,
                            active_car,
                            exit_position,
                            car_transform.rotation,
                        );

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
                // Get the specific active helicopter's transform
                if let Ok((_, helicopter_transform)) = helicopter_query.get(active_helicopter) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
                        // Queue atomic ActiveEntity transfer back to player
                        queue_active_transfer(&mut commands, active_helicopter, player_entity);

                        // Calculate exit position next to the helicopter (a bit further away)
                        let exit_position = helicopter_transform.translation
                            + helicopter_transform.right() * 4.0
                            + Vec3::new(0.0, -1.0, 0.0); // Drop to ground level

                        // Use centralized physics restoration helper
                        transfer_player_from_vehicle_with_physics(
                            &mut commands,
                            player_entity,
                            active_helicopter,
                            exit_position,
                            Quat::IDENTITY,
                        );

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
                // Get the specific active F16's transform
                if let Ok((_, f16_transform)) = f16_query.get(active_f16) {
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
                        // Queue atomic ActiveEntity transfer back to player
                        queue_active_transfer(&mut commands, active_f16, player_entity);

                        // Calculate exit position next to the F16 (further away)
                        let exit_position = f16_transform.translation
                            + f16_transform.right() * 6.0
                            + Vec3::new(0.0, -2.0, 0.0); // Drop to ground level

                        // Use centralized physics restoration helper
                        transfer_player_from_vehicle_with_physics(
                            &mut commands,
                            player_entity,
                            active_f16,
                            exit_position,
                            Quat::IDENTITY,
                        );

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
