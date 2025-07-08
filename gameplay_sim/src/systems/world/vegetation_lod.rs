//! ───────────────────────────────────────────────
//! System:   Vegetation LOD
//! Purpose:  Manages level-of-detail for vegetation
//! Schedule: Update
//! Reads:    ActiveEntity, Transform, VegetationLOD
//! Writes:   VegetationLOD, Visibility
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::cell::RefCell;
use game_core::prelude::*;
use crate::compat::{TransformBundle, VisibilityBundle};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VegetationDetailLevel {
    High,       // Full 3D model
    Medium,     // Simplified 3D model
    Low,        // Very simple 3D model
    Billboard,  // 2D billboard sprite
    Hidden,     // Not visible
}

#[derive(Component, Debug, Clone)]
pub struct VegetationLOD {
    pub detail_level: VegetationDetailLevel,
    pub last_update: f32,
    pub base_distance: f32,
}

impl Default for VegetationLOD {
    fn default() -> Self {
        Self {
            detail_level: VegetationDetailLevel::High,
            last_update: 0.0,
            base_distance: 100.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct VegetationBillboard {
    pub original_up: Vec3,
}

thread_local! {
    static LAST_LOG: RefCell<f32> = RefCell::new(0.0);
}

pub fn vegetation_lod_system(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut vegetation_query: Query<(Entity, &mut VegetationLOD, &Transform, &mut Visibility)>,
    time: Res<Time>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let current_time = time.elapsed_secs();
        
        let mut updated_count = 0;
        
        for (entity, mut veg_lod, transform, mut visibility) in vegetation_query.iter_mut() {
            // Throttle updates to every 0.2 seconds per vegetation entity
            if current_time - veg_lod.last_update < 0.2 {
                continue;
            }
            
            let distance = active_pos.distance(transform.translation);
            let new_detail_level = calculate_vegetation_lod(distance, veg_lod.base_distance);
            
            if new_detail_level != veg_lod.detail_level {
                veg_lod.detail_level = new_detail_level;
                veg_lod.last_update = current_time;
                
                // Update visibility
                *visibility = match new_detail_level {
                    VegetationDetailLevel::Hidden => Visibility::Hidden,
                    _ => Visibility::Visible,
                };
                
                updated_count += 1;
            }
        }
        
        // Log performance stats occasionally
        LAST_LOG.with(|last| {
            let mut last_log = last.borrow_mut();
            if current_time - *last_log > 5.0 {
                info!("Vegetation LOD: Updated {} entities", updated_count);
                *last_log = current_time;
            }
        });
    }
}

pub fn vegetation_billboard_system(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut billboard_query: Query<(&mut Transform, &VegetationLOD, &VegetationBillboard), (Without<ActiveEntity>, With<VegetationBillboard>)>,
) {
    if let Ok(active_transform) = active_query.single() {
        for (mut transform, veg_lod, billboard) in billboard_query.iter_mut() {
            // Only update billboards for entities at billboard LOD level
            if matches!(veg_lod.detail_level, VegetationDetailLevel::Billboard) {
                // Make billboard face the camera
                let direction = (active_transform.translation - transform.translation).normalize();
                transform.look_to(direction, billboard.original_up);
            }
        }
    }
}

pub fn vegetation_billboard_mesh_generator(
    mut commands: Commands,
    vegetation_query: Query<(Entity, &VegetationLOD), (Changed<VegetationLOD>, With<VegetationLOD>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, veg_lod) in vegetation_query.iter() {
        match veg_lod.detail_level {
            VegetationDetailLevel::Billboard => {
                // Switch to billboard representation
                // In a real implementation, you'd swap the mesh and material
                // Here we just add the billboard component if needed
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(VegetationBillboard {
                        original_up: Vec3::Y,
                    });
                }
            }
            _ => {
                // Remove billboard component for non-billboard LODs
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.remove::<VegetationBillboard>();
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct VegetationBillboardResources {
    pub billboard_mesh: Option<Handle<Mesh>>,
    pub billboard_material: Option<Handle<StandardMaterial>>,
}

fn calculate_vegetation_lod(distance: f32, base_distance: f32) -> VegetationDetailLevel {
    let ratio = distance / base_distance;
    
    if ratio < 0.3 {
        VegetationDetailLevel::High
    } else if ratio < 0.6 {
        VegetationDetailLevel::Medium
    } else if ratio < 1.0 {
        VegetationDetailLevel::Low
    } else if ratio < 2.0 {
        VegetationDetailLevel::Billboard
    } else {
        VegetationDetailLevel::Hidden
    }
}

// Helper function to create vegetation with LOD
pub fn spawn_vegetation_with_lod(
    commands: &mut Commands,
    position: Vec3,
    vegetation_type: ContentType,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let base_distance = match vegetation_type {
        ContentType::Tree => 150.0,
        _ => 100.0,
    };
    
    commands.spawn((
        TransformBundle::from_transform(Transform::from_translation(position)),
        VisibilityBundle::default(),
        DynamicContent {
            content_type: vegetation_type,
        },
        VegetationLOD {
            base_distance,
            ..default()
        },
        Cullable {
            max_distance: base_distance * 2.0,
            is_culled: false,
        },
    )).id()
}

pub fn vegetation_instancing_system(
    vegetation_query: Query<(&Transform, &VegetationLOD)>,
    // In a real implementation, you'd have instancing resources here
) {
    // Group vegetation by LOD level for instanced rendering
    let mut high_detail_instances = Vec::new();
    let mut medium_detail_instances = Vec::new();
    let mut low_detail_instances = Vec::new();
    let mut billboard_instances = Vec::new();
    
    for (transform, veg_lod) in vegetation_query.iter() {
        match veg_lod.detail_level {
            VegetationDetailLevel::High => high_detail_instances.push(transform.clone()),
            VegetationDetailLevel::Medium => medium_detail_instances.push(transform.clone()),
            VegetationDetailLevel::Low => low_detail_instances.push(transform.clone()),
            VegetationDetailLevel::Billboard => billboard_instances.push(transform.clone()),
            VegetationDetailLevel::Hidden => {} // Skip hidden vegetation
        }
    }
    
    // In a full implementation, you'd submit these as instanced draws
}


// Oracle's missing vegetation LOD stubs
pub fn adaptive_vegetation_lod_system() {
    // Adaptive vegetation LOD stub - no implementation yet
}

pub fn vegetation_lod_performance_monitor() {
    // Vegetation LOD performance monitor stub - no implementation yet
}

pub fn vegetation_lod_batching_system() {
    // Vegetation LOD batching stub - no implementation yet
}

#[derive(Resource, Default)]
pub struct LODFrameCounter {
    pub frame: u64,
}
