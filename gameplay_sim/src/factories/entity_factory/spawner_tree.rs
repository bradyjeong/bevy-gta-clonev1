use bevy::prelude::*;
use game_core::prelude::*;
use crate::factories::entity_factory::validation::*;

pub fn spawn_tree(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _existing_content: &[(Vec3, ContentType, f32)],
    _current_time: f32,
) -> Result<Option<Entity>, BundleError> {
    let safe_position = validate_position(position);
    
    // Create simple tree mesh (cylinder for trunk + sphere for leaves)
    let trunk_mesh = meshes.add(Cylinder::new(0.3, 3.0));
    let trunk_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.2, 0.1),
        ..default()
    });
    
    let entity = commands.spawn((
        Tree {
            height: 6.0,
            trunk_radius: 0.3,
            tree_type: TreeType::Oak,
            spawn_time: _current_time,
        },
        Cullable {
            is_culled: false,
            max_distance: 200.0,
        },
        MaterialMeshBundle {
            mesh: trunk_mesh,
            material: trunk_material,
            transform: Transform::from_translation(safe_position),
            ..default()
        },
    )).id();
    
    Ok(Some(entity))
}
