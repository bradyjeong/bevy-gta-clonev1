use crate::components::{NPC_LOD_CULL_DISTANCE, NPCState, NPCType};
use crate::constants::{
    LAND_ELEVATION, LEFT_ISLAND_X, RIGHT_ISLAND_X, SPAWN_DROP_HEIGHT, TERRAIN_HALF_SIZE,
};
use crate::systems::world::unified_world::UnifiedWorldManager;
use bevy::{prelude::*, render::view::visibility::VisibilityRange};
use bevy_rapier3d::prelude::*;

use crate::config::GameConfig;
use crate::resources::WorldRng;

use rand::prelude::*;

/// Spawn NPCs using the new architecture while maintaining compatibility
/// This system replaces the old spawn_dynamic_npc function
/// CONSOLIDATED: Now uses spawn validation from UnifiedEntityFactory
pub fn spawn_new_npc_system(
    mut commands: Commands,
    mut spawn_timer: Local<Timer>,
    time: Res<Time>,
    npc_query: Query<Entity, With<NPCState>>,
    world: Res<UnifiedWorldManager>,
    mut world_rng: ResMut<WorldRng>,
    _config: Res<GameConfig>,
) {
    // Initialize timer on first run
    if spawn_timer.duration().as_secs_f32() == 0.0 {
        *spawn_timer = Timer::from_seconds(10.0, TimerMode::Repeating);
    }

    // Tick the timer
    spawn_timer.tick(time.delta());

    // Limit NPC spawning to avoid performance issues (unified entity limits)
    if npc_query.iter().count() >= 150 {
        // GTA-style population density
        return;
    }

    // Spawn new NPCs on both islands (randomly choose left or right)
    if spawn_timer.just_finished() {
        let island_x = if world_rng.global().gen_bool(0.5) {
            LEFT_ISLAND_X
        } else {
            RIGHT_ISLAND_X
        };

        let x = island_x
            + world_rng
                .global()
                .gen_range(-TERRAIN_HALF_SIZE..TERRAIN_HALF_SIZE);
        let z = world_rng
            .global()
            .gen_range(-TERRAIN_HALF_SIZE..TERRAIN_HALF_SIZE);

        let test_position = Vec3::new(x, LAND_ELEVATION, z);

        // Validate spawn position is on terrain island
        if !world.is_on_terrain_island(test_position) {
            return; // Skip this spawn cycle, try again next time
        }

        // Spawn above terrain, let gravity drop NPCs
        let spawn_position = Vec3::new(x, LAND_ELEVATION + SPAWN_DROP_HEIGHT, z);

        // Use spawn_simple_npc which adds ALL required components
        spawn_simple_npc(&mut commands, spawn_position, &mut world_rng);

        println!("DEBUG: Spawned NPC at {spawn_position:?}");
    }
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
    // TODO: Migrate to NPCFactory for proper visuals
    commands
        .spawn((
            npc_state,
            crate::components::NPC {
                target_position: position,
                speed: world_rng.global().gen_range(2.0..4.0),
                last_update: 0.0,
                update_interval: 0.05,
            },
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, -height / 2.0, 0.0),
                Vec3::new(0.0, height / 2.0, 0.0),
                0.3, // TODO: Migrate to config.npc.capsule_radius (currently 0.4 in config but 0.3 historically)
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
    // TODO: Migrate to NPCFactory for proper visuals
    commands
        .spawn((
            // Use new simplified system
            npc_state,
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, -height / 2.0, 0.0),
                Vec3::new(0.0, height / 2.0, 0.0),
                0.3, // TODO: Migrate to config.npc.capsule_radius (currently 0.4 in config but 0.3 historically)
            ),
            Velocity::zero(),
            Transform::from_translation(position),
            Visibility::Visible,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            VisibilityRange::abrupt(0.0, NPC_LOD_CULL_DISTANCE),
        ))
        .id()
}
