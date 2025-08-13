use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::world::RoadNetwork;
use crate::components::RoadEntity;
use crate::systems::world::road_mesh::{generate_road_mesh, generate_road_markings_mesh};
// Removed unused import
use crate::events::PlayerChunkChanged;

// MAIN ROAD GENERATION SYSTEM (Replaces old grid system)
// UNIFIED Y-COORDINATE SYSTEM (prevents z-fighting):
// - Terrain:      y = -0.15  (15cm below ground)  
// - All Roads:    y =  0.0   (unified ground level - highways, main streets, side streets, alleys)
// - Road Markings:y =  0.01  (1cm above road surface)
// Unified road height prevents overlapping geometry and z-fighting issues

/// Observer function that handles road generation when player enters new chunks
pub fn handle_player_chunk_changed(
    trigger: Trigger<PlayerChunkChanged>,
    mut commands: Commands,
    _road_network: ResMut<RoadNetwork>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    road_query: Query<(Entity, &Transform), With<RoadEntity>>,
) {
    let event = trigger.event();
    let active_pos = event.position;
    let (chunk_x, chunk_z) = event.new_chunk;
    
    let chunk_size = 400.0f32;
    let active_radius = 800.0f32;   // Increased for better road coverage
    let cleanup_radius = 2000.0f32;  // Very large cleanup radius to prevent premature despawning
    
    // Clean up distant road entities
    for (entity, transform) in road_query.iter() {
        // Simple distance check - only remove roads that are extremely far away
        let distance = active_pos.distance(transform.translation);
        if distance > cleanup_radius {
            println!("DEBUG: Cleaning up road entity at distance {}", distance);
            commands.entity(entity).despawn();
        }
    }
    
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
                // NOTE: Legacy road generation - new RoadNetwork uses different approach
                // let new_road_ids = road_network.generate_chunk_roads(check_chunk_x, check_chunk_z);
                // println!("DEBUG: Generated {} roads for chunk ({}, {})", new_road_ids.len(), check_chunk_x, check_chunk_z);
                
                // Spawn mesh entities for new roads
                // for road_id in new_road_ids {
                //     if let Some(road) = road_network.roads.get(&road_id) {
                //         // println!("DEBUG: Spawning road entity for road {} at start {:?} with type {:?}", road_id, road.evaluate(0.0), road.road_type);
                //         spawn_road_entity(&mut commands, road_id, road, &mut meshes, &mut materials);
                //     }
                // }
            }
        }
    }
}

#[allow(dead_code)]
fn spawn_road_entity(
    commands: &mut Commands,
    road_id: u32,
    road: &crate::systems::world::road_network::RoadSpline,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use crate::constants::*;
    
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
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP), // Only vehicles collide with roads, not characters
    )).id();
    
    // Main road surface mesh
    let road_mesh = generate_road_mesh(road);
    let mesh_entity = commands.spawn((
        Mesh3d(meshes.add(road_mesh)),
        MeshMaterial3d(road_material),
        Transform::from_translation(Vec3::new(-start_pos.x, 0.0, -start_pos.z)), // Road surface at ground level (y = 0.0)
    )).id();
    commands.entity(road_entity).add_child(mesh_entity);
    
    // Road markings for visibility and realism
    let marking_meshes = generate_road_markings_mesh(road);
    for marking_mesh in marking_meshes {
        let marking_entity = commands.spawn((
            Mesh3d(meshes.add(marking_mesh)),
            MeshMaterial3d(marking_material.clone()),
            Transform::from_translation(Vec3::new(-start_pos.x, 0.01, -start_pos.z)), // Road markings 1cm above road surface (y = 0.01)
        )).id();
        commands.entity(road_entity).add_child(marking_entity);
    }
}

#[allow(dead_code)]
fn create_road_collider(road: &crate::systems::world::road_network::RoadSpline) -> Collider {
    let width = road.road_type.width();
    let length = road.length();
    
    // Create a thin, flat collider for all roads to avoid interfering with character movement
    // Use consistent cuboid collider for all road types - simpler and more predictable
    Collider::cuboid(width * 0.5, 0.02, length * 0.5)  // Very thin (4cm): road surface only
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

// Enhanced road detection for vehicles and NPCs - updated for new RoadNetwork
pub fn is_on_road_spline(position: Vec3, road_network: &RoadNetwork, tolerance: f32) -> bool {
    road_network.is_near_road(position, tolerance)
}

#[allow(dead_code)]
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
    for mut transform in vehicle_query.iter_mut() {
        if !is_on_road_spline(transform.translation, &road_network, 2.0) {
            // Find nearest road and snap to it
            if let Some(nearest_road_pos) = find_nearest_road_position(transform.translation, &road_network) {
                transform.translation.x = nearest_road_pos.x;
                transform.translation.z = nearest_road_pos.z;
            }
        }
    }
    
    // Similar for NPCs
    for mut transform in npc_query.iter_mut() {
        if !is_on_road_spline(transform.translation, &road_network, 1.0) {
            if let Some(nearest_road_pos) = find_nearest_road_position(transform.translation, &road_network) {
                transform.translation.x = nearest_road_pos.x;
                transform.translation.z = nearest_road_pos.z;
            }
        }
    }
}

fn find_nearest_road_position(position: Vec3, road_network: &RoadNetwork) -> Option<Vec3> {
    // Use new RoadNetwork's get_nearest_road_point method
    road_network.get_nearest_road_point(position)
}


