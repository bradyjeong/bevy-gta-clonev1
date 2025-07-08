//! ───────────────────────────────────────────────
//! Module:   Road Network 
//! Purpose:  Road generation and network management
//! Schedule: `DynamicContent`
//! Reads:    `GameConfig`, Transform, `ChunkData`
//! Writes:   Commands, `RoadNetwork`\
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;

// All impl blocks removed to fix orphan rule violations
// RoadNetwork implementations should be in game_core where the type is defined

// This file now only contains local helper types and functions

// Local helper types for road generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoadType {
    Street,
    Highway,
    Residential,
    Commercial,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceType {
    Asphalt,
    Concrete,
    Dirt,
    Gravel,
}

// Local helper functions for road generation
#[must_use] pub fn generate_road_mesh(width: f32, length: f32) -> Vec<Vec3> {
    vec![
        Vec3::new(-width/2.0, 0.0, -length/2.0),
        Vec3::new(width/2.0, 0.0, -length/2.0),
        Vec3::new(width/2.0, 0.0, length/2.0),
        Vec3::new(-width/2.0, 0.0, length/2.0),
    ]
}

#[must_use] pub fn calculate_road_curvature(start: Vec3, end: Vec3, control: Vec3) -> f32 {
    let mid = (start + end) / 2.0;
    let deviation = (control - mid).length();
    let base_distance = (end - start).length();
    
    if base_distance > 0.0 {
        deviation / base_distance
    } else {
        0.0
    }
}

// NOTE: RoadEntity and IntersectionEntity are defined in components/world.rs
