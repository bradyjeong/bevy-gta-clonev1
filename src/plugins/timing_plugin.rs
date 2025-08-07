use bevy::prelude::*;
use crate::services::timing_service::{TimingService, update_timing_service, cleanup_timing_service};
use crate::systems::batching::frame_counter_system;

/// Plugin responsible for timing services and frame management
pub struct TimingPlugin;

impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TimingService>()
            .init_resource::<crate::components::FrameCounter>()
            .add_systems(PreUpdate, frame_counter_system)
            .add_systems(Update, (
                update_timing_service,
                cleanup_timing_service,
            ).chain());
    }
}
