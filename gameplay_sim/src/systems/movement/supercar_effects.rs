use bevy::prelude::*;
use game_core::components::{Car, SuperCar, ActiveEntity, ExhaustFlame};

/// Pre-spawned exhaust flame pool for performance
#[derive(Resource)]
pub struct ExhaustFlamePool {
    pub flames: Vec<Entity>,
    pub available_flames: Vec<Entity>,
    pub max_flames: usize,
}

impl Default for ExhaustFlamePool {
    fn default() -> Self {
        Self {
            flames: Vec::new(),
            available_flames: Vec::new(),
            max_flames: 20, // Pre-spawn 20 flames
        }
    }
}

/// Initialize the exhaust flame pool at startup
pub fn initialize_exhaust_pool_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pool: ResMut<ExhaustFlamePool>,
) {
    // Pre-spawn flame entities
    for _ in 0..pool.max_flames {
        let flame_entity = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.6, 0.2),
                emissive: LinearRgba::rgb(1.0, 0.6, 0.2),
                alpha_mode: AlphaMode::Add,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, -1000.0, 0.0)), // Hidden position
            Visibility::Hidden,
            ExhaustFlame,
        )).id();
        
        pool.flames.push(flame_entity);
        pool.available_flames.push(flame_entity);
    }
}

/// Focused system for managing supercar visual effects using pre-spawned entities
pub fn supercar_effects_system(
    mut supercar_query: Query<(&Transform, &mut SuperCar), (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
    mut flame_query: Query<(&mut Transform, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>), (With<ExhaustFlame>, Without<SuperCar>)>,
    mut pool: ResMut<ExhaustFlamePool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((transform, mut supercar)) = supercar_query.single_mut() else {
        return;
    };

    // Reset exhaust timer
    if supercar.exhaust_timer > 0.04 {
        supercar.exhaust_timer = 0.0;
        
        // Return all flames to available pool first
        for flame_entity in &pool.flames {
            if let Ok((mut flame_transform, mut visibility, _)) = flame_query.get_mut(*flame_entity) {
                *visibility = Visibility::Hidden;
                flame_transform.translation = Vec3::new(0.0, -1000.0, 0.0);
            }
        }
        pool.available_flames = pool.flames.clone();
        
        // Use available flames for current exhaust
        let exhaust_pos = transform.translation + transform.back() * 2.8 + Vec3::new(0.0, 0.15, 0.0);
        
        // Determine flame characteristics based on engine state
        let (flame_color, emission_intensity) = if supercar.turbo_boost && supercar.turbo_stage >= 3 {
            (Color::srgb(0.1, 0.4, 1.0), 4.5) // Quad-turbo plasma flames
        } else if supercar.turbo_boost && supercar.turbo_stage >= 2 {
            (Color::srgb(0.3, 0.7, 1.0), 3.2) // Dual-turbo blue flames
        } else if supercar.turbo_boost {
            (Color::srgb(0.6, 0.8, 1.0), 2.8) // Single turbo blue-orange
        } else if supercar.rpm > 5500.0 {
            (Color::srgb(1.0, 0.2, 0.0), 2.5) // High RPM red-orange flames
        } else if supercar.rpm > 4000.0 {
            (Color::srgb(1.0, 0.4, 0.1), 1.8) // Medium RPM orange flames
        } else {
            (Color::srgb(1.0, 0.6, 0.2), 1.0) // Normal exhaust flames
        };
        
        // Activate flames for quad exhaust (4 tailpipes)
        let flames_needed = if supercar.turbo_boost { 8 } else { 4 };
        let mut flames_used = 0;
        
        for i in 0..4 {
            if flames_used >= flames_needed || pool.available_flames.is_empty() {
                break;
            }
            
            let side_offset = match i {
                0 => Vec3::new(-0.6, 0.0, 0.0),  // Left outer
                1 => Vec3::new(-0.2, 0.0, 0.0),  // Left inner
                2 => Vec3::new(0.2, 0.0, 0.0),   // Right inner
                3 => Vec3::new(0.6, 0.0, 0.0),   // Right outer
                _ => Vec3::ZERO,
            };
            let final_pos = exhaust_pos + transform.right() * side_offset.x;
            
            // Primary flame
            if let Some(flame_entity) = pool.available_flames.pop() {
                if let Ok((mut flame_transform, mut visibility, material_handle)) = flame_query.get_mut(flame_entity) {
                    flame_transform.translation = final_pos;
                    *visibility = Visibility::Visible;
                    
                    // Update material properties
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        material.base_color = flame_color;
                        material.emissive = LinearRgba::rgb(
                            flame_color.to_linear().red * emission_intensity,
                            flame_color.to_linear().green * emission_intensity,
                            flame_color.to_linear().blue * emission_intensity,
                        );
                    }
                    flames_used += 1;
                }
            }
            
            // Secondary flame trail (for turbo mode)
            if supercar.turbo_boost && flames_used < flames_needed {
                if let Some(flame_entity) = pool.available_flames.pop() {
                    if let Ok((mut flame_transform, mut visibility, material_handle)) = flame_query.get_mut(flame_entity) {
                        flame_transform.translation = final_pos + transform.back() * 0.3;
                        flame_transform.scale = Vec3::splat(0.5); // Smaller secondary flame
                        *visibility = Visibility::Visible;
                        
                        // Update material properties for secondary flame
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            material.base_color = Color::srgb(0.4, 0.8, 1.0);
                            material.emissive = LinearRgba::rgb(0.3, 0.6, 0.9);
                        }
                        flames_used += 1;
                    }
                }
            }
        }
    }
}

/// System to automatically hide exhaust flames after a short duration
pub fn exhaust_flame_cleanup_system(
    mut flame_query: Query<(&mut Visibility, &mut Transform), With<ExhaustFlame>>,
    _pool: ResMut<ExhaustFlamePool>,
    _time: Res<Time>,
) {
    
    // Simple auto-hide after 0.1 seconds
    for (mut visibility, mut transform) in flame_query.iter_mut() {
        if matches!(*visibility, Visibility::Visible) {
            // Gradually reduce scale for natural fade effect
            transform.scale *= 0.95;
            
            // Hide when scale gets too small
            if transform.scale.x < 0.1 {
                *visibility = Visibility::Hidden;
                transform.translation = Vec3::new(0.0, -1000.0, 0.0);
                transform.scale = Vec3::ONE;
            }
        }
    }
}
