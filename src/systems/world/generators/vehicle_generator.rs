use crate::components::{ContentType, VehicleType};
use crate::factories::VehicleFactory;
use crate::resources::WorldRng;
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
use rand::Rng;

pub struct VehicleGenerator;

impl VehicleGenerator {
    pub fn generate_vehicles(
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

        // Skip vehicle generation for chunks near world edge (±2000m)
        const WORLD_HALF_SIZE: f32 = 2000.0;
        const EDGE_BUFFER: f32 = 200.0;
        if chunk_center.x.abs() > WORLD_HALF_SIZE - EDGE_BUFFER
            || chunk_center.z.abs() > WORLD_HALF_SIZE - EDGE_BUFFER
        {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.vehicles_generated = true;
            }
            return;
        }

        // Generate vehicles only on roads - increased for more traffic
        let vehicle_attempts = 8;

        for _ in 0..vehicle_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);

            // Only spawn on roads with sufficient spacing
            if self.is_on_road(position, world)
                && world.placement_grid.can_place(
                    position,
                    ContentType::Vehicle,
                    4.0,  // Vehicle radius
                    15.0, // Minimum distance between vehicles (reduced for more traffic)
                )
            {
                if let Ok(vehicle_entity) =
                    self.spawn_vehicle(commands, coord, position, meshes, materials, world_rng)
                {
                    // Add to placement grid
                    world
                        .placement_grid
                        .add_entity(position, ContentType::Vehicle, 4.0);

                    // Add entity to chunk
                    if let Some(chunk) = world.get_chunk_mut(coord) {
                        chunk.entities.push(vehicle_entity);
                    }
                }
            }
        }

        // Mark vehicles as generated
        if let Some(chunk) = world.get_chunk_mut(coord) {
            chunk.vehicles_generated = true;
        }
    }

    fn spawn_vehicle(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        world_rng: &mut WorldRng,
    ) -> Result<Entity, String> {
        // Use focused VehicleFactory for clean, single-responsibility design
        let factory = VehicleFactory::new();
        let vehicle_types = [
            VehicleType::SuperCar,
            VehicleType::Helicopter,
            VehicleType::F16,
        ];
        let vehicle_type = vehicle_types[world_rng.global().gen_range(0..vehicle_types.len())];

        match factory.spawn_vehicle_by_type(
            commands,
            meshes,
            materials,
            vehicle_type,
            position,
            None,
        ) {
            Ok(entity) => {
                // Add chunk-specific components to maintain compatibility
                commands.entity(entity).insert(UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vehicles,
                });
                Ok(entity)
            }
            Err(e) => Err(format!("Failed to spawn vehicle: {e}")),
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
