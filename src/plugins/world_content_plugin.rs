// DEPRECATED: This plugin is replaced by AsyncChunkGenerationPlugin
// The synchronous layer-by-layer generation caused frame drops ("jolting")
// New system uses async generation with strict per-frame budgets for 60+ FPS
//
// DO NOT RE-ENABLE THIS PLUGIN - it will reintroduce performance issues
//
// Migration path:
// - Old: WorldContentPlugin with road/building/vehicle/vegetation layer systems
// - New: AsyncChunkGenerationPlugin with queue_async_chunk_generation + process_completed_chunks
//
// If you need the old generators for reference, see:
// - src/systems/world/generators/ for focused generator implementations
// - src/systems/world/layered_generation.rs for legacy coordinator

use bevy::prelude::*;

/// DEPRECATED: Replaced by AsyncChunkGenerationPlugin
/// This plugin caused frame drops due to synchronous generation on main thread
#[deprecated(
    since = "0.1.0",
    note = "Use AsyncChunkGenerationPlugin instead for smooth 60+ FPS"
)]
pub struct WorldContentPlugin;

#[allow(deprecated)]
impl Plugin for WorldContentPlugin {
    fn build(&self, _app: &mut App) {
        // Intentionally empty - do not add systems
        // This plugin is kept for backwards compatibility only
        warn!(
            "WorldContentPlugin is deprecated and does nothing. Use AsyncChunkGenerationPlugin instead."
        );
    }
}
