use crate::constants::{
    CHARACTER_GROUP, LAND_ELEVATION, LEFT_ISLAND_X, RIGHT_ISLAND_X, STATIC_GROUP,
    TERRAIN_HALF_SIZE, VEHICLE_GROUP,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_bridge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let end_clearance = 1.0;
    let gap_len = (RIGHT_ISLAND_X - TERRAIN_HALF_SIZE) - (LEFT_ISLAND_X + TERRAIN_HALF_SIZE);
    let half_len_x = 0.5 * gap_len - end_clearance;
    let half_width_z = 12.0;
    let half_thickness_y = 1.0;
    let center_y = LAND_ELEVATION - half_thickness_y;
    let bridge_z_offset = -300.0;

    let mesh_size_x = 2.0 * half_len_x;
    let mesh_size_y = 2.0 * half_thickness_y;
    let mesh_size_z = 2.0 * half_width_z;

    let collider_width_z = 0.95 * half_width_z;

    let concrete_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.4),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(mesh_size_x, mesh_size_y, mesh_size_z))),
        MeshMaterial3d(concrete_material),
        Transform::from_translation(Vec3::new(0.0, center_y, bridge_z_offset)),
        RigidBody::Fixed,
        Collider::cuboid(half_len_x, half_thickness_y, collider_width_z),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Friction {
            coefficient: 0.8,
            combine_rule: CoefficientCombineRule::Average,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Name::new("Bridge Deck"),
    ));

    info!(
        "Bridge spawned: {:.1}m long × {:.1}m wide at Z={:.1}, deck top at Y={:.1}",
        mesh_size_x, mesh_size_z, bridge_z_offset, LAND_ELEVATION
    );
}
