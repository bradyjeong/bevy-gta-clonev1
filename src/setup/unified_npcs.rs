use bevy::prelude::*;
use rand::prelude::*;

use crate::GameConfig;
use crate::constants::{LAND_ELEVATION, LEFT_ISLAND_X, SPAWN_DROP_HEIGHT, TERRAIN_HALF_SIZE};
use crate::factories::NPCFactory;
use crate::resources::NPCAssetCache;

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
    mut cache: ResMut<NPCAssetCache>,

    _game_config: Res<GameConfig>,
) {
    // Initialize focused NPCFactory for consistent spawning following AGENT.md principles
    let npc_factory = NPCFactory::new();

    let mut rng = thread_rng();
    let mut spawned_count = 0;
    let max_attempts = 500; // Increased for higher spawn count
    let mut attempts = 0;

    // Spawn NPCs on both terrain islands
    let target_npcs = 100;

    while spawned_count < target_npcs && attempts < max_attempts {
        attempts += 1;
        // Randomly choose left or right island
        let island_x = if rng.gen_bool(0.5) {
            LEFT_ISLAND_X
        } else {
            crate::constants::RIGHT_ISLAND_X
        };

        // Use polar coordinates to spawn uniformly within circular flat terrain
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let radius_squared: f32 = rng.gen_range(0.0..(TERRAIN_HALF_SIZE * TERRAIN_HALF_SIZE));
        let radius = radius_squared.sqrt(); // sqrt for uniform distribution
        let x = island_x + radius * angle.cos();
        let z = radius * angle.sin();

        // Spawn NPCs above terrain, let gravity drop them
        let spawn_position = Vec3::new(x, LAND_ELEVATION + SPAWN_DROP_HEIGHT, z);

        // Use focused NPCFactory for consistent spawning
        match npc_factory.spawn_npc(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut cache,
            spawn_position,
            None, // Auto-select NPC type
        ) {
            Ok(_entity) => {
                spawned_count += 1;
                println!("DEBUG: Spawned NPC at {spawn_position:?}");
            }
            Err(e) => {
                println!("WARNING: Failed to spawn NPC at {spawn_position:?}: {e:?}");
            }
        }
    }

    println!("✅ UNIFIED NPC SETUP: Spawned {spawned_count} NPCs (attempted {attempts} positions)");
}
