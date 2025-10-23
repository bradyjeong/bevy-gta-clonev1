use crate::components::ContentType;
use crate::components::unified_water::UnifiedWaterBody;
use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;
use crate::factories::{BuildingFactory, BuildingType};
use crate::resources::WorldRng;
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
use rand::Rng;

pub struct BuildingGenerator;

impl BuildingGenerator {
    fn choose_manhattan_building(
        &self,
        grid_center: Vec3,
        position: Vec3,
        rng: &mut impl rand::Rng,
    ) -> (f32, f32, f32, Color) {
        let distance_from_center =
            Vec2::new(position.x - grid_center.x, position.z - grid_center.z).length();

        let normalized_distance = (distance_from_center / 900.0).min(1.0);

        let roll: f32 = rng.gen_range(0.0..1.0);

        // Supertall (glass 80%, stone 20%)
        if roll < 0.05 * (1.0 - normalized_distance * 0.8) {
            let color_roll: f32 = rng.gen_range(0.0..1.0);
            let color = if color_roll < 0.8 {
                // Glass/steel
                Color::srgb(
                    rng.gen_range(0.15..0.3),
                    rng.gen_range(0.2..0.35),
                    rng.gen_range(0.35..0.55),
                )
            } else {
                // Dark stone
                Color::srgb(
                    rng.gen_range(0.4..0.6),
                    rng.gen_range(0.4..0.55),
                    rng.gen_range(0.38..0.52),
                )
            };

            (
                rng.gen_range(25.0..44.0), // Max 44m to ensure conservative margin
                rng.gen_range(25.0..51.0), // Max 51m: guaranteed fit in 80m blocks with 5m margins
                rng.gen_range(200.0..320.0),
                color,
            )
        }
        // High-rise (glass 60%, stone 30%, brick 10%)
        else if roll < 0.35 * (1.0 - normalized_distance * 0.5) {
            let color_roll: f32 = rng.gen_range(0.0..1.0);
            let color = if color_roll < 0.6 {
                // Glass/steel
                Color::srgb(
                    rng.gen_range(0.15..0.3),
                    rng.gen_range(0.2..0.35),
                    rng.gen_range(0.35..0.55),
                )
            } else if color_roll < 0.9 {
                // Stone
                Color::srgb(
                    rng.gen_range(0.4..0.6),
                    rng.gen_range(0.4..0.55),
                    rng.gen_range(0.38..0.52),
                )
            } else {
                // Brick
                Color::srgb(
                    rng.gen_range(0.45..0.6),
                    rng.gen_range(0.25..0.4),
                    rng.gen_range(0.2..0.3),
                )
            };

            (
                rng.gen_range(18.0..34.0), // Max 34m to ensure conservative margin
                rng.gen_range(18.0..39.0), // Max 39m to ensure conservative margin
                rng.gen_range(80.0..180.0),
                color,
            )
        }
        // Mid-rise (brick 50%, stone 40%, glass 10%)
        else if roll < 0.85 {
            let color_roll: f32 = rng.gen_range(0.0..1.0);
            let color = if color_roll < 0.5 {
                // Brick
                Color::srgb(
                    rng.gen_range(0.45..0.6),
                    rng.gen_range(0.25..0.4),
                    rng.gen_range(0.2..0.3),
                )
            } else if color_roll < 0.9 {
                // Stone
                Color::srgb(
                    rng.gen_range(0.4..0.6),
                    rng.gen_range(0.4..0.55),
                    rng.gen_range(0.38..0.52),
                )
            } else {
                // Glass
                Color::srgb(
                    rng.gen_range(0.15..0.3),
                    rng.gen_range(0.2..0.35),
                    rng.gen_range(0.35..0.55),
                )
            };

            (
                rng.gen_range(12.0..25.0),
                rng.gen_range(15.0..30.0),
                rng.gen_range(25.0..60.0),
                color,
            )
        }
        // Brownstone (brick 70%, stone 30%, no glass)
        else {
            let color_roll: f32 = rng.gen_range(0.0..1.0);
            let color = if color_roll < 0.7 {
                // Brick
                Color::srgb(
                    rng.gen_range(0.45..0.6),
                    rng.gen_range(0.25..0.4),
                    rng.gen_range(0.2..0.3),
                )
            } else {
                // Stone
                Color::srgb(
                    rng.gen_range(0.4..0.6),
                    rng.gen_range(0.4..0.55),
                    rng.gen_range(0.38..0.52),
                )
            };

            (
                rng.gen_range(8.0..12.0),
                rng.gen_range(12.0..20.0),
                rng.gen_range(12.0..20.0),
                color,
            )
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn generate_buildings(
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
        let chunk_center = coord.to_world_pos_with_size(world.chunk_size);
        let half_size = world.chunk_size * 0.5;

        // Skip building generation for chunks near world edge
        let world_half_size = config.world_bounds.world_half_size;
        let edge_buffer = config.world_bounds.edge_buffer;
        if chunk_center.x.abs() > world_half_size - edge_buffer
            || chunk_center.z.abs() > world_half_size - edge_buffer
        {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.buildings_generated = true;
            }
            return;
        }

        // Skip if chunk is not on a terrain island
        if !world.is_on_terrain_island(chunk_center) {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.buildings_generated = true;
            }
            return;
        }

        // Determine building density based on distance from center
        let distance_from_center = Vec2::new(chunk_center.x, chunk_center.z).length();
        let mut building_density =
            (1.0 - (distance_from_center / world_half_size).min(0.8)).max(0.1);

        // Override density for Grid Island (Manhattan-style urban density)
        if world.is_on_grid_island(chunk_center) {
            building_density = 1.0;
        }

        // Generate building positions - reduced for simplicity
        let building_attempts = if world.is_on_grid_island(chunk_center) {
            60 // Dense Manhattan packing
        } else {
            (building_density * 8.0) as usize
        };

        // Calculate grid center from environment config for Manhattan building selection
        let grid_center = Vec3::new(env.islands.grid_x, 0.0, env.islands.grid_z);

        for _ in 0..building_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(
                chunk_center.x + local_x,
                env.land_elevation,
                chunk_center.z + local_z,
            );

            // Check on_grid per position
            let on_grid = world.is_on_grid_island(position);

            let (footprint_x, footprint_z, height, color, radius) = if on_grid {
                let (fx, fz, h, c) =
                    self.choose_manhattan_building(grid_center, position, world_rng.global());
                let r = fx.max(fz) * 0.5;
                (fx, fz, h, Some(c), r)
            } else {
                let size = world_rng.global().gen_range(8.0..15.0);
                let h = world_rng.global().gen_range(8.0..30.0);
                (size, size, h, None, size * 0.5)
            };

            // Check if position is valid (on island, not on road with radius, not overlapping, not in water)
            if world.is_on_terrain_island(position)
                && !self.is_on_road(position, world, radius)
                && !self.is_in_water_area(position, water_bodies)
            {
                let min_distance = if on_grid {
                    1.0 // True zero-lot-line Manhattan (reduced from 2.0)
                } else {
                    footprint_x.max(footprint_z) // Suburban spacing
                };

                if world.placement_grid.can_place(
                    position,
                    ContentType::Building,
                    radius,
                    min_distance,
                ) {
                    if let Ok(building_entity) = self.spawn_building(
                        commands,
                        coord,
                        position,
                        Vec3::new(footprint_x, height, footprint_z),
                        color,
                        meshes,
                        materials,
                        config,
                    ) {
                        // Add to placement grid
                        world
                            .placement_grid
                            .add_entity(position, ContentType::Building, radius);

                        // Add entity to chunk
                        if let Some(chunk) = world.get_chunk_mut(coord) {
                            chunk.entities.push(building_entity);
                        }
                    }
                }
            }
        }

