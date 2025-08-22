use bevy::prelude::*;
use crate::components::player::ActiveEntity;

/// Optional floating origin system for future scaling beyond 12km
/// Automatically recenters world coordinates when player moves too far from origin
/// This prevents floating-point precision loss in very large worlds
/// 
/// Currently disabled since 12km finite world is well within f32 precision limits
/// Enable this when expanding beyond 16km or for procedural infinite worlds

#[derive(Resource, Debug)]
pub struct FloatingOrigin {
    /// Distance threshold before origin rebasing (default: 2048.0)
    pub rebase_threshold: f32,
    /// Global offset tracking for networking/save systems
    pub global_offset: Vec3,
    /// Whether floating origin is enabled
    pub enabled: bool,
}

impl Default for FloatingOrigin {
    fn default() -> Self {
        Self {
            rebase_threshold: 2048.0, // 2km threshold
            global_offset: Vec3::ZERO,
            enabled: false, // Disabled for finite world
        }
    }
}

impl FloatingOrigin {
    pub fn new(threshold: f32, enabled: bool) -> Self {
        Self {
            rebase_threshold: threshold,
            global_offset: Vec3::ZERO,
            enabled,
        }
    }
}

/// Floating origin system - runs at end of each frame
/// Recenters world when active entity moves too far from origin
pub fn floating_origin_system(
    mut floating_origin: ResMut<FloatingOrigin>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut all_transforms: Query<&mut Transform, Without<ActiveEntity>>,
    // Note: RapierContext shifting would be implemented here for full floating origin
    // Currently commented out as it's complex and not needed for 12km finite world
) {
    // Skip if floating origin is disabled
    if !floating_origin.enabled {
        return;
    }
    
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Check if rebase is needed
    let distance_from_origin = active_pos.length();
    if distance_from_origin <= floating_origin.rebase_threshold {
        return;
    }
    
    // Calculate offset to recenter world
    let rebase_offset = Vec3::new(active_pos.x, 0.0, active_pos.z); // Don't rebase Y axis
    
    info!("Floating origin rebase: offset={:?}, distance={:.1}m", 
          rebase_offset, distance_from_origin);
    
    // Shift all transforms back toward origin
    for mut transform in &mut all_transforms {
        transform.translation -= rebase_offset;
    }
    
    // Shift physics world (Rapier context)
    // Note: This requires careful handling in Rapier 0.30+
    // For now, we log the operation - full implementation would need Rapier API research
    info!("Would shift Rapier physics context by {:?}", -rebase_offset);
    
    // Update global offset for networking/save systems
    floating_origin.global_offset += rebase_offset;
    
    info!("Floating origin rebase completed. New global offset: {:?}", 
          floating_origin.global_offset);
}

/// Plugin for floating origin system (optional - disabled by default)
pub struct FloatingOriginPlugin {
    pub threshold: f32,
    pub enabled: bool,
}

impl Default for FloatingOriginPlugin {
    fn default() -> Self {
        Self {
            threshold: 2048.0,
            enabled: false, // Disabled for finite world
        }
    }
}

impl FloatingOriginPlugin {
    pub fn new(threshold: f32, enabled: bool) -> Self {
        Self { threshold, enabled }
    }
}

impl Plugin for FloatingOriginPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(FloatingOrigin::new(self.threshold, self.enabled))
            .add_systems(PostUpdate, floating_origin_system);
    }
}
