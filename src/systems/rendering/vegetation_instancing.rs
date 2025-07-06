use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

use crate::components::instanced_vegetation::*;
use crate::components::dirty_flags::{DirtyVegetationInstancing, DirtyPriority, FrameCounter};
use crate::components::world::*;
use crate::components::player::ActiveEntity;
use crate::config::GameConfig;
use crate::factories::{RenderingFactory, StandardRenderingPattern, RenderingBundleType, MaterialType};

/// System to collect vegetation entities for instancing with performance optimization
pub fn collect_vegetation_instances_system(
    mut commands: Commands,
    vegetation_query: Query<(Entity, &Transform, &VegetationBatchable, &GlobalTransform), 
                          (With<Cullable>, Without<DirtyVegetationInstancing>)>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut palm_frond_query: Query<&mut InstancedPalmFrond>,
    mut leaf_cluster_query: Query<&mut InstancedLeafCluster>,
    mut tree_trunk_query: Query<&mut InstancedTreeTrunk>,
    mut bush_query: Query<&mut InstancedBush>,
    config: Res<VegetationInstancingConfig>,
    frame_counter: Res<FrameCounter>,
    _game_config: Res<GameConfig>,
    mut last_update: Local<f32>,
    mut vegetation_groups: Local<HashMap<(VegetationType, Option<Handle<Mesh>>, Option<Handle<StandardMaterial>>), Vec<(Entity, Transform)>>>,
    time: Res<Time>,
) {
    let start_time = std::time::Instant::now();
    let current_time = time.elapsed_secs();
    
    // Update every 3-4 frames instead of every frame
    if current_time - *last_update < config.update_interval * 3.0 {
        return;
    }
    *last_update = current_time;

    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Reuse existing HashMap to avoid recreating every frame
    vegetation_groups.clear();
    
    let max_processing_time = 3.0; // 3ms time budget
    let mut processed_count = 0;
    
    for (entity, transform, batchable, global_transform) in vegetation_query.iter() {
        // Check time budget
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        let distance = active_pos.distance(global_transform.translation());
        
        // Skip if too far away
        if distance > config.culling_distance {
            continue;
        }
        
        let key = (batchable.vegetation_type, batchable.mesh_id.clone(), batchable.material_id.clone());
        vegetation_groups.entry(key).or_default().push((entity, *transform));
        
        processed_count += 1;
    }
    
    // Process each vegetation type with time budgeting
    for ((vegetation_type, _mesh_id, _material_id), entities) in vegetation_groups.iter() {
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        match vegetation_type {
            VegetationType::PalmFrond => {
                process_palm_frond_instances(
                    &mut commands,
                    entities.clone(),
                    &mut palm_frond_query,
                    &config,
                    frame_counter.frame,
                );
            },
            VegetationType::LeafCluster => {
                process_leaf_cluster_instances(
                    &mut commands,
                    entities.clone(),
                    &mut leaf_cluster_query,
                    &config,
                    frame_counter.frame,
                );
            },
            VegetationType::TreeTrunk => {
                process_tree_trunk_instances(
                    &mut commands,
                    entities.clone(),
                    &mut tree_trunk_query,
                    &config,
                    frame_counter.frame,
                );
            },
            VegetationType::Bush => {
                process_bush_instances(
                    &mut commands,
                    entities.clone(),
                    &mut bush_query,
                    &config,
                    frame_counter.frame,
                );
            },
        }
    }
    
    // Report performance metrics
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 2.0 {
        warn!("Vegetation instancing took {:.2}ms (> 2ms budget), processed {} entities", processing_time, processed_count);
    }
}

/// Process palm frond instances
fn process_palm_frond_instances(
    commands: &mut Commands,
    entities: Vec<(Entity, Transform)>,
    palm_frond_query: &mut Query<&mut InstancedPalmFrond>,
    config: &VegetationInstancingConfig,
    current_frame: u64,
) {
    // Find or create instanced palm frond entity
    let mut instanced_entity = None;
    for mut palm_frond in palm_frond_query.iter_mut() {
        if !palm_frond.is_full() {
            palm_frond.clear();
            instanced_entity = Some(palm_frond);
            break;
        }
    }
    
    // Create new instanced entity if needed
    if instanced_entity.is_none() {
        let entity = commands.spawn((
            InstancedPalmFrond::new(config.palm_frond_batch_size),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new("InstancedPalmFronds"),
        )).id();
        
        // Mark for processing
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Normal,
            current_frame,
        ));
        return;
    }
    
    if let Some(mut instanced) = instanced_entity {
        for (entity, transform) in entities.iter().take(config.palm_frond_batch_size) {
            let instance_data = InstanceData {
                transform: *transform,
                color: Vec4::new(0.2, 0.8, 0.3, 1.0), // Green color
                scale_variation: 0.8 + (rand::random::<f32>() * 0.4), // 0.8-1.2 scale
                sway_offset: rand::random::<f32>() * std::f32::consts::TAU,
                age: rand::random::<f32>() * 100.0,
            };
            
            if instanced.add_instance(instance_data) {
                // Hide the original entity since it's now instanced
                commands.entity(*entity).insert(Visibility::Hidden);
            }
        }
    }
}

