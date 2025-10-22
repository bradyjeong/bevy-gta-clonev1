#![allow(deprecated)]

use crate::components::ContentType;
use crate::components::unified_water::UnifiedWaterBody;
use crate::config::GameConfig;
use crate::constants::LAND_ELEVATION;
use crate::factories::BuildingFactory;
use crate::resources::WorldRng;
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
use rand::Rng;

pub struct BuildingGenerator;

impl BuildingGenerator {
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
    ) {
        let chunk_center = coord.to_world_pos();
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
        let building_density = (1.0 - (distance_from_center / world_half_size).min(0.8)).max(0.1);

        // Generate building positions - reduced for simplicity
        let building_attempts = (building_density * 8.0) as usize;

        for _ in 0..building_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(
                chunk_center.x + local_x,
                LAND_ELEVATION,
                chunk_center.z + local_z,
            );

            // Check if position is valid (on island, not on road, not overlapping, not in water)
            if world.is_on_terrain_island(position)
                && !self.is_on_road(position, world)
                && !self.is_in_water_area(position, water_bodies)
            {
                let building_size = world_rng.global().gen_range(8.0..15.0);
                if world.placement_grid.can_place(
                    position,
                    ContentType::Building,
                    building_size * 0.5,
                    building_size,
                ) {
                    if let Ok(building_entity) =
                        self.spawn_building(commands, coord, position, meshes, materials)
                    {
                        // Add to placement grid
                        world.placement_grid.add_entity(
                            position,
                            ContentType::Building,
                            building_size * 0.5,
                        );

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

    fn spawn_building(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<Entity, String> {
        // Use focused BuildingFactory for clean, single-responsibility design
        let factory = BuildingFactory::new();

        match factory.spawn_building(commands, meshes, materials, position, None) {
            Ok(entity) => {
                // Add chunk-specific components to maintain compatibility
                commands.entity(entity).insert(UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Buildings,
                });
                Ok(entity)
            }
            Err(e) => Err(format!("Failed to spawn building: {e}")),
        }
    }

    fn is_on_road(&self, position: Vec3, world: &UnifiedWorldManager) -> bool {
        for road in world.road_network.roads.values() {
            if self.is_point_on_road_spline(position, road, 25.0) {
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
