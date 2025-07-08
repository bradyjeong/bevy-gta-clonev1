use bevy::prelude::*;

/// Frame counter resource for batching system
#[derive(Resource, Default)]
pub struct FrameCounter {
    pub frame: u64,
}

/// Minimal frame counter system stub
pub fn frame_counter_system(mut frame_counter: ResMut<FrameCounter>) {
    frame_counter.frame += 1;
}
