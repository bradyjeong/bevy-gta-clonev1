use bevy::prelude::*;

/// Level of Detail component for performance optimization
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum LodLevel {
    /// Full detail - all systems active
    #[default]
    High,
    /// Reduced detail - simplified physics and effects
    Medium,
    /// Minimal detail - visual only, physics disabled
    Sleep,
}


impl LodLevel {
    /// Returns true if this LOD level should have physics enabled
    #[must_use] pub fn has_physics(&self) -> bool {
        matches!(self, LodLevel::High | LodLevel::Medium)
    }
    
    /// Returns true if this LOD level should have visual effects
    #[must_use] pub fn has_effects(&self) -> bool {
        matches!(self, LodLevel::High)
    }
    
    /// Returns true if this LOD level should have audio
    #[must_use] pub fn has_audio(&self) -> bool {
        matches!(self, LodLevel::High)
    }
}

/// System set for LOD-related operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LodSystemSet;

/// Resource for configuring LOD distances
#[derive(Resource)]
pub struct LodConfig {
    /// Distance at which entities switch to medium LOD
    pub medium_distance: f32,
    /// Distance at which entities switch to sleep LOD
    pub sleep_distance: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            medium_distance: 150.0,
            sleep_distance: 300.0,
        }
    }
}
