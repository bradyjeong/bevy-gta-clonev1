use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Player, Car, Helicopter, F16, ActiveEntity, InCar};
use crate::game_state::GameState;

pub fn interaction_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut player_query: Query<(Entity, &mut Transform, &mut Velocity), (With<Player>, Without<Car>, Without<Helicopter>, Without<F16>)>,
    car_query: Query<(Entity, &Transform), (With<Car>, Without<Player>)>,
    helicopter_query: Query<(Entity, &Transform), (With<Helicopter>, Without<Player>)>,
    f16_query: Query<(Entity, &Transform), (With<F16>, Without<Player>)>,
    active_query: Query<Entity, With<ActiveEntity>>,
) {
    if !input.just_pressed(KeyCode::KeyF) {
        return;
    }
    


    match **current_state {
        GameState::Walking => {
            // Try to enter vehicle (car or helicopter)
            let Ok((player_entity, player_transform, _)) = player_query.single_mut() else { 
                warn!("Failed to get player entity!");
                return; 
            };
            
            // Check for cars first
            for (car_entity, car_transform) in car_query.iter() {
                let distance = player_transform.translation.distance(car_transform.translation);
                if distance < 3.0 {
                    // Remove ActiveEntity from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the car
                    commands.entity(player_entity).insert(ChildOf(car_entity));
                    
                    // Add ActiveEntity to car
                    commands.entity(car_entity).insert(ActiveEntity);
                    
                    // Store which car the player is in
                    commands.entity(player_entity).insert(InCar(car_entity));
                    
                    // Switch to driving state
                    state.set(GameState::Driving);
                    info!("Entered car!");
                    return;
                }
            }
            
            // Check for helicopters
            for (helicopter_entity, helicopter_transform) in helicopter_query.iter() {
                let distance = player_transform.translation.distance(helicopter_transform.translation);
                if distance < 5.0 { // Larger range for helicopters
                    // Remove ActiveEntity from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the helicopter
                    commands.entity(player_entity).insert(ChildOf(helicopter_entity));
                    
                    // Add ActiveEntity to helicopter
                    commands.entity(helicopter_entity).insert(ActiveEntity);
                    
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
                    // Remove ActiveEntity from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the F16
                    commands.entity(player_entity).insert(ChildOf(f16_entity));
                    
                    // Add ActiveEntity to F16
                    commands.entity(f16_entity).insert(ActiveEntity);
                    
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
                // Get the specific active car's transform
                if let Ok((_, car_transform)) = car_query.get(active_car) {
                    // Remove ActiveEntity from car
                    commands.entity(active_car).remove::<ActiveEntity>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the car
                        let exit_position = car_transform.translation + car_transform.right() * 3.0;
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(player_entity)
                            .remove::<ChildOf>()
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(car_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
                        info!("Exited car at position: {:?}", exit_position);
                    }
                    
                    // Switch to walking state
                    state.set(GameState::Walking);
                    info!("Exited car!");
                }
            }
        }
        GameState::Flying => {
            // Exit helicopter
            if let Ok(active_helicopter) = active_query.single() {
                // Get the specific active helicopter's transform
                if let Ok((_, helicopter_transform)) = helicopter_query.get(active_helicopter) {
                    // Remove ActiveEntity from helicopter
                    commands.entity(active_helicopter).remove::<ActiveEntity>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the helicopter (a bit further away)
                        let exit_position = helicopter_transform.translation + helicopter_transform.right() * 4.0 + Vec3::new(0.0, -1.0, 0.0); // Drop to ground level
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(player_entity)
                            .remove::<ChildOf>()
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(helicopter_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
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
                    // Remove ActiveEntity from F16
                    commands.entity(active_f16).remove::<ActiveEntity>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the F16 (further away)
                        let exit_position = f16_transform.translation + f16_transform.right() * 6.0 + Vec3::new(0.0, -2.0, 0.0); // Drop to ground level
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(player_entity)
                            .remove::<ChildOf>()
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(f16_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
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
