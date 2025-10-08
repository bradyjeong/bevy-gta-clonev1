use crate::bundles::VisibleChildBundle;
use crate::components::{ContentType, DynamicContent, IntersectionEntity, RoadEntity};
use crate::resources::{MaterialKey, MaterialRegistry, WorldRng};
use crate::systems::world::road_mesh::{generate_road_markings_mesh_local, generate_road_mesh_local};
use crate::systems::world::road_network::{IntersectionType, RoadSpline, RoadType};
use crate::systems::world::unified_world::{
    ChunkCoord, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
};
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;

pub struct RoadGenerator;

impl RoadGenerator {
    #[allow(clippy::too_many_arguments, deprecated)]
    pub fn generate_roads(
        &self,
        commands: &mut Commands,
        world: &mut UnifiedWorldManager,
        coord: ChunkCoord,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        material_registry: &mut MaterialRegistry,
        _world_rng: &mut WorldRng,
    ) {
        let new_road_ids = world.road_network.generate_chunk_roads(coord.x, coord.z);

        // Create road entities and add to placement grid
        for road_id in new_road_ids {
            if let Some(road) = world.road_network.roads.get(&road_id).cloned() {
                // Skip roads that pass through water areas
                if self.road_intersects_water(&road) {
                    continue;
                }
                let road_entity = self.spawn_road_entity(
                    commands,
                    coord,
                    road_id,
                    &road,
                    meshes,
                    materials,
                    material_registry,
                );

                // Add road to placement grid
                let samples = 20;
                for i in 0..samples {
                    let t = i as f32 / (samples - 1) as f32;
                    let road_point = road.evaluate(t);
                    world.placement_grid.add_entity(
                        road_point,
                        ContentType::Road,
                        road.road_type.width() * 0.5,
                    );
                }

                // Add entity to chunk
                if let Some(chunk) = world.get_chunk_mut(coord) {
                    chunk.entities.push(road_entity);
                }
            }
        }

        // Detect and create intersections
        self.detect_and_spawn_intersections(commands, world, coord);

        // Mark roads as generated
        if let Some(chunk) = world.get_chunk_mut(coord) {
            chunk.roads_generated = true;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_road_entity(
        &self,
        commands: &mut Commands,
        chunk_coord: ChunkCoord,
        road_id: u64,
        road: &RoadSpline,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        material_registry: &mut MaterialRegistry,
    ) -> Entity {
        let center_pos = road.evaluate(0.5);

        let road_material =
            self.create_road_material(&road.road_type, materials, material_registry);
        let marking_material = self.create_marking_material(materials, material_registry);

        let road_entity = commands
            .spawn((
                UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Roads,
                },
                RoadEntity { road_id },
                Transform::from_translation(center_pos),
                Visibility::default(),
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: 450.0..550.0,
                    use_aabb: false,
                },
                DynamicContent {
                    content_type: ContentType::Road,
                },
            ))
            .id();

        // Road surface mesh - local coordinates with proper VisibilityRange
        let road_mesh = generate_road_mesh_local(road, center_pos);
        commands.spawn((
            Mesh3d(meshes.add(road_mesh)),
            MeshMaterial3d(road_material),
            Transform::from_translation(Vec3::new(0.0, 0.05, 0.0)),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Road markings - local coordinates with proper VisibilityRange
        let marking_meshes = generate_road_markings_mesh_local(road, center_pos);
        for marking_mesh in marking_meshes {
            commands.spawn((
                Mesh3d(meshes.add(marking_mesh)),
                MeshMaterial3d(marking_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.06, 0.0)),
                ChildOf(road_entity),
                VisibleChildBundle::default(),
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: 450.0..550.0,
                    use_aabb: false,
                },
            ));
        }

        road_entity
    }

