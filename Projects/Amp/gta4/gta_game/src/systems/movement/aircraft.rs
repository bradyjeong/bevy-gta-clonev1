use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Helicopter, F16, ActiveEntity, MainRotor, TailRotor};

pub fn helicopter_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut helicopter_query: Query<(&mut Velocity, &Transform), (With<Helicopter>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = helicopter_query.single_mut() else {
        return;
    };

    let speed = 15.0;
    let rotation_speed = 2.5;
    let vertical_speed = 8.0;
    
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
    
    // Rotation - DIRECT velocity control
    if input.pressed(KeyCode::ArrowLeft) {
        target_angular_velocity.y = rotation_speed;
    } else if input.pressed(KeyCode::ArrowRight) {
        target_angular_velocity.y = -rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // HELICOPTER SPECIFIC: Vertical movement with Shift (up) and Ctrl (down)
    if input.pressed(KeyCode::ShiftLeft) {
        target_linear_velocity.y += vertical_speed;
    }
    if input.pressed(KeyCode::ControlLeft) {
        target_linear_velocity.y -= vertical_speed;
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}

pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut main_rotor_query: Query<&mut Transform, (With<MainRotor>, Without<TailRotor>)>,
    mut tail_rotor_query: Query<&mut Transform, (With<TailRotor>, Without<MainRotor>)>,
) {
    let main_rotor_speed = 20.0; // Fast rotation for main rotor
    let tail_rotor_speed = 35.0; // Even faster for tail rotor

    // Rotate main rotors (around Y axis)
    for mut transform in main_rotor_query.iter_mut() {
        let rotation = Quat::from_rotation_y(time.elapsed_secs() * main_rotor_speed);
        transform.rotation = rotation;
    }

    // Rotate tail rotors (around Z axis)  
    for mut transform in tail_rotor_query.iter_mut() {
        let rotation = Quat::from_rotation_z(time.elapsed_secs() * tail_rotor_speed);
        transform.rotation = rotation;
    }
}

pub fn f16_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut f16_query: Query<(&mut Velocity, &Transform), (With<F16>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = f16_query.single_mut() else {
        return;
    };

    let speed = 30.0; // Faster than helicopter
    let afterburner_speed = 60.0; // Afterburner boost
    let rotation_speed = 3.5; // More agile than helicopter
    let vertical_speed = 15.0; // Fast climb/dive
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Determine speed based on afterburner
    let current_speed = if input.pressed(KeyCode::Space) {
        afterburner_speed
    } else {
        speed
    };
    
    // Forward/backward movement
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * current_speed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * current_speed;
    }
    
    // Rotation - DIRECT velocity control
    if input.pressed(KeyCode::ArrowLeft) {
        target_angular_velocity.y = rotation_speed;
    } else if input.pressed(KeyCode::ArrowRight) {
        target_angular_velocity.y = -rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // F16 SPECIFIC: Vertical movement with Q (up) and E (down)
    if input.pressed(KeyCode::KeyQ) {
        target_linear_velocity.y += vertical_speed;
    }
    if input.pressed(KeyCode::KeyE) {
        target_linear_velocity.y -= vertical_speed;
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}
