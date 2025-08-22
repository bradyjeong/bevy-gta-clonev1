use bevy::prelude::*;
use rand::prelude::*;

use crate::GameConfig;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::services::ground_detection::GroundDetectionService;

/// UNIFIED NPC SETUP SYSTEM
/// Consolidates setup_new_npcs (good patterns) and setup_npcs (bad patterns)
/// Features:
/// - ✅ Ground detection and spawn validation
/// - ✅ UnifiedEntityFactory for consistent spawning
/// - ✅ Proper physics and collision setup
/// - ✅ Safety features: max attempts, position validation
/// - ✅ Performance: reduced spawn count with LOD handling
pub fn setup_initial_npcs_unified(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground_service: Res<GroundDetectionService>,
    game_config: Res<GameConfig>,
) {
    // Initialize UnifiedEntityFactory for consistent spawning
    let mut entity_factory = UnifiedEntityFactory::with_config(game_config.clone());

    let mut rng = thread_rng();
    let mut spawned_count = 0;
    let max_attempts = 100; // Prevent infinite loop
    let mut attempts = 0;

    // Spawn fewer NPCs initially - LOD system will handle performance
    // Using 25 like the good setup_new_npcs function
    let target_npcs = 25;

    while spawned_count < target_npcs && attempts < max_attempts {
        attempts += 1;
        let x = rng.gen_range(-900.0..900.0);
        let z = rng.gen_range(-900.0..900.0);
        let position = Vec2::new(x, z);

        // Use ground detection service for spawn validation (from good implementation)
        if ground_service.is_spawn_position_valid(position) {
            let ground_height = ground_service.get_ground_height_simple(position);
            let spawn_position = Vec3::new(x, ground_height + 0.1, z);

            // Use UnifiedEntityFactory for consistent spawning
            match entity_factory.spawn_npc_consolidated(
                &mut commands,
                &mut meshes,
                &mut materials,
                spawn_position,
                0.0, // Initial time
            ) {
                Ok(_entity) => {
                    spawned_count += 1;
                    println!(
                        "DEBUG: Spawned NPC at {:?} (ground: {:.2})",
                        spawn_position, ground_height
                    );
                }
                Err(e) => {
                    println!(
                        "WARNING: Failed to spawn NPC at {:?}: {:?}",
                        spawn_position, e
                    );
                }
            }
        }
    }

    println!(
        "✅ UNIFIED NPC SETUP: Spawned {} NPCs with ground detection (attempted {} positions)",
        spawned_count, attempts
    );
}
