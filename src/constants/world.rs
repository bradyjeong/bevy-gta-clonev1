/// World generation constants
/// 
/// These constants define the fundamental parameters for world streaming,
/// chunk loading, and spatial organization.

/// Chunk size in world units (meters)
/// This MUST be consistent across all world streaming systems:
/// - ChunkCoord::from_world_pos
/// - on_chunk_loaded observer 
/// - unified_world_streaming_system_v2
/// - cleanup_distant_content
pub const CHUNK_SIZE: f32 = 256.0;

/// Half chunk size for centering calculations
pub const HALF_CHUNK_SIZE: f32 = CHUNK_SIZE * 0.5;

/// Maximum render distance for content (in meters)
pub const MAX_RENDER_DISTANCE: f32 = 2500.0;

/// Chunk loading radius (in chunks)
pub const CHUNK_LOAD_RADIUS: usize = 3;

/// Content spawn radius within a chunk (in meters)
pub const CONTENT_SPAWN_RADIUS: f32 = 120.0; // Slightly less than half chunk
