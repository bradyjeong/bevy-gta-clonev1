use crate::factories::{EntityLimitManager, EntityType};
use bevy::prelude::*;

/// Setup system for the focused entity factories and limit manager
pub fn setup_entity_factories(mut commands: Commands) {
    let limit_manager = EntityLimitManager::new();

    info!("Focused factory system initialized following AGENT.md simplicity principles");
    info!(
        "Entity limits: Buildings: {}, Vehicles: {}, NPCs: {}, Trees: {}",
        limit_manager
            .limits
            .get(&EntityType::Building)
            .map(|l| l.max_count)
            .unwrap_or(0),
        limit_manager
            .limits
            .get(&EntityType::Vehicle)
            .map(|l| l.max_count)
            .unwrap_or(0),
        limit_manager
            .limits
            .get(&EntityType::NPC)
            .map(|l| l.max_count)
            .unwrap_or(0),
        limit_manager
            .limits
            .get(&EntityType::Tree)
            .map(|l| l.max_count)
            .unwrap_or(0),
    );

    commands.insert_resource(limit_manager);
}

/// Debug system to show entity limit statistics
pub fn entity_limit_debug_system(limit_manager: Res<EntityLimitManager>, time: Res<Time>) {
    // Only show stats every 10 seconds to avoid spam
    if (time.elapsed_secs() % 10.0) < time.delta_secs() {
        let building_current = limit_manager
            .current_counts
            .get(&EntityType::Building)
            .unwrap_or(&0);
        let building_max = limit_manager
            .limits
            .get(&EntityType::Building)
            .map(|l| l.max_count)
            .unwrap_or(0);

        let vehicle_current = limit_manager
            .current_counts
            .get(&EntityType::Vehicle)
            .unwrap_or(&0);
        let vehicle_max = limit_manager
            .limits
            .get(&EntityType::Vehicle)
            .map(|l| l.max_count)
            .unwrap_or(0);

        let npc_current = limit_manager
            .current_counts
            .get(&EntityType::NPC)
            .unwrap_or(&0);
        let npc_max = limit_manager
            .limits
            .get(&EntityType::NPC)
            .map(|l| l.max_count)
            .unwrap_or(0);

        let tree_current = limit_manager
            .current_counts
            .get(&EntityType::Tree)
            .unwrap_or(&0);
        let tree_max = limit_manager
            .limits
            .get(&EntityType::Tree)
            .map(|l| l.max_count)
            .unwrap_or(0);

        info!(
            "ENTITY LIMIT STATUS:\n\
            Current Entity Counts:\n\
            • Buildings: {}/{} ({:.1}% full)\n\
            • Vehicles:  {}/{} ({:.1}% full)\n\
            • NPCs:      {}/{} ({:.1}% full)\n\
            • Trees:     {}/{} ({:.1}% full)",
            building_current,
            building_max,
            (*building_current as f32 / building_max.max(1) as f32) * 100.0,
            vehicle_current,
            vehicle_max,
            (*vehicle_current as f32 / vehicle_max.max(1) as f32) * 100.0,
            npc_current,
            npc_max,
            (*npc_current as f32 / npc_max.max(1) as f32) * 100.0,
            tree_current,
            tree_max,
            (*tree_current as f32 / tree_max.max(1) as f32) * 100.0,
        );
    }
}
