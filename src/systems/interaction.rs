use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Player, Car, Helicopter, F16, ActiveEntity, InCar, ControlState, PlayerControlled, VehicleControlType};
use crate::game_state::GameState;

pub fn interaction_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut player_query: Query<(Entity, &mut Transform, &mut Velocity, Option<&ControlState>, Option<&PlayerControlled>, Option<&VehicleControlType>), (With<Player>, Without<Car>, Without<Helicopter>, Without<F16>)>,
    car_query: Query<(Entity, &Transform), (With<Car>, Without<Player>)>,
    helicopter_query: Query<(Entity, &Transform), (With<Helicopter>, Without<Player>)>,
    f16_query: Query<(Entity, &Transform), (With<F16>, Without<Player>)>,
    vehicle_control_query: Query<(Option<&ControlState>, Option<&PlayerControlled>, Option<&VehicleControlType>), (Or<(With<Car>, With<Helicopter>, With<F16>)>, Without<Player>)>,
    active_query: Query<Entity, With<ActiveEntity>>,
) {
    if !input.just_pressed(KeyCode::KeyF) {
        return;
    }
    


    match **current_state {
        GameState::Walking => {
            // Try to enter vehicle (car or helicopter)
            let Ok((player_entity, player_transform, _, control_state, player_controlled, _vehicle_control_type)) = player_query.single_mut() else { 
                warn!("Failed to get player entity!");
                return; 
            };
            
            // Check for cars first
            for (car_entity, car_transform) in car_query.iter() {
                let distance = player_transform.translation.distance(car_transform.translation);
                if distance < 3.0 {
                    // Remove ActiveEntity and control components from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .remove::<PlayerControlled>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the car
                    commands.entity(car_entity).add_child(player_entity);
                    
                    // Add ActiveEntity and control components to car
                    let mut car_commands = commands.entity(car_entity);
                    car_commands.insert(ActiveEntity);
                    
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
                    info!("🚗 ActiveEntity transferred from Player({:?}) to Car({:?})", player_entity, car_entity);
                    return;
                }
            }
            
            // Check for helicopters
            for (helicopter_entity, helicopter_transform) in helicopter_query.iter() {
                let distance = player_transform.translation.distance(helicopter_transform.translation);
                if distance < 5.0 { // Larger range for helicopters
                    // Remove ActiveEntity and control components from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .remove::<PlayerControlled>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the helicopter
                    commands.entity(helicopter_entity).add_child(player_entity);
                    
                    // Add ActiveEntity and control components to helicopter
                    let mut helicopter_commands = commands.entity(helicopter_entity);
                    helicopter_commands.insert(ActiveEntity);
                    
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
                    commands.entity(player_entity).insert(InCar(helicopter_entity)); // Reuse InCar for vehicles
                    
                    // Switch to flying state
                    state.set(GameState::Flying);
                    info!("Entered helicopter!");
                    return;
                }
            }
            
            // Check for F16s
            for (f16_entity, f16_transform) in f16_query.iter() {
                let distance = player_transform.translation.distance(f16_transform.translation);
                if distance < 8.0 { // Larger range for F16s
                    // Remove ActiveEntity and control components from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .remove::<PlayerControlled>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the F16
                    commands.entity(f16_entity).add_child(player_entity);
                    
                    // Add ActiveEntity and control components to F16
                    let mut f16_commands = commands.entity(f16_entity);
                    f16_commands.insert(ActiveEntity);
                    
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
        }
        GameState::Driving => {
            // Exit car
            if let Ok(active_car) = active_query.single() {
                // Get the specific active car's transform and control components
                if let Ok((_, car_transform)) = car_query.get(active_car) {
                    // Get control components from the car
                    let vehicle_control_components = vehicle_control_query.get(active_car).ok();
                    
                    // Remove ActiveEntity and control components from car
                    commands.entity(active_car)
                        .remove::<ActiveEntity>()
                        .remove::<ControlState>()
                        .remove::<PlayerControlled>()
                        .remove::<VehicleControlType>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the car
                        let exit_position = car_transform.translation + car_transform.right() * 3.0;
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(active_car).remove_children(&[player_entity]);
                        let mut player_commands = commands.entity(player_entity);
                        player_commands
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(car_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
                        // Transfer control components back to player
                        if let Some((control_state, player_controlled, _)) = vehicle_control_components {
                            if let Some(control_state) = control_state {
                                player_commands.insert(control_state.clone());
                            } else {
                                player_commands.insert(ControlState::default());
                            }
                            
                            if player_controlled.is_some() {
                                player_commands.insert(PlayerControlled);
                            }
                            
                            // Set back to walking control type
                            player_commands.insert(VehicleControlType::Walking);
                        }
                        
                        info!("🚗 ActiveEntity transferred from Car({:?}) back to Player({:?})", active_car, player_entity);
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
                    // Get control components from the helicopter
                    let vehicle_control_components = vehicle_control_query.get(active_helicopter).ok();
                    
                    // Remove ActiveEntity and control components from helicopter
                    commands.entity(active_helicopter)
                        .remove::<ActiveEntity>()
                        .remove::<ControlState>()
                        .remove::<PlayerControlled>()
                        .remove::<VehicleControlType>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the helicopter (a bit further away)
                        let exit_position = helicopter_transform.translation + helicopter_transform.right() * 4.0 + Vec3::new(0.0, -1.0, 0.0); // Drop to ground level
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(active_helicopter).remove_children(&[player_entity]);
                        let mut player_commands = commands.entity(player_entity);
                        player_commands
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(helicopter_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
                        // Transfer control components back to player
                        if let Some((control_state, player_controlled, _)) = vehicle_control_components {
                            if let Some(control_state) = control_state {
                                player_commands.insert(control_state.clone());
                            } else {
                                player_commands.insert(ControlState::default());
                            }
                            
                            if player_controlled.is_some() {
                                player_commands.insert(PlayerControlled);
                            }
                            
                            // Set back to walking control type
                            player_commands.insert(VehicleControlType::Walking);
                        }
                        
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
                    // Get control components from the F16
                    let vehicle_control_components = vehicle_control_query.get(active_f16).ok();
                    
                    // Remove ActiveEntity and control components from F16
                    commands.entity(active_f16)
                        .remove::<ActiveEntity>()
                        .remove::<ControlState>()
                        .remove::<PlayerControlled>()
                        .remove::<VehicleControlType>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _, _, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the F16 (further away)
                        let exit_position = f16_transform.translation + f16_transform.right() * 6.0 + Vec3::new(0.0, -2.0, 0.0); // Drop to ground level
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(active_f16).remove_children(&[player_entity]);
                        let mut player_commands = commands.entity(player_entity);
                        player_commands
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(f16_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
                        // Transfer control components back to player
                        if let Some((control_state, player_controlled, _)) = vehicle_control_components {
                            if let Some(control_state) = control_state {
                                player_commands.insert(control_state.clone());
                            } else {
                                player_commands.insert(ControlState::default());
                            }
                            
                            if player_controlled.is_some() {
                                player_commands.insert(PlayerControlled);
                            }
                            
                            // Set back to walking control type
                            player_commands.insert(VehicleControlType::Walking);
                        }
                        
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