    fn detect_and_spawn_intersections(
        &self,
        commands: &mut Commands,
        world: &mut UnifiedWorldManager,
        coord: ChunkCoord,
    ) {
        let chunk_center = coord.to_world_pos();
        let chunk_size = world.chunk_size;
        let half_size = chunk_size * 0.5;

        // Collect all roads in and around this chunk
        let mut chunk_roads = Vec::new();
        for (road_id, road) in &world.road_network.roads {
            let road_center = road.evaluate(0.5);
            if (road_center.x - chunk_center.x).abs() < chunk_size
                && (road_center.z - chunk_center.z).abs() < chunk_size
            {
                chunk_roads.push((*road_id, road.clone()));
            }
        }

        // Find intersections between roads
        let mut detected_intersections = Vec::new();
        for i in 0..chunk_roads.len() {
            for j in (i + 1)..chunk_roads.len() {
                let (road1_id, road1) = &chunk_roads[i];
                let (road2_id, road2) = &chunk_roads[j];

                if let Some(intersection_point) = self.find_road_intersection(road1, road2) {
                    if intersection_point.x >= chunk_center.x - half_size
                        && intersection_point.x <= chunk_center.x + half_size
                        && intersection_point.z >= chunk_center.z - half_size
                        && intersection_point.z <= chunk_center.z + half_size
                    {
                        let intersection_type = IntersectionType::Cross;

                        detected_intersections.push((
                            intersection_point,
                            vec![*road1_id, *road2_id],
                            intersection_type,
                        ));
                    }
                }
            }
        }

        // Create intersection entities
        for (position, connected_roads, intersection_type) in detected_intersections {
            let intersection_id =
                world
                    .road_network
                    .add_intersection(position, connected_roads, intersection_type);

            if world
                .road_network
                .intersections
                .contains_key(&intersection_id)
            {
                let intersection_entity = commands
                    .spawn((
                        UnifiedChunkEntity {
                            coord,
                            layer: ContentLayer::Roads,
                        },
                        IntersectionEntity { intersection_id },
                        Transform::from_translation(position),
                        Visibility::default(),
                        InheritedVisibility::VISIBLE,
                        ViewVisibility::default(),
                        DynamicContent {
                            content_type: ContentType::Road,
                        },
                    ))
                    .id();

                // Add entity to chunk
                if let Some(chunk) = world.get_chunk_mut(coord) {
                    chunk.entities.push(intersection_entity);
                }
            }
        }
    }

    fn find_road_intersection(&self, road1: &RoadSpline, road2: &RoadSpline) -> Option<Vec3> {
        let samples = 20;
        let intersection_threshold = 3.0;

        for i in 0..samples {
            let t1 = i as f32 / (samples - 1) as f32;
            let point1 = road1.evaluate(t1);

            for j in 0..samples {
                let t2 = j as f32 / (samples - 1) as f32;
                let point2 = road2.evaluate(t2);

                let distance = Vec3::new(point1.x - point2.x, 0.0, point1.z - point2.z).length();
                if distance < intersection_threshold {
                    return Some(Vec3::new(
                        (point1.x + point2.x) * 0.5,
                        0.0,
                        (point1.z + point2.z) * 0.5,
                    ));
                }
            }
        }

        None
    }

    /// Check if road intersects water area
    fn road_intersects_water(&self, road: &RoadSpline) -> bool {
        let lake_center = Vec3::new(300.0, 0.0, 300.0);
        let lake_size = 200.0;
        let buffer = 10.0; // Smaller buffer for roads

        // Sample road spline at multiple points
        let samples = 10;
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let road_point = road.evaluate(t);
            let distance =
                Vec2::new(road_point.x - lake_center.x, road_point.z - lake_center.z).length();

            if distance < (lake_size / 2.0 + buffer) {
                return true;
            }
        }
        false
    }

    fn create_road_material(
        &self,
        road_type: &RoadType,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        material_registry: &mut MaterialRegistry,
    ) -> Handle<StandardMaterial> {
        let (base_color, roughness) = match road_type {
            RoadType::Highway => (Color::srgb(0.4, 0.4, 0.45), 0.8),
            RoadType::MainStreet => (Color::srgb(0.35, 0.35, 0.4), 0.8),
            RoadType::SideStreet => (Color::srgb(0.45, 0.45, 0.5), 0.7),
            RoadType::Alley => (Color::srgb(0.5, 0.5, 0.45), 0.6),
        };

        // Use registry for performance (no depth_bias needed with physical offset)
        let key = MaterialKey::road(base_color).with_roughness(roughness);
        material_registry.get_or_create(materials, key)
    }

    fn create_marking_material(
        &self,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        material_registry: &mut MaterialRegistry,
    ) -> Handle<StandardMaterial> {
        let color = Color::srgb(0.95, 0.95, 0.95);

        // Use registry for performance (no depth_bias needed with physical offset)
        let key = MaterialKey::road_marking(color).with_roughness(0.6);
        material_registry.get_or_create(materials, key)
    }
}
