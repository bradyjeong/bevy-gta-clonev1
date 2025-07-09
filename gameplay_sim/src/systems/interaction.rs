use bevy::prelude::*;
use game_core::prelude::*;
// Removed bevy16_compat - using direct Bevy methods

/// System to handle player interaction with vehicles
pub fn player_interaction_system(
    mut commands: Commands,
    input_manager: Res<super::input::input_manager::InputManager>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    car_query: Query<(Entity, &Transform), With<Car>>,
    helicopter_query: Query<(Entity, &Transform), With<Helicopter>>,
    f16_query: Query<(Entity, &Transform), With<F16>>,
    active_entity: Res<ActiveEntityResource>,
) {
    if !input_manager.is_action_just_pressed(super::input::input_config::InputAction::Interact) {
        return;
    }

    match **current_state {
        GameState::Walking => {
            if let Ok((_player_entity, player_transform)) = player_query.single() {
                // Check for nearby vehicles to enter
                
                // Check cars
                for (car_entity, car_transform) in car_query.iter() {
                    let distance = player_transform.translation.distance(car_transform.translation);
                    if distance < 3.0 {
                        commands.insert_resource(ActiveEntityResource(Some(car_entity)));
                        next_state.set(GameState::Driving);
                        return;
                    }
                }

                // Check helicopters
                for (helicopter_entity, helicopter_transform) in helicopter_query.iter() {
                    let distance = player_transform.translation.distance(helicopter_transform.translation);
                    if distance < 5.0 { // Larger range for helicopters
                        commands.insert_resource(ActiveEntityResource(Some(helicopter_entity)));
                        next_state.set(GameState::Flying);
                        return;
                    }
                }

                // Check F16s
                for (f16_entity, f16_transform) in f16_query.iter() {
                    let distance = player_transform.translation.distance(f16_transform.translation);
                    if distance < 4.0 {
                        commands.insert_resource(ActiveEntityResource(Some(f16_entity)));
                        next_state.set(GameState::Jetting);
                        return;
                    }
                }
            }
        }
        GameState::Driving => {
            // Exit vehicle and return to walking
            if let Some(active_car) = active_entity.0 {
                if let Ok((_, car_transform)) = car_query.get(active_car) {
                    // Spawn player near the vehicle
                    let exit_position = car_transform.translation + Vec3::new(2.0, 0.0, 0.0);
                    
                    // Find existing player or create new one
                    if let Ok((player_entity, _player_transform)) = player_query.single() {
                        commands.entity(player_entity).insert(Transform::from_translation(exit_position));
                    }
                    
                    commands.insert_resource(ActiveEntityResource(None));
                    next_state.set(GameState::Walking);
                }
            }
        }
        GameState::Flying => {
            // Exit helicopter
            if let Some(active_helicopter) = active_entity.0 {
                if let Ok((_, helicopter_transform)) = helicopter_query.get(active_helicopter) {
                    let exit_position = helicopter_transform.translation + Vec3::new(0.0, -2.0, 2.0);
                    
                    if let Ok((player_entity, _player_transform)) = player_query.single() {
                        commands.entity(player_entity).insert(Transform::from_translation(exit_position));
                    }
                    
                    commands.insert_resource(ActiveEntityResource(None));
                    next_state.set(GameState::Walking);
                }
            }
        }
        GameState::Jetting => {
            // Exit F16
            if let Some(active_f16) = active_entity.0 {
                if let Ok((_, f16_transform)) = f16_query.get(active_f16) {
                    let exit_position = f16_transform.translation + Vec3::new(0.0, -1.0, 3.0);
                    
                    if let Ok((player_entity, _player_transform)) = player_query.single() {
                        commands.entity(player_entity).insert(Transform::from_translation(exit_position));
                    }
                    
                    commands.insert_resource(ActiveEntityResource(None));
                    next_state.set(GameState::Walking);
                }
            }
        }
    }
}

// Alias for backward compatibility
pub use player_interaction_system as interaction_system;
