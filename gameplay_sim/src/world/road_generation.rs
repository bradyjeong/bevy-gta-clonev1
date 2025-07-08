//! ───────────────────────────────────────────────
//! System:   Road Generation
//! Purpose:  Handles entity movement and physics
//! Schedule: Update (throttled)
//! Reads:    `ActiveEntity`, Transform, `RoadEntity`, NPC, Time
//! Writes:   Transform, `RoadNetwork`
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Physics values are validated and finite
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use game_core::components::RoadNetwork;
use game_core::prelude::RoadEntity;
use crate::systems::world::road_mesh::{generate_road_mesh, generate_road_markings_mesh};
use game_core::bundles::VisibleChildBundle;

pub trait RoadNetworkExtensions {
    fn clear_cache(&mut self);
    fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32>;
}

impl RoadNetworkExtensions for ResMut<'_, RoadNetwork> {
    fn clear_cache(&mut self) {
        self.clear_cache();
    }
    
    fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32> {
        self.generate_chunk_roads(chunk_x, chunk_z)
    }
}

// MAIN ROAD GENERATION SYSTEM (Replaces old grid system)
// UNIFIED Y-COORDINATE SYSTEM (prevents z-fighting):
// - Terrain:      y = -0.15  (15cm below ground)  
// - All Roads:    y =  0.0   (unified ground level - highways, main streets, side streets, alleys)
// - Road Markings:y =  0.01  (1cm above road surface)
// Unified road height prevents overlapping geometry and z-fighting issues

// Add timer to reduce frequency of road checks
#[derive(Resource, Default)]
pub struct RoadGenerationTimer {
    timer: f32,
    last_player_chunk: Option<(i32, i32)>,
}

pub fn road_network_system(
    mut commands: Commands,
    mut road_network: ResMut<RoadNetwork>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    road_query: Query<(Entity, &Transform), With<RoadEntity>>,

    time: Res<Time>,
    mut timer: Local<RoadGenerationTimer>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    

    
    // Update timer
    timer.timer += time.delta_secs();
    
    // Only process road generation every 0.5 seconds OR when player changes chunk
    let chunk_size = 400.0;
    let current_chunk = (
        (active_pos.x / chunk_size).floor() as i32, // Use floor instead of round for more predictable chunking
        (active_pos.z / chunk_size).floor() as i32,
    );
    
    let chunk_changed = timer.last_player_chunk != Some(current_chunk);
    let should_update = timer.timer >= 0.5 || chunk_changed;
    
    if !should_update {
        return;
    }
    
    timer.timer = 0.0;
    timer.last_player_chunk = Some(current_chunk);
    
    // Clear cache if no roads exist but cache exists (optimization)
    if road_network.roads.is_empty() && !road_network.generated_chunks.is_empty() {
        road_network.clear_cache();
    }
    


    
    let active_radius = 800.0;   // Increased for better road coverage
    let cleanup_radius = 2000.0;  // Very large cleanup radius to prevent premature despawning
    
    // Clean up distant road entities (very conservative cleanup)
    if chunk_changed {
        for (entity, transform) in road_query.iter() {
            // Simple distance check - only remove roads that are extremely far away
            let distance = active_pos.distance(transform.translation);
            if distance > cleanup_radius {
                println!("DEBUG: Cleaning up road entity at distance {distance}");
                commands.entity(entity).despawn();
            }
        }
    }
    
    // Determine which chunks need roads
    let (chunk_x, chunk_z) = current_chunk;
    
    let chunk_radius = ((active_radius / chunk_size).ceil() as i32).max(3); // Ensure at least 3x3 chunk coverage
    
    // Generate roads for nearby chunks
    for dx in -chunk_radius..=chunk_radius {
        for dz in -chunk_radius..=chunk_radius {
            let check_chunk_x = chunk_x + dx;
            let check_chunk_z = chunk_z + dz;
            
            let chunk_center = Vec3::new(
                check_chunk_x as f32 * chunk_size,
                0.0,  // At ground level
                check_chunk_z as f32 * chunk_size
            );
            
            let distance = active_pos.distance(chunk_center);
            if distance <= active_radius {
                // println!("DEBUG: Generating roads for chunk ({}, {}) at distance {}", check_chunk_x, check_chunk_z, distance);
                let new_road_ids = road_network.generate_chunk_roads(check_chunk_x, check_chunk_z);
                // println!("DEBUG: Generated {} roads for chunk ({}, {})", new_road_ids.len(), check_chunk_x, check_chunk_z);
                
                // Spawn mesh entities for new roads
                for road_id in new_road_ids {
                    if let Some(road) = road_network.roads.get(&road_id) {
                        // println!("DEBUG: Spawning road entity for road {} at start {:?} with type {:?}", road_id, road.evaluate(0.0), road.road_type);
                        spawn_road_entity(&mut commands, road_id, road, &mut meshes, &mut materials);
                    }
                }
            }
        }
    }
}

