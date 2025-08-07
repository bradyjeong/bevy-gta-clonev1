use bevy::prelude::*;
use crate::components::dirty_flags::FrameCounter;

/// Simple frame counter system
pub fn frame_counter_system(mut frame_counter: ResMut<FrameCounter>) {
    frame_counter.frame = frame_counter.frame.wrapping_add(1);
}
