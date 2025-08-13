use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Car, SuperCar, ActiveEntity, ExhaustFlame};

pub fn car_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCar>)>,
) {
    let Ok((mut velocity, transform)) = car_query.single_mut() else {
        return;
    };

    let speed = 25.0;
    let rotation_speed = 2.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * speed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * speed;
    }
    
    // Rotation (only when moving) - DIRECT velocity control
    if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::ArrowDown) {
        if input.pressed(KeyCode::ArrowLeft) {
            target_angular_velocity.y = rotation_speed;
        } else if input.pressed(KeyCode::ArrowRight) {
            target_angular_velocity.y = -rotation_speed;
        } else {
            target_angular_velocity.y = 0.0; // Force zero rotation
        }
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation when not moving
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}

pub fn supercar_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut supercar_query: Query<(&mut Velocity, &Transform, &mut SuperCar), (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((mut velocity, transform, mut supercar)) = supercar_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    supercar.exhaust_timer += dt;
    
    // Enhanced performance parameters
    let base_speed = supercar.max_speed;
    let rotation_speed = 3.5; // Tighter turning than regular cars
    
    // Turbo boost activation
    let turbo_active = input.pressed(KeyCode::Space);
    supercar.turbo_boost = turbo_active;
    
    let speed_multiplier = if turbo_active { 1.8 } else { 1.0 };
    let target_speed = base_speed * speed_multiplier;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * target_speed;
        
        // Spawn exhaust flames when accelerating
        if supercar.exhaust_timer > 0.1 {
            supercar.exhaust_timer = 0.0;
            
            // Spawn exhaust particles behind the car
            let exhaust_pos = transform.translation + transform.back() * 2.5 + Vec3::new(0.0, 0.2, 0.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.15))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if turbo_active { Color::srgb(0.2, 0.4, 1.0) } else { Color::srgb(1.0, 0.3, 0.0) },
                    emissive: if turbo_active { 
                        LinearRgba::rgb(0.3, 0.6, 1.5)  // Blue turbo flames
                    } else { 
                        LinearRgba::rgb(1.0, 0.4, 0.0)  // Orange flames
                    },
                    ..default()
                })),
                Transform::from_translation(exhaust_pos),
                ExhaustFlame,
            ));
        }
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * (target_speed * 0.7);
    }
    
    // Enhanced rotation (can turn even without moving for supercars) - DIRECT velocity control
    if input.pressed(KeyCode::ArrowLeft) {
        target_angular_velocity.y = rotation_speed;
    } else if input.pressed(KeyCode::ArrowRight) {
        target_angular_velocity.y = -rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}
