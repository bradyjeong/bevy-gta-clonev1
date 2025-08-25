use crate::components::ContentType;
use crate::factories::VegetationFactory;
use crate::resources::WorldRng;
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UNIFIED_CHUNK_SIZE, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
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
        let half_size = UNIFIED_CHUNK_SIZE * 0.5;

        // Generate trees and vegetation in open areas
        let vegetation_attempts = 8;

        for _ in 0..vegetation_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);

            // Only spawn vegetation away from roads and buildings
            if !self.is_on_road(position, world) {
                if world.placement_grid.can_place(
                    position,
                    ContentType::Tree,
                    2.0, // Tree radius
                    8.0, // Minimum distance between trees
                ) {
                    if let Ok(tree_entity) =
                        self.spawn_tree(commands, coord, position, meshes, materials)
                    {
                        // Add to placement grid
                        world
                            .placement_grid
                            .add_entity(position, ContentType::Tree, 2.0);

                        // Add entity to chunk
                        if let Some(chunk) = world.get_chunk_mut(coord) {
                            chunk.entities.push(tree_entity);
                        }
                    }
                }
            }
        }

        // Mark vegetation as generated
        if let Some(chunk) = world.get_chunk_mut(coord) {
            chunk.vegetation_generated = true;
        }
    }

    fn spawn_tree(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<Entity, String> {
        // Use focused VegetationFactory for clean, single-responsibility design
        let factory = VegetationFactory::new();

        match factory.spawn_tree(commands, meshes, materials, position, None) {
            Ok(entity) => {
                // Add chunk-specific components to maintain compatibility
                commands.entity(entity).insert(UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vegetation,
                });
                Ok(entity)
            }
            Err(e) => Err(format!("Failed to spawn tree: {}", e)),
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
}
