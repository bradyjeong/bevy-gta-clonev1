use crate::bundles::VisibleChildBundle;
use crate::components::ContentType;
use crate::constants::STATIC_GROUP;
use crate::resources::WorldRng;
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub struct VegetationGenerator;

impl VegetationGenerator {
    pub fn generate_vegetation(
        &self,
        commands: &mut Commands,
        world: &mut UnifiedWorldManager,
        coord: ChunkCoord,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        world_rng: &mut WorldRng,
    ) {
        let chunk_center = coord.to_world_pos();
        let half_size = world.chunk_size * 0.5;

        // Skip vegetation generation for chunks near world edge (±2000m)
        const WORLD_HALF_SIZE: f32 = 2000.0;
        const EDGE_BUFFER: f32 = 200.0;
        if chunk_center.x.abs() > WORLD_HALF_SIZE - EDGE_BUFFER
            || chunk_center.z.abs() > WORLD_HALF_SIZE - EDGE_BUFFER
        {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.vegetation_generated = true;
            }
            return;
        }

        // Determine vegetation density based on distance from center
        let distance_from_center = Vec2::new(chunk_center.x, chunk_center.z).length();
        let vegetation_density = (1.0 - (distance_from_center / 2000.0).min(0.7)).max(0.2);

        // Generate palm tree positions - scaled by density
        let tree_attempts = (vegetation_density * 5.0) as usize;

        for _ in 0..tree_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);

            // Check if position is valid (not on road, not overlapping other trees, not in water)
            if !self.is_on_road(position, world)
                && !self.is_in_water_area(position)
                && world
                    .placement_grid
                    .can_place(position, ContentType::Tree, 3.0, 10.0)
            {
                if let Ok(tree_entity) =
                    self.spawn_palm_tree(commands, coord, position, meshes, materials)
                {
                    // Add to placement grid
                    world
                        .placement_grid
                        .add_entity(position, ContentType::Tree, 3.0);

                    // Add entity to chunk
                    if let Some(chunk) = world.get_chunk_mut(coord) {
                        chunk.entities.push(tree_entity);
                    }
                }
            }
        }

        // Mark vegetation as generated
        if let Some(chunk) = world.get_chunk_mut(coord) {
            chunk.vegetation_generated = true;
        }
    }

    fn spawn_palm_tree(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<Entity, String> {
        // Create palm tree parent entity
        let palm_entity = commands
            .spawn((
                Transform::from_translation(position),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
                VisibilityRange::abrupt(0.0, 500.0), // Standardized 500m culling
                UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vegetation,
                },
            ))
            .id();

        // Simple trunk - needs own VisibilityRange (doesn't inherit in 0.16)
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.4, 0.25, 0.15))), // Brown trunk
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(palm_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Simple fronds - 4 green rectangles arranged in a cross
        for i in 0..4 {
            let angle = (i as f32) * std::f32::consts::PI / 2.0;

            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.5, 0.1, 0.8))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.25))), // Green fronds
                Transform::from_xyz(angle.cos() * 1.2, 7.5, angle.sin() * 1.2).with_rotation(
                    Quat::from_rotation_y(angle) * Quat::from_rotation_z(-0.2), // Slight droop
                ),
                ChildOf(palm_entity),
                VisibleChildBundle::default(),
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: 450.0..550.0,
                    use_aabb: false,
                },
            ));
        }

        // Simple physics collider for trunk - inherits visibility from parent
        commands.spawn((
            RigidBody::Fixed,
            Collider::cylinder(4.0, 0.3),
            CollisionGroups::new(STATIC_GROUP, Group::ALL),
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(palm_entity),
        ));

        Ok(palm_entity)
    }

    fn is_on_road(&self, position: Vec3, world: &UnifiedWorldManager) -> bool {
        for road in world.road_network.roads.values() {
            if self.is_point_on_road_spline(position, road, 15.0) {
                return true;
            }
        }
        false
    }

    fn is_point_on_road_spline(
        &self,
        position: Vec3,
        road: &crate::systems::world::road_network::RoadSpline,
        tolerance: f32,
    ) -> bool {
        let samples = 20;
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

    fn is_in_water_area(&self, position: Vec3) -> bool {
        let lake_center = Vec3::new(300.0, 0.0, 300.0);
        let lake_size = 200.0;
        let buffer = 20.0;

        let distance = Vec2::new(position.x - lake_center.x, position.z - lake_center.z).length();
        distance < (lake_size / 2.0 + buffer)
    }
}
