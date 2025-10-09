use crate::resources::NPCAssetCache;
use crate::systems::world::{
    npc::simple_npc_movement, npc_animation::npc_animation_system, npc_spawn::spawn_new_npc_system,
};
use bevy::prelude::*;

/// Plugin responsible for NPC spawning, behavior, and management
pub struct WorldNpcPlugin;

impl Plugin for WorldNpcPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NPCAssetCache::new())
            .add_systems(Startup, initialize_npc_assets)
            .add_systems(Update, spawn_new_npc_system)
            .add_systems(Update, simple_npc_movement)
            .add_systems(Update, npc_animation_system.after(simple_npc_movement))
            .add_systems(Update, log_cache_stats);
    }
}

fn initialize_npc_assets(
    mut cache: ResMut<NPCAssetCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cache.initialize_common_assets(&mut meshes, &mut materials);
}

fn log_cache_stats(cache: Res<NPCAssetCache>, mut timer: Local<Timer>, time: Res<Time>) {
    if timer.duration().as_secs_f32() == 0.0 {
        *timer = Timer::from_seconds(30.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if timer.just_finished() {
        let (hits, misses, hit_rate) = cache.stats();
        info!(
            "📊 NPC Asset Cache Stats - Hits: {}, Misses: {}, Hit Rate: {:.1}%",
            hits, misses, hit_rate
        );
    }
}