/// Process leaf cluster instances
fn process_leaf_cluster_instances(
    commands: &mut Commands,
    entities: Vec<(Entity, Transform)>,
    leaf_cluster_query: &mut Query<&mut InstancedLeafCluster>,
    config: &VegetationInstancingConfig,
    current_frame: u64,
) {
    let mut instanced_entity = None;
    for mut leaf_cluster in leaf_cluster_query.iter_mut() {
        if !leaf_cluster.is_full() {
            leaf_cluster.clear();
            instanced_entity = Some(leaf_cluster);
            break;
        }
    }
    
    if instanced_entity.is_none() {
        let entity = commands.spawn((
            InstancedLeafCluster::new(config.leaf_cluster_batch_size),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new("InstancedLeafClusters"),
        )).id();
        
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Normal,
            current_frame,
        ));
        return;
    }
    
    if let Some(mut instanced) = instanced_entity {
        for (entity, transform) in entities.iter().take(config.leaf_cluster_batch_size) {
            let instance_data = InstanceData {
                transform: *transform,
                color: Vec4::new(0.15, 0.7, 0.25, 1.0), // Darker green
                scale_variation: 0.7 + (rand::random::<f32>() * 0.6), // 0.7-1.3 scale
                sway_offset: rand::random::<f32>() * std::f32::consts::TAU,
                age: rand::random::<f32>() * 100.0,
            };
            
            if instanced.add_instance(instance_data) {
                commands.entity(*entity).insert(Visibility::Hidden);
            }
        }
    }
}

/// Process tree trunk instances
fn process_tree_trunk_instances(
    commands: &mut Commands,
    entities: Vec<(Entity, Transform)>,
    tree_trunk_query: &mut Query<&mut InstancedTreeTrunk>,
    config: &VegetationInstancingConfig,
    current_frame: u64,
) {
    let mut instanced_entity = None;
    for mut tree_trunk in tree_trunk_query.iter_mut() {
        if !tree_trunk.is_full() {
            tree_trunk.clear();
            instanced_entity = Some(tree_trunk);
            break;
        }
    }
    
    if instanced_entity.is_none() {
        let entity = commands.spawn((
            InstancedTreeTrunk::new(config.tree_trunk_batch_size),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new("InstancedTreeTrunks"),
        )).id();
        
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Normal,
            current_frame,
        ));
        return;
    }
    
    if let Some(mut instanced) = instanced_entity {
        for (entity, transform) in entities.iter().take(config.tree_trunk_batch_size) {
            let instance_data = InstanceData {
                transform: *transform,
                color: Vec4::new(0.4, 0.25, 0.1, 1.0), // Brown color
                scale_variation: 0.9 + (rand::random::<f32>() * 0.2), // 0.9-1.1 scale
                sway_offset: 0.0, // Trunks don't sway much
                age: rand::random::<f32>() * 100.0,
            };
            
            if instanced.add_instance(instance_data) {
                commands.entity(*entity).insert(Visibility::Hidden);
            }
        }
    }
}

/// Process bush instances
fn process_bush_instances(
    commands: &mut Commands,
    entities: Vec<(Entity, Transform)>,
    bush_query: &mut Query<&mut InstancedBush>,
    config: &VegetationInstancingConfig,
    current_frame: u64,
) {
    let mut instanced_entity = None;
    for mut bush in bush_query.iter_mut() {
        if !bush.is_full() {
            bush.clear();
            instanced_entity = Some(bush);
            break;
        }
    }
    
    if instanced_entity.is_none() {
        let entity = commands.spawn((
            InstancedBush::new(config.bush_batch_size),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new("InstancedBushes"),
        )).id();
        
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Normal,
            current_frame,
        ));
        return;
    }
    
    if let Some(mut instanced) = instanced_entity {
        for (entity, transform) in entities.iter().take(config.bush_batch_size) {
            let instance_data = InstanceData {
                transform: *transform,
                color: Vec4::new(0.1, 0.6, 0.2, 1.0), // Dark green
                scale_variation: 0.6 + (rand::random::<f32>() * 0.8), // 0.6-1.4 scale
                sway_offset: rand::random::<f32>() * std::f32::consts::TAU,
                age: rand::random::<f32>() * 100.0,
            };
            
            if instanced.add_instance(instance_data) {
                commands.entity(*entity).insert(Visibility::Hidden);
            }
        }
    }
}

