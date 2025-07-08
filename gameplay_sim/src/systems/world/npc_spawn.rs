//! ───────────────────────────────────────────────
//! System:   NPC Spawn
//! Purpose:  Spawns and manages NPCs in the world
//! Schedule: Update (throttled)
//! Reads:    `ActiveEntity`, Transform, `EntityLimits`
//! Writes:   Commands, NPC entities
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
// Removed bevy16_compat - using direct Bevy methods
use rand::Rng;
use game_core::prelude::*;
use crate::systems::world::npc_lod::spawn_npc_with_lod;

pub fn npc_spawn_system(
    mut commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    npc_query: Query<&Transform, With<NPC>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_config: Res<GameConfig>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let current_time = time.elapsed_secs();
        
        // Throttle NPC spawning to every 10 seconds
        if current_time % 10.0 < 0.1 {
            let existing_npcs = npc_query.iter().count();
            let max_npcs = 50; // Conservative limit
            
            if existing_npcs < max_npcs {
                // Spawn a few NPCs around the player
                for _ in 0..3 {
                    let spawn_pos = generate_npc_spawn_position(active_pos);
                    spawn_npc_unified(&mut commands, spawn_pos, &mut meshes, &mut materials);
                }
            }
        }
    }
}

fn generate_npc_spawn_position(active_pos: Vec3) -> Vec3 {
    let mut rng = rand::thread_rng();
    let distance = rng.gen_range(50.0..150.0);
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    
    Vec3::new(
        active_pos.x + distance * angle.cos(),
        active_pos.y,
        active_pos.z + distance * angle.sin(),
    )
}

pub fn spawn_npc_unified(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_npc_with_lod(commands, position, meshes, materials)
}

pub fn cleanup_distant_npcs_system(
    mut commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    npc_query: Query<(Entity, &Transform), With<NPC>>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let cleanup_distance = 500.0; // Much larger than spawn distance
        
        for (entity, npc_transform) in npc_query.iter() {
            if active_pos.distance(npc_transform.translation) > cleanup_distance {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// Legacy function signatures for compatibility
pub fn spawn_simple_npc_with_ground_detection(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_npc_unified(commands, position, meshes, materials)
}

pub fn spawn_simple_npc(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_npc_unified(commands, position, meshes, materials)
}

pub fn spawn_npc_with_new_architecture(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    spawn_npc_unified(commands, position, meshes, materials)
}

pub fn migrate_legacy_npcs(
    mut commands: Commands,
    legacy_npc_query: Query<Entity, (With<NPC>, Without<DynamicContent>)>,
) {
    for entity in legacy_npc_query.iter() {
        // Add missing components to legacy NPCs
        commands.entity(entity).insert((
            DynamicContent {
                content_type: ContentType::NPC,
            },
            Cullable {
                max_distance: 200.0,
                is_culled: false,
            },
        ));
        
        println!("DEBUG: Migrated NPC entity {entity:?} to unified architecture");
    }
}
