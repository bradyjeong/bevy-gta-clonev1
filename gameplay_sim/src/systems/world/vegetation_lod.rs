use bevy::prelude::*;
use game_core::components::*;
use crate::systems::distance_cache::{DistanceCache, get_cached_distance};

/// Frame counter for LOD updates
#[derive(Resource, Default)]
pub struct LODFrameCounter {
    pub frame: u64,
}

/// System to update vegetation LOD based on player distance
pub fn vegetation_lod_system(
    mut lod_counter: ResMut<LODFrameCounter>,
    mut distance_cache: ResMut<DistanceCache>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut vegetation_query: Query<
        (Entity, &mut VegetationLOD, &Transform, &mut Visibility, &mut Mesh3d),
        With<VegetationMeshLOD>
    >,
    mesh_lod_query: Query<&VegetationMeshLOD>,
) {
    lod_counter.frame += 1;
    
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;

    for (entity, mut veg_lod, transform, mut visibility, mut mesh_handle) in vegetation_query.iter_mut() {
        // Use distance cache for efficient distance calculation
        let distance = get_cached_distance(
            entity,
            Entity::from_raw(0), // Use entity ID 0 as player placeholder
            transform.translation,
            active_pos,
            &mut distance_cache,
        );

        let old_level = veg_lod.detail_level;
        veg_lod.update_from_distance(distance, lod_counter.frame);

        // Update visibility
        *visibility = if veg_lod.should_be_visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // Update mesh if LOD level changed
        if old_level != veg_lod.detail_level {
            if let Ok(mesh_lod) = mesh_lod_query.get(entity) {
                if let Some(new_mesh) = mesh_lod.get_mesh_for_level(veg_lod.detail_level) {
                    mesh_handle.0 = new_mesh.clone();
                }
            }
        }
    }
}

/// System to make billboard vegetation always face the camera
pub fn vegetation_billboard_system(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut billboard_query: Query<
        (&mut Transform, &VegetationLOD, &VegetationBillboard),
        (Without<ActiveEntity>, With<VegetationBillboard>)
    >,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let camera_pos = active_transform.translation;

    for (mut transform, veg_lod, billboard) in billboard_query.iter_mut() {
        // Only update billboards for entities at billboard LOD level
        if matches!(veg_lod.detail_level, VegetationDetailLevel::Billboard) {
            let direction = (camera_pos - transform.translation).normalize();
            
            // Create rotation to face camera (Y-axis billboard)
            let look_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            transform.rotation = look_rotation;
            
            // Ensure billboard maintains correct size
            let distance_scale = (veg_lod.distance_to_player / 150.0).clamp(0.5, 1.0);
            transform.scale = billboard.original_scale * distance_scale;
        }
    }
}

/// System to create billboard meshes for distant vegetation
pub fn vegetation_billboard_mesh_generator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    // Generate a simple quad mesh for billboards
    let billboard_mesh = create_billboard_quad();
    let billboard_mesh_handle = meshes.add(billboard_mesh);
    
    // Create a simple material for billboards using solid colors instead of textures
    let billboard_material = StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.3), // Green color for vegetation
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    };
    let billboard_material_handle = materials.add(billboard_material);
    
    // Store these as resources for reuse
    commands.insert_resource(VegetationBillboardResources {
        mesh: billboard_mesh_handle,
        material: billboard_material_handle,
    });
}

#[derive(Resource)]
pub struct VegetationBillboardResources {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

fn create_billboard_quad() -> Mesh {
    Mesh::from(Plane3d::default().mesh().size(2.0, 3.0))
}

/// System to adaptively adjust LOD distances based on performance
pub fn adaptive_vegetation_lod_system(
    time: Res<Time>,
    mut vegetation_query: Query<&mut VegetationLOD>,
) {
    let frame_time = time.delta_secs();
    let target_frame_time = 1.0 / 60.0; // 60 FPS target
    
    // If performance is poor, reduce LOD distances to cull more aggressively
    let distance_multiplier = if frame_time > target_frame_time * 1.5 {
        0.8 // Reduce distances by 20%
    } else if frame_time < target_frame_time * 0.8 {
        1.1 // Increase distances by 10%
    } else {
        1.0 // No change
    };
    
    if distance_multiplier != 1.0 {
        for mut veg_lod in vegetation_query.iter_mut() {
            let adjusted_distance = veg_lod.distance_to_player * distance_multiplier;
            veg_lod.update_from_distance(adjusted_distance, 0);
        }
    }
}

/// Performance monitoring for vegetation LOD system
pub fn vegetation_lod_performance_monitor(
    vegetation_query: Query<&VegetationLOD>,
    mut performance_stats: ResMut<PerformanceStats>,
) {
    let mut full_count = 0;
    let mut medium_count = 0;
    let mut billboard_count = 0;
    let mut culled_count = 0;
    
    for veg_lod in vegetation_query.iter() {
        match veg_lod.detail_level {
            VegetationDetailLevel::Full => full_count += 1,
            VegetationDetailLevel::Medium => medium_count += 1,
            VegetationDetailLevel::Billboard => billboard_count += 1,
            VegetationDetailLevel::Culled => culled_count += 1,
        }
    }
    
    // Update performance stats (assuming these fields exist)
    performance_stats.entity_count = full_count + medium_count + billboard_count;
    performance_stats.culled_entities = culled_count;
    
    // EMERGENCY: Disable excessive logging to improve performance
    // Only log every 5 seconds to reduce spam
    use std::time::{Duration, Instant};
    thread_local! {
        static LAST_LOG: std::cell::Cell<Option<Instant>> = std::cell::Cell::new(None);
    }
    
    if cfg!(feature = "debug-ui") {
        LAST_LOG.with(|last| {
            let now = Instant::now();
            let should_log = last.get()
                .map(|last_time| now.duration_since(last_time) > Duration::from_secs(5))
                .unwrap_or(true);
                
            if should_log {
                info!(
                    "Vegetation LOD: Full: {}, Medium: {}, Billboard: {}, Culled: {}",
                    full_count, medium_count, billboard_count, culled_count
                );
                last.set(Some(now));
            }
        });
    }
}

/// System to batch vegetation entities by LOD level for efficient rendering
pub fn vegetation_lod_batching_system(
    vegetation_query: Query<(&VegetationLOD, &Transform, &Mesh3d)>,
    mut batches: Local<Vec<Vec<Entity>>>,
) {
    // Clear previous batches
    batches.clear();
    batches.resize(3, Vec::new()); // Full, Medium, Billboard

    // Group entities by LOD level
    for (entity, (veg_lod, _transform, _mesh)) in vegetation_query.iter().enumerate() {
        let batch_index = match veg_lod.detail_level {
            VegetationDetailLevel::Full => 0,
            VegetationDetailLevel::Medium => 1,
            VegetationDetailLevel::Billboard => 2,
            VegetationDetailLevel::Culled => continue,
        };
        
        // In a real implementation, you'd store the actual entity IDs
        // This is simplified for demonstration
        batches[batch_index].push(Entity::from_raw(entity as u32));
    }
    
    // Process batches for efficient rendering
    // In a full implementation, you'd submit these as instanced draws
}
