use bevy::prelude::*;
use game_core::prelude::*;
use crate::factories::entity_factory::validation::validate_position;


pub fn spawn_building(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _existing_content: &[(Vec3, ContentType, f32)],
    _current_time: f32,
) -> Result<Option<Entity>, BundleError> {
    let safe_position = validate_position(position);
    
    // Create simple building mesh
    let mesh = meshes.add(Cuboid::new(8.0, 12.0, 8.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.4),
        ..default()
    });
    
    let entity = commands.spawn((
        Building {
            building_type: BuildingType::Residential,
            height: 12.0,
            scale: Vec3::ONE,
            max_occupants: Some(4),
            current_occupants: Some(0),
            spawn_time: Some(_current_time),
        },
        Cullable {
            is_culled: false,
            max_distance: 300.0,
        },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(safe_position),
    )).id();
    
    Ok(Some(entity))
}
