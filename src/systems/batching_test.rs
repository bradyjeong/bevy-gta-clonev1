//! ───────────────────────────────────────────────
//! System:   Batching Test
//! Purpose:  Handles batch processing and optimization
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;


/// Test system to demonstrate and verify batching performance
pub fn batching_test_system(
    mut commands: Commands,
    dirty_metrics: Res<DirtyFlagsMetrics>,
    batch_config: Res<BatchConfig>,
    frame_counter: Res<FrameCounter>,
    time: Res<Time>,
    mut test_timer: Local<f32>,
    mut spawn_timer: Local<f32>,
    test_entities: Query<Entity, With<Transform>>,
) {
    *test_timer += time.delta_secs();
    *spawn_timer += time.delta_secs();
    
    // Spawn test entities periodically to test the batching system
    if *spawn_timer > 2.0 && test_entities.iter().count() < 500 {
        spawn_test_entities(&mut commands, 50, frame_counter.frame);
        *spawn_timer = 0.0;
    }
    
    // Report performance every 10 seconds
    if *test_timer > 10.0 {
        let entity_count = test_entities.iter().count();
        
        info!(
            "Batching Test Report - Entities: {} | Batch Sizes: T:{} V:{} P:{} L:{} | Dirty Entities: T:{} V:{} P:{} L:{}",
            entity_count,
            batch_config.transform_batch_size,
            batch_config.visibility_batch_size,
            batch_config.physics_batch_size,
            batch_config.lod_batch_size,
            dirty_metrics.entities_marked_transform,
            dirty_metrics.entities_marked_visibility,
            dirty_metrics.entities_marked_physics,
            dirty_metrics.entities_marked_lod,
        );
        
        *test_timer = 0.0;
    }
}

/// Spawn test entities with various dirty flags to test the batching system
fn spawn_test_entities(commands: &mut Commands, count: usize, current_frame: u64) {
    use rand::prelude::*;
    let mut rng = thread_rng();
    
    for i in 0..count {
        let position = Vec3::new(
            rng.gen_range(-500.0..500.0),
            rng.gen_range(0.0..10.0),
            rng.gen_range(-500.0..500.0),
        );
        
        let entity = commands.spawn((
            Transform::from_translation(position),
            Visibility::Visible,
            Cullable::new(200.0),
            DirtyFlagsBundle::default(),
            RigidBody::Dynamic,
            Velocity::default(),
            Collider::cuboid(1.0, 1.0, 1.0),
        )).id();
        
        // Randomly mark entities with different dirty flags
        match i % 4 {
            0 => {
                commands.entity(entity).insert(DirtyTransform::new(
                    DirtyPriority::Normal,
                    current_frame,
                ));
            }
            1 => {
                commands.entity(entity).insert(DirtyVisibility::new(
                    DirtyPriority::Normal,
                    current_frame,
                ));
            }
            2 => {
                commands.entity(entity).insert(DirtyPhysics::new(
                    DirtyPriority::High,
                    current_frame,
                ));
            }
            3 => {
                commands.entity(entity).insert(DirtyLOD::new(
                    DirtyPriority::Normal,
                    current_frame,
                    position.length(),
                ));
            }
            _ => {}
        }
    }
    
    info!("Spawned {} test entities for batching system", count);
}

/// System to stress test the batching system
pub fn batching_stress_test_system(
    mut commands: Commands,
    entities_query: Query<Entity, With<Transform>>,
    frame_counter: Res<FrameCounter>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut stress_timer: Local<f32>,
) {
    if keyboard_input.just_pressed(KeyCode::F10) {
        info!("Starting batching stress test...");
        
        // Mark all entities as dirty with high priority
        let current_frame = frame_counter.frame;
        let entities: Vec<_> = entities_query.iter().collect();
        
        for (i, entity) in entities.iter().enumerate() {
            match i % 4 {
                0 => {
                    commands.entity(*entity).insert(DirtyTransform::new(
                        DirtyPriority::Critical,
                        current_frame,
                    ));
                }
                1 => {
                    commands.entity(*entity).insert(DirtyVisibility::new(
                        DirtyPriority::Critical,
                        current_frame,
                    ));
                }
                2 => {
                    commands.entity(*entity).insert(DirtyPhysics::new(
                        DirtyPriority::Critical,
                        current_frame,
                    ));
                }
                3 => {
                    commands.entity(*entity).insert(DirtyLOD::new(
                        DirtyPriority::Critical,
                        current_frame,
                        0.0,
                    ));
                }
                _ => {}
            }
        }
        
        info!("Marked {} entities as dirty for stress test", entities.len());
    }
    
    *stress_timer += time.delta_secs();
    
    // Randomly mark entities as dirty to simulate real gameplay
    if *stress_timer > 0.1 {
        use rand::prelude::*;
        let mut rng = thread_rng();
        
        let entities: Vec<_> = entities_query.iter().take(20).collect();
        let current_frame = frame_counter.frame;
        
        for entity in entities {
            if rng.gen_bool(0.3) { // 30% chance
                let priority = if rng.gen_bool(0.1) {
                    DirtyPriority::High
                } else {
                    DirtyPriority::Normal
                };
                
                match rng.gen_range(0..4) {
                    0 => {
                        commands.entity(entity).insert(DirtyTransform::new(
                            priority,
                            current_frame,
                        ));
                    }
                    1 => {
                        commands.entity(entity).insert(DirtyVisibility::new(
                            priority,
                            current_frame,
                        ));
                    }
                    2 => {
                        commands.entity(entity).insert(DirtyPhysics::new(
                            priority,
                            current_frame,
                        ));
                    }
                    3 => {
                        commands.entity(entity).insert(DirtyLOD::new(
                            priority,
                            current_frame,
                            0.0,
                        ));
                    }
                    _ => {}
                }
            }
        }
        
        *stress_timer = 0.0;
    }
}