        // Mark buildings as generated
        if let Some(chunk) = world.get_chunk_mut(coord) {
            chunk.buildings_generated = true;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_building(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        size: Vec3,
        color: Option<Color>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        config: &GameConfig,
    ) -> Result<Entity, String> {
        let factory = BuildingFactory::with_config(config.clone());

        let entity = if let Some(c) = color {
            factory.spawn_building_with_size(
                commands,
                meshes,
                materials,
                position,
                size,
                BuildingType::Commercial,
                c,
            )
        } else {
            factory.spawn_building(commands, meshes, materials, position, None)
        };

        match entity {
            Ok(entity) => {
                commands.entity(entity).insert(UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Buildings,
                });
                Ok(entity)
            }
            Err(e) => Err(format!("Failed to spawn building: {e}")),
        }
    }

    fn is_on_road(&self, position: Vec3, world: &UnifiedWorldManager, radius: f32) -> bool {
        let tolerance = if world.is_on_grid_island(position) {
            5.0 // Conservative margin - buildings NEVER touch roads
        } else {
            25.0 // Suburban spacing elsewhere
        };

        for road in world.road_network.roads.values() {
            if self.is_point_on_road_spline(position, road, tolerance, radius) {
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
        radius: f32,
    ) -> bool {
        let width = road.road_type.width();

        // For straight roads (2 control points), use exact point-to-segment distance
        if road.control_points.len() == 2 {
            let a = road.control_points[0];
            let b = road.control_points[road.control_points.len() - 1];

            // Project point onto line segment
            let vx = b.x - a.x;
            let vz = b.z - a.z;
            let wx = position.x - a.x;
            let wz = position.z - a.z;

            let len_sq = vx * vx + vz * vz;
            let t = if len_sq > 0.0 {
                ((wx * vx + wz * vz) / len_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };

            // Closest point on segment
            let closest_x = a.x + t * vx;
            let closest_z = a.z + t * vz;

            // Distance from position to closest point
            let dx = position.x - closest_x;
            let dz = position.z - closest_z;
            let distance = (dx * dx + dz * dz).sqrt();

            return distance <= width * 0.5 + tolerance + radius;
        }

        // For curved roads, fall back to sampling
        let samples = 20;
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let road_point = road.evaluate(t);
            let distance =
                Vec3::new(position.x - road_point.x, 0.0, position.z - road_point.z).length();

            // Include building radius in overlap check
            if distance <= width * 0.5 + tolerance + radius {
                return true;
            }
        }

        false
    }

    /// Check if position is in water area (3D check - buildings above water are OK)
    fn is_in_water_area(&self, position: Vec3, water_bodies: &Query<&UnifiedWaterBody>) -> bool {
        let buffer = 20.0; // Buffer zone around water
        let vertical_clearance = 1.0; // Buildings must be this much above water

        // Check if position is in any water body
        for water_body in water_bodies.iter() {
            // Skip check if building is above water surface (islands are elevated)
            let water_surface = water_body.get_water_surface_level(0.0);
            if position.y > water_surface + vertical_clearance {
                continue; // Building is safely above water
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
}