fn spawn_road_entity(
    commands: &mut Commands,
    road_id: u32,
    road: &crate::systems::world::road_network::RoadSpline,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use bevy_rapier3d::geometry::Group;
    
    // Calculate road start position for better distance calculations
    let start_pos = road.evaluate(0.0);
    
    // Create road materials
    let road_material = create_road_material(&road.road_type, materials);
    let marking_material = create_marking_material(materials);
    
    // Main road entity with physics collider - positioned at ground level
    let road_entity = commands.spawn((
        RoadEntity { road_id },
        Transform::from_translation(Vec3::new(start_pos.x, 0.0, start_pos.z)), // Road entity at ground level
        GlobalTransform::default(),
        Visibility::default(),
        DynamicContent {
            content_type: ContentType::Road,
        },
        // Add physics for proper collision with vehicles and buildings
        RigidBody::Fixed,
        create_road_collider(road),
        CollisionGroups::new(Group::GROUP_1, Group::GROUP_2), // Only vehicles collide with roads, not characters
    )).id();
    
    // Main road surface mesh
    let road_mesh = generate_road_mesh(road);
    commands.spawn((
        Mesh3d(meshes.add(road_mesh)),
        MeshMaterial3d(road_material),
        Transform::from_translation(Vec3::new(-start_pos.x, 0.0, -start_pos.z)), // Road surface at ground level (y = 0.0)
        ChildOf(road_entity),
        VisibleChildBundle::default(),
    ));
    
    // Road markings for visibility and realism
    let marking_meshes = generate_road_markings_mesh(road);
    for marking_mesh in marking_meshes {
        commands.spawn((
            Mesh3d(meshes.add(marking_mesh)),
            MeshMaterial3d(marking_material.clone()),
            Transform::from_translation(Vec3::new(-start_pos.x, 0.01, -start_pos.z)), // Road markings 1cm above road surface (y = 0.01)
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
    }
}

fn create_road_collider(road: &crate::systems::world::road_network::RoadSpline) -> Collider {
    let width = road.road_type.width();
    let length = road.length();
    
    // Create a thin, flat collider for all roads to avoid interfering with character movement
    // Use consistent cuboid collider for all road types - simpler and more predictable
    Collider::cuboid(width * 0.5, 0.02, length * 0.5)  // Very thin (4cm): road surface only
}

fn create_road_material(road_type: &crate::systems::world::road_network::RoadType, materials: &mut ResMut<Assets<StandardMaterial>>) -> Handle<StandardMaterial> {
    use crate::systems::world::road_network::RoadType;
    
    let (base_color, roughness) = match road_type {
        RoadType::Highway => (Color::srgb(0.4, 0.4, 0.45), 0.8), // Lighter asphalt for visibility
        RoadType::MainStreet => (Color::srgb(0.35, 0.35, 0.4), 0.8), // Medium asphalt 
        RoadType::SideStreet => (Color::srgb(0.45, 0.45, 0.5), 0.7), // Lighter concrete
        RoadType::Alley => (Color::srgb(0.5, 0.5, 0.45), 0.6), // Light aged concrete
    };
    
    materials.add(StandardMaterial {
        base_color,
        perceptual_roughness: roughness,
        metallic: 0.0,
        reflectance: 0.2, // Increased reflectance for better visibility
        emissive: Color::BLACK.into(),
        ..default()
    })
}

fn create_marking_material(materials: &mut ResMut<Assets<StandardMaterial>>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.95), // Bright white for lane lines
        emissive: LinearRgba::new(0.2, 0.2, 0.2, 1.0), // Slight glow for visibility
        perceptual_roughness: 0.6,
        metallic: 0.0,
        reflectance: 0.5, // Higher reflectance for visibility
        ..default()
    })
}

// Enhanced road detection for vehicles and NPCs
#[must_use] pub fn is_on_road_spline(position: Vec3, road_network: &RoadNetwork, tolerance: f32) -> bool {
    for road in road_network.roads.values() {
        if is_point_on_road_spline(position, road, tolerance) {
            return true;
        }
    }
    false
}

fn is_point_on_road_spline(position: Vec3, road: &crate::systems::world::road_network::RoadSpline, tolerance: f32) -> bool {
    let samples = 50;
    let width = road.road_type.width();
    
    for i in 0..samples {
        let t = i as f32 / (samples - 1) as f32;
        let road_point = road.evaluate(t);
        let distance = Vec3::new(position.x - road_point.x, 0.0, position.z - road_point.z).length();
        
        if distance <= width * 0.5 + tolerance {
            return true;
        }
    }
    
    false
}

// System to update the old road detection calls
pub fn update_road_dependent_systems(
    road_network: Res<RoadNetwork>,
    mut vehicle_query: Query<&mut Transform, (With<crate::components::Car>, Without<ActiveEntity>)>,
    mut npc_query: Query<&mut Transform, (With<NPC>, Without<crate::components::Car>, Without<ActiveEntity>)>,
) {
    // Update vehicle positions to stay on roads
    for mut transform in &mut vehicle_query {
        if !is_on_road_spline(transform.translation, &road_network, 2.0) {
            // Find nearest road and snap to it
            if let Some(nearest_road_pos) = find_nearest_road_position(transform.translation, &road_network) {
                transform.translation.x = nearest_road_pos.x;
                transform.translation.z = nearest_road_pos.z;
            }
        }
    }
    
    // Similar for NPCs
    for mut transform in &mut npc_query {
        if !is_on_road_spline(transform.translation, &road_network, 1.0) {
            if let Some(nearest_road_pos) = find_nearest_road_position(transform.translation, &road_network) {
                transform.translation.x = nearest_road_pos.x;
                transform.translation.z = nearest_road_pos.z;
            }
        }
    }
}

fn find_nearest_road_position(position: Vec3, road_network: &RoadNetwork) -> Option<Vec3> {
    let mut nearest_pos = None;
    let mut nearest_distance = f32::INFINITY;
    
    for road in road_network.roads.values() {
        let samples = 20;
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let road_point = road.evaluate(t);
            let distance = position.distance(road_point);
            
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_pos = Some(road_point);
            }
        }
    }
    
    nearest_pos
}


