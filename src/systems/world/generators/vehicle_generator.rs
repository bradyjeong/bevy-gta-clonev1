use crate::components::{ContentType, VehicleType};
use crate::config::GameConfig;
use crate::factories::VehicleFactory;
use crate::resources::WorldRng;
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
use rand::Rng;

pub struct VehicleGenerator;

impl VehicleGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn generate_vehicles(
        &self,
        commands: &mut Commands,
        world: &mut UnifiedWorldManager,
        road_network: &RoadNetwork,
        coord: ChunkCoord,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        world_rng: &mut WorldRng,
        config: &GameConfig,
    ) {
        let chunk_center = coord.to_world_pos_with_size(world.chunk_size);
        let half_size = world.chunk_size * 0.5;

        // Skip vehicle generation for chunks near world edge
        let world_half_size = config.world_bounds.vehicle_spawn_half_size;
        let edge_buffer = config.world_bounds.edge_buffer;
        if chunk_center.x.abs() > world_half_size - edge_buffer
            || chunk_center.z.abs() > world_half_size - edge_buffer
        {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.vehicles_generated = true;
            }
            return;
        }

        // Skip if chunk is not on a terrain island
        if !world.is_on_terrain_island(chunk_center) {
            if let Some(chunk) = world.get_chunk_mut(coord) {
                chunk.vehicles_generated = true;
            }
            return;
        }

        // Generate road vehicles
        let vehicle_attempts = 8;
        for _ in 0..vehicle_attempts {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(
                chunk_center.x + local_x,
                config.world_env.land_elevation,
                chunk_center.z + local_z,
            );

            // Only spawn on roads (which are on islands) with sufficient spacing
            if world.is_on_terrain_island(position)
                && self.is_on_road(position, road_network)
                && world
                    .placement_grid
                    .can_place(position, ContentType::Vehicle, 4.0, 15.0)
            {
                if let Ok(vehicle_entity) = self.spawn_ground_vehicle(
                    commands,
                    coord,
                    position,
                    meshes,
                    materials,
                    asset_server,
                    world_rng,
                    config,
                ) {
                    world
                        .placement_grid
                        .add_entity(position, ContentType::Vehicle, 4.0);

                    if let Some(chunk) = world.get_chunk_mut(coord) {
                        chunk.entities.push(vehicle_entity);
                    }
                }
            }
        }

        // Generate aircraft in open areas (2% chance per chunk)
        if world_rng.global().gen_bool(0.02) {
            let local_x = world_rng.global().gen_range(-half_size..half_size);
            let local_z = world_rng.global().gen_range(-half_size..half_size);
            let position = Vec3::new(
                chunk_center.x + local_x,
                config.world_env.land_elevation + config.world_env.spawn_drop_height,
                chunk_center.z + local_z,
            );

            // Spawn in open areas away from roads
            if !self.is_on_road(position, road_network)
                && world.placement_grid.can_place(
                    position,
                    ContentType::Vehicle,
                    10.0, // Larger radius for aircraft
                    50.0, // More spacing
                )
            {
                if let Ok(aircraft_entity) = self.spawn_aircraft(
                    commands,
                    coord,
                    position,
                    meshes,
                    materials,
                    asset_server,
                    world_rng,
                    config,
                ) {
                    world
                        .placement_grid
                        .add_entity(position, ContentType::Vehicle, 10.0);

                    if let Some(chunk) = world.get_chunk_mut(coord) {
                        chunk.entities.push(aircraft_entity);
                    }
                }
            }
        }

        // Mark vehicles as generated
        if let Some(chunk) = world.get_chunk_mut(coord) {
            chunk.vehicles_generated = true;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_ground_vehicle(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        world_rng: &mut WorldRng,
        config: &GameConfig,
    ) -> Result<Entity, String> {
        let factory = VehicleFactory::with_config(config.clone());
        let vehicle_types = [VehicleType::SuperCar];
        let vehicle_type = vehicle_types[world_rng.global().gen_range(0..vehicle_types.len())];

        match factory.spawn_vehicle_by_type(
            commands,
            meshes,
            materials,
            asset_server,
            vehicle_type,
            position,
            None,
        ) {
            Ok(entity) => {
                commands.entity(entity).insert(UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vehicles,
                });
                Ok(entity)
            }
            Err(e) => Err(format!("Failed to spawn ground vehicle: {e}")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_aircraft(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        position: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        world_rng: &mut WorldRng,
        config: &GameConfig,
    ) -> Result<Entity, String> {
        let factory = VehicleFactory::with_config(config.clone());
        let aircraft_types = [VehicleType::Helicopter, VehicleType::F16];
        let vehicle_type = aircraft_types[world_rng.global().gen_range(0..aircraft_types.len())];

        match factory.spawn_vehicle_by_type(
            commands,
            meshes,
            materials,
            asset_server,
            vehicle_type,
            position,
            None,
        ) {
            Ok(entity) => {
                commands.entity(entity).insert(UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vehicles,
                });
                Ok(entity)
            }
            Err(e) => Err(format!("Failed to spawn aircraft: {e}")),
        }
    }

    fn is_on_road(&self, position: Vec3, road_network: &RoadNetwork) -> bool {
        for road in road_network.roads.values() {
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
