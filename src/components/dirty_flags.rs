use bevy::prelude::*;

#[derive(Component, Default)]
pub struct DirtyLOD {
    pub priority: DirtyPriority,
    pub frame_marked: u64,
}

#[derive(Component, Default)]
pub struct DirtyVisibility {
    pub priority: DirtyPriority,
    pub frame_marked: u64,
}

#[derive(Component, Default)]
pub struct DirtyVegetationInstancing {
    pub priority: DirtyPriority,
    pub frame_marked: u64,
}

#[derive(Default, Clone, Copy)]
pub enum DirtyPriority {
    High,
    #[default]
    Normal,
    Low,
}

#[derive(Resource, Default)]
pub struct FrameCounter {
    pub frame: u64,
}

#[derive(Resource, Default)]
pub struct DirtyFlagsMetrics {
    pub entities_processed_lod: u32,
    pub entities_marked_lod: u32,
    pub processing_time_lod: f32,
    pub entities_processed_visibility: u32,
    pub entities_marked_visibility: u32,
    pub processing_time_visibility: f32,
    pub entities_processed_physics: u32,
    pub entities_marked_physics: u32,
    pub processing_time_physics: f32,
    pub entities_processed_transform: u32,
    pub entities_marked_transform: u32,
    pub processing_time_transform: f32,
    pub lod_count: u32,
    pub visibility_count: u32,
    pub instancing_count: u32,
}

impl DirtyLOD {
    pub fn new(priority: DirtyPriority, frame: u64) -> Self {
        Self {
            priority,
            frame_marked: frame,
        }
    }
}

impl DirtyVisibility {
    pub fn new(priority: DirtyPriority, frame: u64) -> Self {
        Self {
            priority,
            frame_marked: frame,
        }
    }
}