/// Performance comparison system
pub fn batching_performance_comparison_system(
    dirty_metrics: Res<DirtyFlagsMetrics>,
    performance_stats: Res<PerformanceStats>,
    time: Res<Time>,
    mut comparison_timer: Local<f32>,
    mut baseline_frame_time: Local<f32>,
    mut samples: Local<Vec<f32>>,
) {
    *comparison_timer += time.delta_secs();
    
    // Collect frame time samples
    samples.push(performance_stats.frame_time);
    if samples.len() > 60 { // Keep only last 60 samples (1 second at 60fps)
        samples.remove(0);
    }
    
    // Report performance comparison every 15 seconds
    if *comparison_timer > 15.0 {
        let avg_frame_time = samples.iter().sum::<f32>() / samples.len() as f32;
        let max_frame_time = samples.iter().fold(0.0f32, |acc, &x| acc.max(x));
        let min_frame_time = samples.iter().fold(1000.0f32, |acc, &x| acc.min(x));
        
        if *baseline_frame_time == 0.0 {
            *baseline_frame_time = avg_frame_time;
            info!("Baseline frame time set: {:.2}ms", avg_frame_time);
        } else {
            let improvement = (*baseline_frame_time - avg_frame_time) / *baseline_frame_time * 100.0;
            
            info!(
                "Performance Comparison - Avg: {:.2}ms | Min: {:.2}ms | Max: {:.2}ms | Improvement: {:.1}% | Processing Time: T:{:.1}ms V:{:.1}ms P:{:.1}ms L:{:.1}ms",
                avg_frame_time,
                min_frame_time,
                max_frame_time,
                improvement,
                dirty_metrics.processing_time_transform,
                dirty_metrics.processing_time_visibility,
                dirty_metrics.processing_time_physics,
                dirty_metrics.processing_time_lod,
            );
        }
        
        *comparison_timer = 0.0;
    }
}

/// Helper function to demonstrate entity marking
pub fn demo_entity_marking(
    commands: &mut Commands,
    entity: Entity,
    mark_type: DirtyMarkType,
    frame: u64,
) {
    match mark_type {
        DirtyMarkType::Transform => {
            commands.entity(entity).insert(DirtyTransform::new(
                DirtyPriority::Normal,
                frame,
            ));
        }
        DirtyMarkType::Visibility => {
            commands.entity(entity).insert(DirtyVisibility::new(
                DirtyPriority::Normal,
                frame,
            ));
        }
        DirtyMarkType::Physics => {
            commands.entity(entity).insert(DirtyPhysics::new(
                DirtyPriority::High,
                frame,
            ));
        }
        DirtyMarkType::LOD => {
            commands.entity(entity).insert(DirtyLOD::new(
                DirtyPriority::Normal,
                frame,
                0.0,
            ));
        }
        DirtyMarkType::All => {
            commands.entity(entity).insert((
                DirtyTransform::new(DirtyPriority::Normal, frame),
                DirtyVisibility::new(DirtyPriority::Normal, frame),
                DirtyPhysics::new(DirtyPriority::High, frame),
                DirtyLOD::new(DirtyPriority::Normal, frame, 0.0),
            ));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DirtyMarkType {
    Transform,
    Visibility,
    Physics,
    LOD,
    All,
}

/// System to clean up test entities
pub fn cleanup_test_entities_system(
    mut commands: Commands,
    test_entities: Query<Entity, With<Transform>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        let entity_count = test_entities.iter().count();
        
        for entity in test_entities.iter() {
            commands.entity(entity).despawn();
        }
        
        info!("Cleaned up {} test entities", entity_count);
    }
}
