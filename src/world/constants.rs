// Shared constants between v1 and v2 world systems
// Phase 2: Oracle-approved constant decoupling

/// Size of world chunks in world units
pub const UNIFIED_CHUNK_SIZE: f32 = 100.0;

/// Default streaming radius in chunks
pub const UNIFIED_STREAMING_RADIUS: i32 = 5;

/// Maximum chunks to process per frame
pub const MAX_CHUNKS_PER_FRAME: usize = 2;

/// Default LOD levels
pub const LOD_HIGH: u8 = 0;
pub const LOD_MEDIUM: u8 = 1;
pub const LOD_LOW: u8 = 2;

/// Content type collision radii
pub const BUILDING_COLLISION_RADIUS: f32 = 35.0;
pub const VEHICLE_COLLISION_RADIUS: f32 = 25.0;
pub const TREE_COLLISION_RADIUS: f32 = 10.0;
pub const NPC_COLLISION_RADIUS: f32 = 5.0;

/// Road network constants
pub const MAX_ROAD_NODES: usize = 256;
pub const MAX_ROAD_SEGMENTS: usize = 512;

/// PlacementGrid constants
pub const DEFAULT_GRID_CELL_SIZE: f32 = 10.0;
pub const PLACEMENT_SAFETY_MARGIN: f32 = 2.0;
