use bevy::prelude::*;

/// Dirty flag components for optimized batch processing
/// These components mark entities that need specific system updates

/// Marks entities whose position/rotation changed
#[derive(Component, Debug, Clone, Default)]
pub struct DirtyTransform {
    pub marked_frame: u64,
    pub priority: DirtyPriority,
}

/// Marks entities whose visibility state changed
#[derive(Component, Debug, Clone, Default)]
pub struct DirtyVisibility {
    pub marked_frame: u64,
    pub priority: DirtyPriority,
}

/// Marks entities with physics state changes
#[derive(Component, Debug, Clone, Default)]
pub struct DirtyPhysics {
    pub marked_frame: u64,
    pub priority: DirtyPriority,
}

/// Marks entities needing LOD recalculation
#[derive(Component, Debug, Clone, Default)]
pub struct DirtyLOD {
    pub marked_frame: u64,
    pub priority: DirtyPriority,
    pub last_distance: f32,
}

/// Marks entities needing vegetation instancing updates
#[derive(Component, Debug, Clone, Default)]
pub struct DirtyVegetationInstancing {
    pub marked_frame: u64,
    pub priority: DirtyPriority,
}

/// Priority levels for dirty flag processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum DirtyPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}


/// Bundle for entities that support all dirty flags
#[derive(Bundle, Default)]
pub struct DirtyFlagsBundle {
    pub transform: DirtyTransform,
    pub visibility: DirtyVisibility,
    pub physics: DirtyPhysics,
    pub lod: DirtyLOD,
}

/// Component to track the last time an entity was processed
#[derive(Component, Debug, Clone)]
#[derive(Default)]
pub struct LastProcessed {
    pub transform_frame: u64,
    pub visibility_frame: u64,
    pub physics_frame: u64,
    pub lod_frame: u64,
}


/// Resource for tracking frame counter
#[derive(Resource, Default)]
pub struct FrameCounter {
    pub frame: u64,
}

/// Batch processing configuration
#[derive(Resource, Debug, Clone)]
pub struct BatchConfig {
    pub transform_batch_size: usize,
    pub visibility_batch_size: usize,
    pub physics_batch_size: usize,
    pub lod_batch_size: usize,
    pub max_processing_time_ms: f32,
    pub priority_boost_frames: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            transform_batch_size: 75,
            visibility_batch_size: 100,
            physics_batch_size: 50,
            lod_batch_size: 80,
            max_processing_time_ms: 8.0, // Max 8ms per system per frame
            priority_boost_frames: 10, // Boost priority if not processed for 10 frames
        }
    }
}

/// Batch processing state tracker
#[derive(Resource, Default)]
pub struct BatchState {
    pub transform_offset: usize,
    pub visibility_offset: usize,
    pub physics_offset: usize,
    pub lod_offset: usize,
}

/// Performance metrics for dirty flag systems
#[derive(Resource, Default)]
pub struct DirtyFlagsMetrics {
    pub entities_marked_transform: usize,
    pub entities_marked_visibility: usize,
    pub entities_marked_physics: usize,
    pub entities_marked_lod: usize,
    pub entities_processed_transform: usize,
    pub entities_processed_visibility: usize,
    pub entities_processed_physics: usize,
    pub entities_processed_lod: usize,
    pub processing_time_transform: f32,
    pub processing_time_visibility: f32,
    pub processing_time_physics: f32,
    pub processing_time_lod: f32,
    pub last_report_time: f32,
}

impl DirtyTransform {
    #[must_use] pub fn new(priority: DirtyPriority, frame: u64) -> Self {
        Self {
            marked_frame: frame,
            priority,
        }
    }
    
    #[must_use] pub fn is_stale(&self, current_frame: u64, stale_threshold: u64) -> bool {
        current_frame.saturating_sub(self.marked_frame) > stale_threshold
    }
}

impl DirtyVisibility {
    #[must_use] pub fn new(priority: DirtyPriority, frame: u64) -> Self {
        Self {
            marked_frame: frame,
            priority,
        }
    }
    
    #[must_use] pub fn is_stale(&self, current_frame: u64, stale_threshold: u64) -> bool {
        current_frame.saturating_sub(self.marked_frame) > stale_threshold
    }
}

impl DirtyPhysics {
    #[must_use] pub fn new(priority: DirtyPriority, frame: u64) -> Self {
        Self {
            marked_frame: frame,
            priority,
        }
    }
    
    #[must_use] pub fn is_stale(&self, current_frame: u64, stale_threshold: u64) -> bool {
        current_frame.saturating_sub(self.marked_frame) > stale_threshold
    }
}

impl DirtyLOD {
    #[must_use] pub fn new(priority: DirtyPriority, frame: u64, distance: f32) -> Self {
        Self {
            marked_frame: frame,
            priority,
            last_distance: distance,
        }
    }
    
    #[must_use] pub fn is_stale(&self, current_frame: u64, stale_threshold: u64) -> bool {
        current_frame.saturating_sub(self.marked_frame) > stale_threshold
    }
    
    #[must_use] pub fn distance_changed_significantly(&self, new_distance: f32, threshold: f32) -> bool {
        (self.last_distance - new_distance).abs() > threshold
    }
}

impl DirtyVegetationInstancing {
    #[must_use] pub fn new(priority: DirtyPriority, frame: u64) -> Self {
        Self {
            marked_frame: frame,
            priority,
        }
    }
    
    #[must_use] pub fn is_stale(&self, current_frame: u64, stale_threshold: u64) -> bool {
        current_frame.saturating_sub(self.marked_frame) > stale_threshold
    }
}

/// Helper trait for marking entities as dirty
pub trait MarkDirty {
    fn mark_transform_dirty(&mut self, priority: DirtyPriority, frame: u64);
    fn mark_visibility_dirty(&mut self, priority: DirtyPriority, frame: u64);
    fn mark_physics_dirty(&mut self, priority: DirtyPriority, frame: u64);
    fn mark_lod_dirty(&mut self, priority: DirtyPriority, frame: u64, distance: f32);
}

/// Implement `MarkDirty` for `EntityCommands` for easy entity marking
impl MarkDirty for EntityCommands<'_> {
    fn mark_transform_dirty(&mut self, priority: DirtyPriority, frame: u64) {
        self.insert(DirtyTransform::new(priority, frame));
    }
    
    fn mark_visibility_dirty(&mut self, priority: DirtyPriority, frame: u64) {
        self.insert(DirtyVisibility::new(priority, frame));
    }
    
    fn mark_physics_dirty(&mut self, priority: DirtyPriority, frame: u64) {
        self.insert(DirtyPhysics::new(priority, frame));
    }
    
    fn mark_lod_dirty(&mut self, priority: DirtyPriority, frame: u64, distance: f32) {
        self.insert(DirtyLOD::new(priority, frame, distance));
    }
}
