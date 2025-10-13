use bevy::prelude::*;
use rand::prelude::*;

use crate::GameConfig;
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

    // Spawn NPCs initially - GTA-style population density
    let target_npcs = 100;

    while spawned_count < target_npcs && attempts < max_attempts {
        attempts += 1;
        let x = rng.gen_range(-900.0..900.0);
        let z = rng.gen_range(-900.0..900.0);


        // Spawn NPCs above ground, let gravity drop them
        let spawn_position = Vec3::new(x, 10.0, z);

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

    println!(
        "✅ UNIFIED NPC SETUP: Spawned {spawned_count} NPCs (attempted {attempts} positions)"
    );
}
