#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::bundles::VisibleChildBundle;
use crate::components::RoadEntity;
use crate::components::{ActiveEntity, ContentType, DynamicContent, NPC};
use crate::systems::world::road_mesh::{generate_road_markings_mesh, generate_road_mesh};
use crate::systems::world::road_network::RoadNetwork;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Resource, Default)]
pub struct RoadGenerationTimer {
    _timer: f32,
    _last_player_chunk: Option<(i32, i32)>,
}

#[deprecated(note = "Use AsyncChunkGenerationPlugin road pipeline")]
pub fn road_network_system(
    _commands: Commands,
    _road_network: ResMut<RoadNetwork>,
    _active_query: Query<&Transform, With<ActiveEntity>>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    _road_query: Query<(Entity, &Transform), With<RoadEntity>>,
    _time: Res<Time>,
    _timer: Local<RoadGenerationTimer>,
) {
}

#[deprecated(note = "Handled by placement grid")]
pub fn update_road_dependent_systems(
    _npcs_query: Query<(Entity, &Transform, &NPC)>,
    _road_network: Res<RoadNetwork>,
    _commands: Commands,
) {
}

pub fn is_on_road_spline(position: Vec3, road_network: &RoadNetwork, tolerance: f32) -> bool {
    for road in road_network.roads.values() {
        if is_point_on_road_spline(position, road, tolerance) {
            return true;
        }
    }
    false
}

fn is_point_on_road_spline(
    position: Vec3,
    road: &crate::systems::world::road_network::RoadSpline,
    tolerance: f32,
) -> bool {
    let samples = 50;
    let width = road.road_type.width();

    for i in 0..samples {
        let t = i as f32 / (samples - 1) as f32;
        let road_point = road.evaluate(t);
        let distance =
            Vec3::new(position.x - road_point.x, 0.0, position.z - road_point.z).length();

        if distance <= width * 0.5 + tolerance {
            return true;
        }
    }

    false
}

#[allow(dead_code)]
fn spawn_road_entity(
    commands: &mut Commands,
    road_id: u64,
    road: &crate::systems::world::road_network::RoadSpline,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use crate::constants::{STATIC_GROUP, VEHICLE_GROUP};

    let start_pos = road.evaluate(0.0);

    let road_material = create_road_material(&road.road_type, materials);
    let marking_material = create_marking_material(materials);

    let road_entity = commands
        .spawn((
            RoadEntity { road_id },
            Transform::from_translation(Vec3::new(start_pos.x, 0.0, start_pos.z)),
            GlobalTransform::default(),
            Visibility::default(),
            DynamicContent {
                content_type: ContentType::Road,
            },
            RigidBody::Fixed,
            create_road_collider(road),
            CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP),
        ))
        .id();

    let road_mesh = generate_road_mesh(road);
    commands.spawn((
        Mesh3d(meshes.add(road_mesh)),
        MeshMaterial3d(road_material),
        Transform::from_translation(Vec3::new(-start_pos.x, 0.0, -start_pos.z)),
        ChildOf(road_entity),
        VisibleChildBundle::default(),
    ));

    let marking_meshes = generate_road_markings_mesh(road);
    for marking_mesh in marking_meshes {
        commands.spawn((
            Mesh3d(meshes.add(marking_mesh)),
            MeshMaterial3d(marking_material.clone()),
            Transform::from_translation(Vec3::new(-start_pos.x, 0.01, -start_pos.z)),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
    }
}

fn create_road_collider(road: &crate::systems::world::road_network::RoadSpline) -> Collider {
    let width = road.road_type.width();
    let length = road.length();
    Collider::cuboid(width * 0.5, 0.02, length * 0.5)
}

fn create_road_material(
    road_type: &crate::systems::world::road_network::RoadType,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    use crate::systems::world::road_network::RoadType;

    let (base_color, roughness) = match road_type {
        RoadType::Highway => (Color::srgb(0.4, 0.4, 0.45), 0.8),
        RoadType::MainStreet => (Color::srgb(0.35, 0.35, 0.4), 0.8),
        RoadType::SideStreet => (Color::srgb(0.45, 0.45, 0.5), 0.7),
        RoadType::Alley => (Color::srgb(0.5, 0.5, 0.45), 0.6),
    };

    materials.add(StandardMaterial {
        base_color,
        perceptual_roughness: roughness,
        ..default()
    })
}

fn create_marking_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.95),
        perceptual_roughness: 0.6,
        ..default()
    })
}
