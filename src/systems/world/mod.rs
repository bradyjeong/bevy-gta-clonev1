// pub mod culling; // DELETED: Using Bevy's built-in VisibilityRange instead
pub mod debug;
pub mod npc;
pub mod performance;
pub mod road_generation;
pub mod road_mesh;
pub mod road_network;

// NEW UNIFIED WORLD SYSTEM
// pub mod async_chunk_generation; // REMOVED: Dead code for streaming
pub mod generators; // NEW: Focused chunk generators following AGENT.md simplicity principles
pub mod layered_generation;
pub mod npc_spawn;
pub mod simulation_lod;
pub mod unified_world;
// pub mod optimized_lod; // Removed - functionality moved to unified_lod.rs
pub mod dynamic_physics_culling;
pub mod physics_activation;

pub mod debug_layers;

// pub mod unified_distance_culling; - REMOVED: Replaced with Bevy's VisibilityRange
pub mod boundaries;
pub mod boundary_effects;
pub mod unified_factory_setup;
// pub mod asset_streaming; - REMOVED: Dead code, not used
// pub mod floating_origin; - REMOVED: Finite world doesn't need floating origin

// All wildcard exports removed - use explicit paths like world::debug::toggle_debug_overlay
// This enforces clear dependency relationships and prevents hidden coupling

// Example usage:
// - world::road_generation::spawn_roads()
// - world::npc::spawn_npc()
// - world::unified_world::UnifiedWorldSystem