/// System to update vegetation instance rendering
pub fn update_vegetation_instancing_system(
    mut commands: Commands,
    mut palm_frond_query: Query<(Entity, &mut InstancedPalmFrond), With<DirtyVegetationInstancing>>,
    mut leaf_cluster_query: Query<(Entity, &mut InstancedLeafCluster), With<DirtyVegetationInstancing>>,
    mut tree_trunk_query: Query<(Entity, &mut InstancedTreeTrunk), With<DirtyVegetationInstancing>>,
    mut bush_query: Query<(Entity, &mut InstancedBush), With<DirtyVegetationInstancing>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _config: Res<VegetationInstancingConfig>,
) {
    let start_time = Instant::now();
    
    // Process palm fronds
    for (entity, mut palm_frond) in palm_frond_query.iter_mut() {
        if palm_frond.dirty && !palm_frond.instances.is_empty() {
            // Create instanced mesh
            create_instanced_mesh(&mut commands, entity, &palm_frond.instances, &mut meshes, &mut materials, "PalmFrond");
            palm_frond.dirty = false;
        }
        commands.entity(entity).remove::<DirtyVegetationInstancing>();
        
        if start_time.elapsed().as_millis() > 5 { break; }
    }
    
    // Process leaf clusters
    for (entity, mut leaf_cluster) in leaf_cluster_query.iter_mut() {
        if leaf_cluster.dirty && !leaf_cluster.instances.is_empty() {
            create_instanced_mesh(&mut commands, entity, &leaf_cluster.instances, &mut meshes, &mut materials, "LeafCluster");
            leaf_cluster.dirty = false;
        }
        commands.entity(entity).remove::<DirtyVegetationInstancing>();
        
        if start_time.elapsed().as_millis() > 5 { break; }
    }
    
    // Process tree trunks
    for (entity, mut tree_trunk) in tree_trunk_query.iter_mut() {
        if tree_trunk.dirty && !tree_trunk.instances.is_empty() {
            create_instanced_mesh(&mut commands, entity, &tree_trunk.instances, &mut meshes, &mut materials, "TreeTrunk");
            tree_trunk.dirty = false;
        }
        commands.entity(entity).remove::<DirtyVegetationInstancing>();
        
        if start_time.elapsed().as_millis() > 5 { break; }
    }
    
    // Process bushes
    for (entity, mut bush) in bush_query.iter_mut() {
        if bush.dirty && !bush.instances.is_empty() {
            create_instanced_mesh(&mut commands, entity, &bush.instances, &mut meshes, &mut materials, "Bush");
            bush.dirty = false;
        }
        commands.entity(entity).remove::<DirtyVegetationInstancing>();
        
        if start_time.elapsed().as_millis() > 5 { break; }
    }
}

/// Create instanced mesh for vegetation
fn create_instanced_mesh(
    commands: &mut Commands,
    entity: Entity,
    instances: &[InstanceData],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    vegetation_type: &str,
) {
    // Use RenderingFactory to create standardized mesh and material
    let pattern = match vegetation_type {
        "PalmFrond" => StandardRenderingPattern::CustomCuboid {
            size: Vec3::new(2.0, 0.5, 0.1),
            color: Color::srgb(0.2, 0.8, 0.3),
            material_type: MaterialType::Standard,
        },
        "LeafCluster" => StandardRenderingPattern::CustomSphere {
            radius: 0.5,
            color: Color::srgb(0.15, 0.7, 0.25),
            material_type: MaterialType::Standard,
        },
        "TreeTrunk" => StandardRenderingPattern::CustomCylinder {
            radius: 0.3,
            height: 4.0,
            color: Color::srgb(0.4, 0.25, 0.1),
            material_type: MaterialType::Standard,
        },
        "Bush" => StandardRenderingPattern::CustomSphere {
            radius: 0.8,
            color: Color::srgb(0.1, 0.6, 0.2),
            material_type: MaterialType::Standard,
        },
        _ => StandardRenderingPattern::CustomSphere {
            radius: 0.5,
            color: Color::srgb(0.2, 0.7, 0.2),
            material_type: MaterialType::Standard,
        },
    };
    
    let (mesh_handle, material_handle) = RenderingFactory::create_mesh_and_material(
        meshes,
        materials,
        &pattern,
    );
    
    // Create instanced entity
    commands.entity(entity).insert((
        MeshMaterial3d(material_handle),
        Mesh3d(mesh_handle),
        Transform::IDENTITY,
    ));
    
    info!("Created instanced {} with {} instances", vegetation_type, instances.len());
}

