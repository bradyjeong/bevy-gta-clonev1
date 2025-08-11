use bevy::prelude::*;
use gta_game::components::*;
use gta_game::systems::*;
use gta_game::systems::vegetation_instancing_integration::*;
use gta_game::systems::rendering::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_rapier3d::plugin::RapierPhysicsPlugin::<()>::default())
        
        // Resources
        .init_resource::<VegetationInstancingConfig>()
        .init_resource::<FrameCounter>()
        
        // Startup
        .add_systems(Startup, (
            setup_camera,
            spawn_test_vegetation_system,
        ))
        
        // Update systems
        .add_systems(Update, (
            // Core batching systems
            frame_counter_system,
            mark_vegetation_instancing_dirty_system,
            
            // Vegetation instancing systems
            collect_vegetation_instances_system,
            update_vegetation_instancing_system,
            animate_vegetation_instances_system,
            vegetation_instancing_metrics_system,
            
            // Integration systems
            integrate_vegetation_with_instancing_system,
        ))
        
        .run();
}

fn setup_camera(mut commands: Commands) {
    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 30.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Spawn active entity (player)
    commands.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        GlobalTransform::default(),
        ActiveEntity,
        Name::new("ActiveEntity"),
    ));
    
    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));
}
