use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;

use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::services::ground_detection::GroundDetectionService;
use crate::GameConfig;
use crate::systems::world::unified_distance_culling::UnifiedCullable;

use crate::components::world::NPC;

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
                    println!("DEBUG: Spawned NPC at {:?} (ground: {:.2})", spawn_position, ground_height);
                }
                Err(e) => {
                    println!("WARNING: Failed to spawn NPC at {:?}: {:?}", spawn_position, e);
                }
            }
        }
    }
    
    println!("✅ UNIFIED NPC SETUP: Spawned {} NPCs with ground detection (attempted {} positions)", spawned_count, attempts);
}

/// Legacy spawn function for compatibility - creates NPC with simplified approach
/// Used if UnifiedEntityFactory is not available
#[allow(dead_code)]
pub fn spawn_npc_legacy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    ground_service: &GroundDetectionService,
) -> Entity {
    let mut rng = thread_rng();
    
    // Ground detection
    let ground_height = ground_service.get_ground_height_simple(Vec2::new(position.x, position.z));
    let spawn_position = Vec3::new(position.x, ground_height + 0.1, position.z);
    
    // Random NPC colors (from old setup_npcs)
    let npc_colors = [
        Color::srgb(0.8, 0.6, 0.4), // Skin tone 1
        Color::srgb(0.6, 0.4, 0.3), // Skin tone 2
        Color::srgb(0.9, 0.7, 0.5), // Skin tone 3
        Color::srgb(0.7, 0.5, 0.4), // Skin tone 4
    ];
    let color = npc_colors[rng.gen_range(0..npc_colors.len())];
    
    // Random target position for movement
    let target_x = rng.gen_range(-900.0..900.0);
    let target_z = rng.gen_range(-900.0..900.0);
    let target_position = Vec3::new(target_x, ground_height + 0.1, target_z);
    
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
        MeshMaterial3d(materials.add(color)),
        Transform::from_translation(spawn_position),
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -0.9, 0.0), Vec3::new(0.0, 0.9, 0.0), 0.3),
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        NPC {
            target_position,
            speed: rng.gen_range(2.0..5.0),
            last_update: 0.0,
            update_interval: rng.gen_range(0.05..0.2),
        },
        UnifiedCullable::npc(),
    )).id()
}
