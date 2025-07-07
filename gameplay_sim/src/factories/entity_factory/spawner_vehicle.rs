use bevy::prelude::*;
use game_core::prelude::*;
use crate::factories::entity_factory::validation::*;

pub fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _existing_content: &[(Vec3, ContentType, f32)],
    _current_time: f32,
) -> Result<Option<Entity>, BundleError> {
    let safe_position = validate_position(position);
    
    // Create simple vehicle mesh
    let mesh = meshes.add(Cuboid::new(1.8, 1.4, 4.2));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    });
    
    let entity = commands.spawn((
        Vehicle {
            max_speed: 60.0,
            current_speed: 0.0,
            fuel: 100.0,
            engine_power: 150.0,
            vehicle_type: VehicleType::Car,
            spawn_time: _current_time,
        },
        Cullable {
            is_culled: false,
            max_distance: 150.0,
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
