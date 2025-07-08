//! ───────────────────────────────────────────────
//! System:   Unified Factory Setup
//! Purpose:  Handles user interface display and interaction
//! Schedule: Update
//! Reads:    UnifiedEntityFactory, GameConfig, Time
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::config::GameConfig;

/// Setup system for the UnifiedEntityFactory resource
pub fn setup_unified_entity_factory(mut commands: Commands, config: Res<GameConfig>) {
    let factory = UnifiedEntityFactory::with_config(game_core::config::GameConfig::default());
    
    info!("✅ Phase 2.1: UnifiedEntityFactory initialized with consolidated spawn logic");
    info!("📊 Entity limits: Buildings: {}, Vehicles: {}, NPCs: {}, Trees: {}", 
        factory.limit_manager.max_buildings,
        factory.limit_manager.max_vehicles,
        factory.limit_manager.max_npcs,
        factory.limit_manager.max_trees
    );
    
    commands.insert_resource(factory);
}

/// Debug system to show entity factory statistics
pub fn unified_factory_debug_system(
    factory: Res<UnifiedEntityFactory>,
    time: Res<Time>,
) {
    // Only show stats every 10 seconds to avoid spam
    if (time.elapsed_secs() % 10.0) < time.delta_secs() {
        let (buildings, vehicles, npcs, trees) = factory.limit_manager.counts();
        
        info!(
            "🏭 UNIFIED FACTORY STATUS:\n\
            📊 Current Entity Counts:\n\
            • Buildings: {}/{} ({:.1}% full)\n\
            • Vehicles:  {}/{} ({:.1}% full)\n\
            • NPCs:      {}/{} ({:.1}% full)\n\
            • Trees:     {}/{} ({:.1}% full)\n\
            🚀 Position Cache Size: {} entries",
            buildings, factory.limit_manager.max_buildings, 
            (buildings as f32 / factory.limit_manager.max_buildings as f32) * 100.0,
            vehicles, factory.limit_manager.max_vehicles,
            (vehicles as f32 / factory.limit_manager.max_vehicles as f32) * 100.0,
            npcs, factory.limit_manager.max_npcs,
            (npcs as f32 / factory.limit_manager.max_npcs as f32) * 100.0,
            trees, factory.limit_manager.max_trees,
            (trees as f32 / factory.limit_manager.max_trees as f32) * 100.0,
            0 // TODO: Add position cache count
        );
    }
}
