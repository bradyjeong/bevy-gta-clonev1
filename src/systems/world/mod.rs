// pub mod culling; // DELETED: Using Bevy's built-in VisibilityRange instead
pub mod debug;
pub mod npc;
pub mod performance;
pub mod road_generation;
pub mod road_mesh;
pub mod road_network;

// NEW UNIFIED WORLD SYSTEM
pub mod async_chunk_generation;
pub mod generators; // NEW: Focused chunk generators following AGENT.md simplicity principles
pub mod layered_generation;
pub mod npc_spawn;
pub mod simulation_lod;
pub mod unified_world;
// pub mod optimized_lod; // Removed - functionality moved to unified_lod.rs
pub mod asset_streaming;
pub mod bevy_vegetation_lod;
pub mod debug_layers;
pub mod vegetation_lod;
// pub mod unified_distance_culling; - REMOVED: Replaced with Bevy's VisibilityRange
pub mod boundaries;
pub mod boundary_effects;
pub mod floating_origin;
pub mod unified_factory_setup;

// All wildcard exports removed - use explicit paths like world::debug::toggle_debug_overlay
// This enforces clear dependency relationships and prevents hidden coupling

// Example usage:
// - world::road_generation::spawn_roads()
// - world::npc::spawn_npc()
// - world::unified_world::UnifiedWorldSystem
