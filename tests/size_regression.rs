// Size regression tests for CI integration
// Ensures hot-path components and resources remain cache-efficient

use bevy::prelude::*;
use std::mem::size_of;

// Import all components from the game
use gta_game::*;
use gta_game::shared::movement_tracking::SharedMovementTracker;

/// Maximum size for hot-path types (single cache line)
const CACHE_LINE_SIZE: usize = 64;

#[test]
fn hot_path_components_within_limit() {
    // Hot-path components accessed every frame
    assert!(size_of::<ControlState>() <= CACHE_LINE_SIZE, 
        "ControlState ({} bytes) exceeds cache line", size_of::<ControlState>());
    
    assert!(size_of::<HumanMovement>() <= CACHE_LINE_SIZE,
        "HumanMovement ({} bytes) exceeds cache line", size_of::<HumanMovement>());
    
    assert!(size_of::<HumanAnimation>() <= CACHE_LINE_SIZE,
        "HumanAnimation ({} bytes) exceeds cache line", size_of::<HumanAnimation>());
    
    assert!(size_of::<SharedMovementTracker>() <= CACHE_LINE_SIZE,
        "SharedMovementTracker ({} bytes) exceeds cache line", size_of::<SharedMovementTracker>());
    
    assert!(size_of::<VegetationLOD>() <= CACHE_LINE_SIZE,
        "VegetationLOD ({} bytes) exceeds cache line", size_of::<VegetationLOD>());
    
    // NPCCore is the hot-path part of the split NPC component
    assert!(size_of::<NPCCore>() <= CACHE_LINE_SIZE,
        "NPCCore ({} bytes) exceeds cache line", size_of::<NPCCore>());
}

#[test]
fn hot_path_resources_within_limit() {
    // Hot-path resources accessed every frame
    assert!(size_of::<ChunkTracker>() <= CACHE_LINE_SIZE,
        "ChunkTracker ({} bytes) exceeds cache line", size_of::<ChunkTracker>());
}

#[test]
fn bevy_builtin_components_check() {
    // Document Bevy built-in component sizes for reference
    // These are provided by Bevy and we can't control their size
    println!("Transform size: {} bytes", size_of::<Transform>());
    println!("GlobalTransform size: {} bytes", size_of::<GlobalTransform>());
    
    // Transform is actually 40 bytes in Bevy (Vec3 position + Quat rotation + Vec3 scale)
    // This is acceptable as it's a fundamental component optimized by Bevy
    assert!(size_of::<Transform>() <= CACHE_LINE_SIZE,
        "Transform ({} bytes) is within cache line", size_of::<Transform>());
}

#[test]
fn marker_components_are_zero_sized() {
    // Marker components should be zero-sized for efficiency
    assert_eq!(size_of::<Player>(), 0, "Player should be a zero-sized marker");
    assert_eq!(size_of::<ActiveEntity>(), 0, "ActiveEntity should be a zero-sized marker");
    assert_eq!(size_of::<PlayerControlled>(), 0, "PlayerControlled should be a zero-sized marker");
    assert_eq!(size_of::<AIControlled>(), 0, "AIControlled should be a zero-sized marker");
}

#[test]
fn cache_resources_documented() {
    // These resources are large but that's OK as they're cache/config types
    // We document their size for awareness but don't enforce limits
    
    // ChunkTables is a cache resource with HashMaps - size is dynamic and that's OK
    println!("ChunkTables size: {} bytes (cache resource, dynamic size OK)", 
        size_of::<ChunkTables>());
    
    // Document that ChunkTables is properly classified as a cache resource
    // It stores dynamic HashMap data and is not accessed in hot paths
    assert!(size_of::<ChunkTables>() > CACHE_LINE_SIZE,
        "ChunkTables is correctly classified as a cache resource");
}

#[test]
fn component_splitting_validation() {
    // Validate that split components maintain size benefits
    
    // NPCCore should be small (hot-path)
    assert!(size_of::<NPCCore>() <= CACHE_LINE_SIZE,
        "NPCCore hot-path component within limit");
    
    // NPCConfig can be larger (cold-path)
    println!("NPCConfig size: {} bytes (cold-path, size not critical)", 
        size_of::<NPCConfig>());
}

/// Print a size report for all components
#[test] 
fn print_size_report() {
    println!("\n=== Component Size Report ===\n");
    
    println!("Hot-Path Components:");
    println!("  ControlState: {} bytes", size_of::<ControlState>());
    println!("  HumanMovement: {} bytes", size_of::<HumanMovement>()); 
    println!("  HumanAnimation: {} bytes", size_of::<HumanAnimation>());
    println!("  SharedMovementTracker: {} bytes", size_of::<SharedMovementTracker>());
    println!("  VegetationLOD: {} bytes", size_of::<VegetationLOD>());
    println!("  NPCCore: {} bytes", size_of::<NPCCore>());
    
    println!("\nHot-Path Resources:");
    println!("  ChunkTracker: {} bytes", size_of::<ChunkTracker>());
    
    println!("\nBevy Built-in Components:");
    println!("  Transform: {} bytes", size_of::<Transform>());
    println!("  GlobalTransform: {} bytes", size_of::<GlobalTransform>());
    
    println!("\nCache Resources (large size OK):");
    println!("  ChunkTables: {} bytes", size_of::<ChunkTables>());
    
    println!("\n=== End Report ===\n");
}
