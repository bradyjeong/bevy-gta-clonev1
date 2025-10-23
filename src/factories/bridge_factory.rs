use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_bridge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &GameConfig,
    env: &WorldEnvConfig,
) {
    let end_clearance = 1.0;
    let gap_len = (env.islands.right_x - env.terrain.half_size)
        - (env.islands.left_x + env.terrain.half_size);

    let min_gap = end_clearance * 2.0;
    if gap_len <= min_gap {
        warn!(
            "Islands too close to spawn bridge: gap={:.1}m, min_required={:.1}m. Skipping bridge spawn.",
            gap_len, min_gap
        );
        return;
    }

    let half_len_x = 0.5 * gap_len - end_clearance;
    let half_width_z = 12.0;
    let half_thickness_y = 1.0;
    let center_y = env.land_elevation - half_thickness_y;
    let bridge_z_offset = -300.0;

    let mesh_size_x = 2.0 * half_len_x;
    let mesh_size_y = 2.0 * half_thickness_y;
    let mesh_size_z = 2.0 * half_width_z;

    let collider_width_z = half_width_z;

    let bridge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(192.0 / 255.0, 80.0 / 255.0, 77.0 / 255.0),
        perceptual_roughness: 0.75,
        metallic: 0.05,
        ..default()
    });

    let deck_top_y = half_thickness_y;

    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(mesh_size_x, mesh_size_y, mesh_size_z))),
            MeshMaterial3d(bridge_material.clone()),
            Transform::from_translation(Vec3::new(0.0, center_y, bridge_z_offset)),
            RigidBody::Fixed,
            Collider::cuboid(half_len_x, half_thickness_y, collider_width_z),
            CollisionGroups::new(
                config.physics.static_group,
                config.physics.vehicle_group | config.physics.character_group,
            ),
            Friction {
                coefficient: 0.8,
                combine_rule: CoefficientCombineRule::Average,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Name::new("Bridge Deck"),
        ))
        .with_children(|parent| {
            let rail_offset_z = half_width_z - 0.5;
            let h_post = 1.2;
            let r_post = 0.08;
            let x_margin = 1.5;
            let s_post = 4.0;
            let x_start = -half_len_x + x_margin;
            let x_end = half_len_x - x_margin;
            let rail_length = x_end - x_start;

            let post_mesh = meshes.add(Cylinder::new(r_post, h_post));
            let top_rail_mesh = meshes.add(Cuboid::new(rail_length, 0.1, 0.1));

            let barrier_h = 0.9;
            let barrier_t = 0.3;
            let barrier_y = deck_top_y + barrier_h * 0.5;
            let barrier_x = (x_start + x_end) * 0.5;

            for side_z in [rail_offset_z, -rail_offset_z] {
                let mut x = x_start;
                while x <= x_end {
                    parent.spawn((
                        Mesh3d(post_mesh.clone()),
                        MeshMaterial3d(bridge_material.clone()),
                        Transform::from_translation(Vec3::new(
                            x,
                            deck_top_y + h_post / 2.0,
                            side_z,
                        )),
                        Name::new("Railing Post"),
                    ));
                    x += s_post;
                }

                parent.spawn((
                    Mesh3d(top_rail_mesh.clone()),
                    MeshMaterial3d(bridge_material.clone()),
                    Transform::from_translation(Vec3::new(
                        (x_start + x_end) / 2.0,
                        deck_top_y + h_post - 0.05,
                        side_z,
                    )),
                    Name::new("Top Rail"),
                ));

                let barrier_z = side_z - barrier_t * 0.5;
                parent.spawn((
                    Collider::cuboid(rail_length * 0.5, barrier_h * 0.5, barrier_t * 0.5),
                    Transform::from_translation(Vec3::new(barrier_x, barrier_y, barrier_z)),
                    CollisionGroups::new(
                        config.physics.static_group,
                        config.physics.vehicle_group | config.physics.character_group,
                    ),
                    Name::new("Side Barrier"),
                ));
            }
        });

    info!(
        "Bridge spawned: {:.1}m long × {:.1}m wide at Z={:.1}, deck top at Y={:.1}",
        mesh_size_x, mesh_size_z, bridge_z_offset, env.land_elevation
    );
}
