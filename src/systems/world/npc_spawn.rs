use crate::components::{
    NPC_LOD_CULL_DISTANCE, NPCAppearance, NPCBehaviorType, NPCGender, NPCLOD, NPCState, NPCType,
};
use bevy::{prelude::*, render::view::visibility::VisibilityRange};
use bevy_rapier3d::prelude::*;

use crate::config::GameConfig;
use crate::resources::WorldRng;
use crate::services::ground_detection::GroundDetectionService;
use crate::services::terrain_service::TerrainService;
use crate::services::timing_service::TimingService;
use rand::prelude::*;

/// Spawn NPCs using the new architecture while maintaining compatibility
/// This system replaces the old spawn_dynamic_npc function
/// CONSOLIDATED: Now uses spawn validation from UnifiedEntityFactory
pub fn spawn_new_npc_system(
    mut commands: Commands,
    timing_service: Res<TimingService>,
    npc_query: Query<Entity, With<NPCState>>,
    ground_service: Res<GroundDetectionService>,
    terrain_service: Res<TerrainService>,
    mut world_rng: ResMut<WorldRng>,
    _config: Res<GameConfig>,
) {
    // Limit NPC spawning to avoid performance issues (unified entity limits)
    if npc_query.iter().count() >= 20 {
        // REDUCED: From 100 to 20 NPCs max
        return;
    }

    // Spawn new NPCs occasionally using unified spawning pipeline
    if timing_service.current_time % 10.0 < 0.1 {
        // REDUCED: From 5.0 to 10.0 seconds
        // Try to find a valid spawn position using unified validation
        for _ in 0..5 {
            // REDUCED: From 10 to 5 attempts
            let x = world_rng.global().gen_range(-50.0..50.0);
            let z = world_rng.global().gen_range(-50.0..50.0);
            let position = Vec2::new(x, z);

            if ground_service.is_spawn_position_valid(position) {
                spawn_simple_npc_with_ground_detection_simple(
                    &mut commands,
                    position,
                    &ground_service,
                    &terrain_service,
                    &mut world_rng,
                );
                break; // Found valid position, spawn and exit
            }
        }
    }
}

/// Spawn a single NPC with ground detection (simplified version without RapierContext)
pub fn spawn_simple_npc_with_ground_detection_simple(
    commands: &mut Commands,
    position: Vec2,
    ground_service: &GroundDetectionService,
    terrain_service: &TerrainService,
    world_rng: &mut WorldRng,
) -> Entity {
    // Use simplified ground detection
    let ground_height = ground_service.get_ground_height_simple(position, terrain_service);
    let ground_clearance = 0.02; // Very small clearance to avoid clipping
    let spawn_height = ground_height + ground_clearance; // Place NPC feet on ground

    let spawn_position = Vec3::new(position.x, spawn_height, position.y);

    // Create NPC with new state-based architecture
    let entity = commands
        .spawn((
            NPCState {
                npc_type: NPCType::Civilian,
                appearance: NPCAppearance {
                    height: 1.8, // Standard NPC height
                    build: world_rng.global().gen_range(0.8..1.2),
                    skin_tone: Color::linear_rgb(0.8, 0.7, 0.6),
                    hair_color: Color::linear_rgb(0.4, 0.3, 0.2),
                    shirt_color: Color::linear_rgb(
                        world_rng.global().gen_range(0.2..0.8),
                        world_rng.global().gen_range(0.2..0.8),
                        world_rng.global().gen_range(0.2..0.8),
                    ),
                    pants_color: Color::linear_rgb(
                        world_rng.global().gen_range(0.1..0.6),
                        world_rng.global().gen_range(0.1..0.6),
                        world_rng.global().gen_range(0.1..0.6),
                    ),
                    gender: if world_rng.global().gen_bool(0.5) {
                        NPCGender::Male
                    } else {
                        NPCGender::Female
                    },
                },
                behavior: NPCBehaviorType::Wandering,
                target_position: spawn_position,
                speed: world_rng.global().gen_range(2.0..4.0),
                current_lod: NPCLOD::Full,
                last_lod_check: 0.0,
            },
            Transform::from_translation(spawn_position),
            GlobalTransform::default(),
        ))
        .id();

    println!("DEBUG: Spawned NPC at {spawn_position:?} (ground: {ground_height:.2})",);
    entity
}

/// Spawn a single NPC with ground detection (full physics version)
pub fn spawn_simple_npc_with_ground_detection(
    commands: &mut Commands,
    position: Vec2,
    ground_service: &GroundDetectionService,
    rapier_context: &RapierContext,
    world_rng: &mut WorldRng,
) -> Entity {
    // Create NPC with new state-based architecture
    let npc_type = match world_rng.global().gen_range(0..4) {
        0 => NPCType::Civilian,
        1 => NPCType::Worker,
        2 => NPCType::Police,
        _ => NPCType::Emergency,
    };

    let npc_state = NPCState::new(npc_type);
    let height = npc_state.appearance.height;

    // Get ground height at spawn position
    let ground_y = ground_service.get_spawn_height(position, height, rapier_context);
    let spawn_position = Vec3::new(position.x, ground_y, position.y);

    // Use simplified entity creation
    commands
        .spawn((
            npc_state,
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, -height / 2.0, 0.0),
                Vec3::new(0.0, height / 2.0, 0.0),
                0.3,
            ),
            Velocity::zero(),
            Transform::from_translation(spawn_position),
            Visibility::Visible,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            VisibilityRange::abrupt(0.0, NPC_LOD_CULL_DISTANCE),
        ))
        .id()
}

/// Legacy spawn a single NPC using the simplified system
pub fn spawn_simple_npc(
    commands: &mut Commands,
    position: Vec3,
    world_rng: &mut WorldRng,
) -> Entity {
    // Create NPC with new state-based architecture
    let npc_type = match world_rng.global().gen_range(0..4) {
        0 => NPCType::Civilian,
        1 => NPCType::Worker,
        2 => NPCType::Police,
        _ => NPCType::Emergency,
    };

    let npc_state = NPCState::new(npc_type);
    let height = npc_state.appearance.height;

    // Use simplified entity creation
    commands
        .spawn((
            npc_state,
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, -height / 2.0, 0.0),
                Vec3::new(0.0, height / 2.0, 0.0),
                0.3,
            ),
            Velocity::zero(),
            Transform::from_translation(position),
            Visibility::Visible,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            VisibilityRange::abrupt(0.0, NPC_LOD_CULL_DISTANCE),
        ))
        .id()
}

/// NPC spawn using unified factory (replaces legacy functions)
pub fn spawn_npc_with_new_architecture(
    commands: &mut Commands,
    position: Vec3,
    world_rng: &mut WorldRng,
) -> Entity {
    // Create NPC with new state-based architecture
    let npc_type = match world_rng.global().gen_range(0..4) {
        0 => NPCType::Civilian,
        1 => NPCType::Worker,
        2 => NPCType::Police,
        _ => NPCType::Emergency,
    };

    let npc_state = NPCState::new(npc_type);
    let height = npc_state.appearance.height;

    #[allow(deprecated)]
    commands
        .spawn((
            // Use new simplified system
            npc_state,
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, -height / 2.0, 0.0),
                Vec3::new(0.0, height / 2.0, 0.0),
                0.3,
            ),
            Velocity::zero(),
            Transform::from_translation(position),
            Visibility::Visible,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            VisibilityRange::abrupt(0.0, NPC_LOD_CULL_DISTANCE),
        ))
        .id()
}
