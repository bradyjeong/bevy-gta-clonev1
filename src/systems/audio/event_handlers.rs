use bevy::prelude::*;
use crate::events::cross_plugin_events::*;

/// Handle footstep sound requests
pub fn handle_footstep_sound_request_system(
    mut events: EventReader<RequestFootstepSound>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut audio_instances: ResMut<AudioInstances>,
) {
    for event in events.read() {
        // Determine sound file based on surface and movement type
        let sound_file = match (event.surface_type, event.is_running) {
            (SurfaceType::Concrete, true) => "audio/footstep_concrete_run.ogg",
            (SurfaceType::Concrete, false) => "audio/footstep_concrete_walk.ogg",
            (SurfaceType::Grass, true) => "audio/footstep_grass_run.ogg",
            (SurfaceType::Grass, false) => "audio/footstep_grass_walk.ogg",
            (SurfaceType::Metal, _) => "audio/footstep_metal.ogg",
            (SurfaceType::Wood, _) => "audio/footstep_wood.ogg",
        };
        
        // Spawn spatial audio source
        let audio_entity = commands.spawn((
            AudioPlayer::<AudioSource>(asset_server.load(sound_file)),
            PlaybackSettings::DESPAWN,
            SpatialAudioBundle {
                source: SpatialAudioSource {
                    position: event.position,
                    max_distance: 20.0,
                    ..default()
                },
                ..default()
            },
            FootstepAudio {
                owner: event.entity,
            },
        )).id();
        
        // Track audio instance for cleanup
        audio_instances.footsteps.insert(event.entity, audio_entity);
    }
}

/// Handle audio cleanup requests
pub fn handle_audio_cleanup_request_system(
    mut events: EventReader<RequestAudioCleanup>,
    mut commands: Commands,
    mut audio_instances: ResMut<AudioInstances>,
) {
    for event in events.read() {
        // Clean up footstep sounds
        if let Some(audio_entity) = audio_instances.footsteps.remove(&event.entity) {
            if let Some(entity_commands) = commands.get_entity(audio_entity) {
                entity_commands.despawn_recursive();
            }
        }
        
        // Clean up any other audio sources associated with this entity
        audio_instances.cleanup_entity(event.entity);
    }
}

/// Resource to track active audio instances
#[derive(Resource, Default)]
pub struct AudioInstances {
    pub footsteps: bevy::utils::HashMap<Entity, Entity>,
    pub engines: bevy::utils::HashMap<Entity, Entity>,
    pub effects: bevy::utils::HashMap<Entity, Vec<Entity>>,
}

impl AudioInstances {
    pub fn cleanup_entity(&mut self, entity: Entity) {
        self.footsteps.remove(&entity);
        self.engines.remove(&entity);
        self.effects.remove(&entity);
    }
}

/// Component to mark footstep audio entities
#[derive(Component)]
pub struct FootstepAudio {
    pub owner: Entity,
}

/// Spatial audio bundle for 3D positioned sounds
#[derive(Bundle, Default)]
pub struct SpatialAudioBundle {
    pub source: SpatialAudioSource,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Spatial audio source configuration
#[derive(Component, Default)]
pub struct SpatialAudioSource {
    pub position: Vec3,
    pub max_distance: f32,
    pub rolloff_factor: f32,
}

// Note: In a real implementation, you would use bevy_kira_audio or another
// audio plugin that supports spatial audio. This is a simplified version
// showing the event-based architecture.
