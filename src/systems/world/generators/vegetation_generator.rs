use crate::bundles::VisibleChildBundle;
use crate::components::ContentType;
use crate::components::unified_water::UnifiedWaterBody;
use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;
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
    #[allow(clippy::too_many_arguments)]
    pub fn generate_vegetation(
        &self,
        commands: &mut Commands,
        world: &mut UnifiedWorldManager,
        coord: ChunkCoord,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        world_rng: &mut WorldRng,
        water_bodies: &Query<&UnifiedWaterBody>,
        config: &GameConfig,
        env: &WorldEnvConfig,
    ) {
        let chunk_center = coord.to_world_pos();
        let half_size = world.chunk_size * 0.5;

        // Skip if chunk is not on a terrain island (including beach margin for beach vegetation)
        if !world.is_on_terrain_island_with_margin(chunk_center, env.terrain.beach_width) {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.vegetation_generated = true;
            }
            return;
        }

        // Determine vegetation density based on distance from center
        let distance_from_center = Vec2::new(chunk_center.x, chunk_center.z).length();
        let radius = env.max_world_coordinate.max(1.0);
        let vegetation_density = (1.0 - (distance_from_center / radius).min(0.7)).max(0.2);

        // Generate palm tree positions - scaled by density
        let tree_attempts = (vegetation_density * 5.0) as usize;
        let mut trees_spawned = 0;

        for _ in 0..tree_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(
                chunk_center.x + local_x,
                env.land_elevation,
                chunk_center.z + local_z,
            );

            // Check if position is valid (on beach band only, not on road, not overlapping, not in water)
            if self.is_on_beach_band(position, env)
                && !self.is_on_road(position, world)
                && !self.is_in_water_area(position, water_bodies)
                && world
                    .placement_grid
                    .can_place(position, ContentType::Tree, 3.0, 10.0)
            {
                if let Ok(tree_entity) =
                    self.spawn_palm_tree(commands, coord, position, meshes, materials, config)
                {
                    trees_spawned += 1;

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

        if trees_spawned > 0 {
            debug!(
                "Spawned {} palm trees in chunk ({}, {})",
                trees_spawned, coord.x, coord.z
            );
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
        config: &GameConfig,
    ) -> Result<Entity, String> {
        // Create palm tree parent entity
        let palm_entity = commands
            .spawn((
                Transform::from_translation(position),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
                // Removed parent VisibilityRange - let children control their own culling
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
                end_margin: (config.world_streaming.vegetation_cull_distance * 0.9)
                    ..(config.world_streaming.vegetation_cull_distance * 1.1),
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
                    end_margin: (config.world_streaming.vegetation_cull_distance * 0.9)
                        ..(config.world_streaming.vegetation_cull_distance * 1.1),
                    use_aabb: true, // Use AABB for accurate culling
                },
            ));
        }

        // Tree trunk collider from config
        let tree_config = &config.world_objects.palm_tree;
        commands.spawn((
            RigidBody::Fixed,
            tree_config.create_collider(),
            CollisionGroups::new(
                config.physics.static_group,
                config.physics.vehicle_group | config.physics.character_group,
            ),
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

    fn is_in_water_area(&self, position: Vec3, water_bodies: &Query<&UnifiedWaterBody>) -> bool {
        let buffer = 20.0; // Buffer zone around water to avoid palm trees
        let vertical_clearance = 1.0; // Trees must be this much above water

        // Check if position is in any water body
        for water_body in water_bodies.iter() {
            // Skip check if tree is above water surface (islands are elevated)
            let water_surface = water_body.get_water_surface_level(0.0);
            if position.y > water_surface + vertical_clearance {
                continue; // Tree is safely above water
            }

            if water_body.contains_point(position.x, position.z) {
                return true;
            }

            // Check buffer zone around water body
            let (min_x, min_z, max_x, max_z) = water_body.bounds;
            let expanded_bounds = (
                min_x - buffer,
                min_z - buffer,
                max_x + buffer,
                max_z + buffer,
            );

            if position.x >= expanded_bounds.0
                && position.x <= expanded_bounds.2
                && position.z >= expanded_bounds.1
                && position.z <= expanded_bounds.3
            {
                return true;
            }
        }
        false
    }

    /// Check if position is in beach band (0-100m outside terrain edge) for any island
    fn is_on_beach_band(&self, position: Vec3, env: &WorldEnvConfig) -> bool {
        self.is_in_island_beach_band(position, env.islands.left_x, 0.0, env)
            || self.is_in_island_beach_band(position, env.islands.right_x, 0.0, env)
            || self.is_in_island_beach_band(position, env.islands.grid_x, env.islands.grid_z, env)
    }

    /// Check if position is in beach band for a specific island
    fn is_in_island_beach_band(
        &self,
        position: Vec3,
        center_x: f32,
        center_z: f32,
        env: &WorldEnvConfig,
    ) -> bool {
        let dx = (position.x - center_x).abs();
        let dz = (position.z - center_z).abs();
        let half = env.terrain.half_size;

        // Side bands (outside terrain edge but within beach width)
        let on_side_band = ((dx > half && dx <= half + env.terrain.beach_width) && dz <= half)
            || ((dz > half && dz <= half + env.terrain.beach_width) && dx <= half);

        // Corner bands (diagonal corners using Chebyshev distance)
        let on_corner_band =
            (dx > half && dz > half) && ((dx - half).max(dz - half) <= env.terrain.beach_width);

        on_side_band || on_corner_band
    }
}
