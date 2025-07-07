use bevy::prelude::*;
use game_core::prelude::*;
use crate::factories::entity_factory::validation::*;

pub fn spawn_npc(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _existing_content: &[(Vec3, ContentType, f32)],
    _current_time: f32,
) -> Result<Option<Entity>, BundleError> {
    let safe_position = validate_position(position);
    
    // Create simple NPC mesh
    let mesh = meshes.add(Capsule3d::new(0.3, 1.8));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2),
        ..default()
    });
    
    let entity = commands.spawn((
        NPC {
            target_position: safe_position,
            speed: 2.0,
            last_update: _current_time,
            update_interval: 1.0,
            health: Some(100.0),
            max_health: Some(100.0),
            behavior_state: Some(NPCBehaviorState::Idle),
            spawn_time: Some(_current_time),
        },
        Cullable {
            is_culled: false,
            max_distance: 100.0,
        },
        MaterialMeshBundle {
            mesh,
            material,
            transform: Transform::from_translation(safe_position),
            ..default()
        },
    )).id();
    
    Ok(Some(entity))
}