/// System to mark vegetation as dirty when individual vegetation changes
pub fn mark_vegetation_instancing_dirty_system(
    mut commands: Commands,
    changed_vegetation: Query<Entity, (Changed<Transform>, With<VegetationBatchable>, Without<DirtyVegetationInstancing>)>,
    frame_counter: Res<FrameCounter>,
) {
    let current_frame = frame_counter.frame;
    
    for entity in changed_vegetation.iter() {
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Low, // Vegetation changes are low priority
            current_frame,
        ));
    }
}

/// System to animate vegetation instances
pub fn animate_vegetation_instances_system(
    mut palm_frond_query: Query<&mut InstancedPalmFrond>,
    mut leaf_cluster_query: Query<&mut InstancedLeafCluster>,
    mut bush_query: Query<&mut InstancedBush>,
    time: Res<Time>,
) {
    let wind_time = time.elapsed_secs();
    let wind_strength = 0.05; // Reduced from 0.1 to reduce flickering
    
    // Only update every 3rd frame to reduce flickering from constant dirty marking
    if ((wind_time * 60.0) as u32) % 3 != 0 {
        return;
    }
    
    // Animate palm fronds
    for mut palm_frond in palm_frond_query.iter_mut() {
        for instance in palm_frond.instances.iter_mut() {
            let sway = (wind_time + instance.sway_offset).sin() * wind_strength;
            // Apply sway to transform rotation
            instance.transform.rotation = Quat::from_rotation_z(sway);
        }
        palm_frond.dirty = true;
    }
    
    // Animate leaf clusters
    for mut leaf_cluster in leaf_cluster_query.iter_mut() {
        for instance in leaf_cluster.instances.iter_mut() {
            let sway = (wind_time * 0.8 + instance.sway_offset).sin() * wind_strength * 0.7;
            instance.transform.rotation = Quat::from_rotation_y(sway);
        }
        leaf_cluster.dirty = true;
    }
    
    // Animate bushes
    for mut bush in bush_query.iter_mut() {
        for instance in bush.instances.iter_mut() {
            let sway = (wind_time * 1.2 + instance.sway_offset).sin() * wind_strength * 0.5;
            instance.transform.rotation = Quat::from_rotation_x(sway * 0.5) * Quat::from_rotation_z(sway);
        }
        bush.dirty = true;
    }
}

/// Debug system to report vegetation instancing metrics
pub fn vegetation_instancing_metrics_system(
    palm_frond_query: Query<&InstancedPalmFrond>,
    leaf_cluster_query: Query<&InstancedLeafCluster>,
    tree_trunk_query: Query<&InstancedTreeTrunk>,
    bush_query: Query<&InstancedBush>,
    mut last_report: Local<f32>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    if current_time - *last_report < 5.0 {
        return;
    }
    *last_report = current_time;
    
    let palm_frond_count: usize = palm_frond_query.iter().map(|p| p.instances.len()).sum();
    let leaf_cluster_count: usize = leaf_cluster_query.iter().map(|l| l.instances.len()).sum();
    let tree_trunk_count: usize = tree_trunk_query.iter().map(|t| t.instances.len()).sum();
    let bush_count: usize = bush_query.iter().map(|b| b.instances.len()).sum();
    
    let total_instances = palm_frond_count + leaf_cluster_count + tree_trunk_count + bush_count;
    let total_draws = palm_frond_query.iter().count() + leaf_cluster_query.iter().count() + 
                     tree_trunk_query.iter().count() + bush_query.iter().count();
    
    info!(
        "Vegetation Instancing - Total Instances: {} | Draw Calls: {} | PF:{} LC:{} TT:{} B:{}",
        total_instances, total_draws, palm_frond_count, leaf_cluster_count, tree_trunk_count, bush_count
    );
}
