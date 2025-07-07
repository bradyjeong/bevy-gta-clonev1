//! ───────────────────────────────────────────────
//! System:   Debug
//! Purpose:  Handles entity movement and physics
//! Schedule: Update
//! Reads:    ActiveEntity, Transform, Car, Helicopter, F16
//! Writes:   System state
//! Invariants:
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use crate::components::{Player, Car, Helicopter, F16, ActiveEntity};
use crate::game_state::GameState;

pub fn debug_player_position(
    _player_query: Query<&Transform, (With<Player>, With<ActiveEntity>)>,
    _car_query: Query<&Transform, (With<Car>, With<ActiveEntity>)>,
    _helicopter_query: Query<&Transform, (With<Helicopter>, With<ActiveEntity>)>,
    _f16_query: Query<&Transform, (With<F16>, With<ActiveEntity>)>,
    _state: Res<State<GameState>>,
) {
    #[cfg(feature = "debug-movement")]
    {
        match **_state {
            GameState::Walking => {
                if let Ok(player_transform) = _player_query.single() {
                    // Only log occasionally to avoid spam
                    if player_transform.translation.x.abs() > 100.0 || player_transform.translation.z.abs() > 100.0 {
                        info!("DEBUG: Player walking at position: {:?}", player_transform.translation);
                    }
                }
            }
            GameState::Driving => {
                if let Ok(car_transform) = _car_query.single() {
                    // Only log occasionally 
                    if car_transform.translation.x.abs() > 100.0 || car_transform.translation.z.abs() > 100.0 {
                        info!("DEBUG: Car driving at position: {:?}", car_transform.translation);
                    }
                }
            }
            GameState::Flying => {
                if let Ok(helicopter_transform) = _helicopter_query.single() {
                    // Log helicopter altitude and position
                    if helicopter_transform.translation.y > 5.0 || helicopter_transform.translation.x.abs() > 100.0 || helicopter_transform.translation.z.abs() > 100.0 {
                        info!("DEBUG: Helicopter flying at position: {:?} (altitude: {:.1}m)", helicopter_transform.translation, helicopter_transform.translation.y);
                    }
                }
            }
            GameState::Jetting => {
                if let Ok(f16_transform) = _f16_query.single() {
                    // Log F16 altitude and position
                    if f16_transform.translation.y > 10.0 || f16_transform.translation.x.abs() > 100.0 || f16_transform.translation.z.abs() > 100.0 {
                        info!("DEBUG: F16 flying at position: {:?} (altitude: {:.1}m)", f16_transform.translation, f16_transform.translation.y);
                    }
                }
            }
        }
    }
}
