use bevy::prelude::*;

/// Debug system for distance cache
pub fn distance_cache_debug_system(
    mut timer: Local<f32>,
    time: Res<Time>,
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
) {
    // Update timer
    *timer += time.delta_secs();
    
    // Only output stats every 5 seconds OR when F3 is pressed
    let should_output = *timer >= 5.0 || keyboard_input.as_ref().is_some_and(|input| input.just_pressed(KeyCode::F3));
    if should_output {
        *timer = 0.0;
        info!("Distance cache debug system not yet implemented");
    }
}
/// Plugin to add distance cache debugging
pub struct DistanceCacheDebugPlugin;

impl Plugin for DistanceCacheDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, distance_cache_debug_system);
    }
}
