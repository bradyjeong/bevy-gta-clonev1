// Boundary condition tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use crate::utils::*;

#[test]
fn test_world_boundary_enforcement() {
    let mut app = create_test_app();
    let config = GameConfig::default();
    
    // Create entity at world boundary
    let entity = {
        let mut world = app.world_mut();
        world.spawn((
            Transform::from_xyz(config.physics.max_world_coord - 1.0, 1.0, 0.0),
            GlobalTransform::default(),
            Velocity {
                linvel: Vec3::new(10.0, 0.0, 0.0), // Moving toward boundary
                angvel: Vec3::ZERO,
            },
            RigidBody::Dynamic,
            Collider::ball(1.0),
        )).id()
    };
    // Run simulation
    run_simulation_duration(&mut app, 2.0);
    // Check that entity was stopped at boundary
    let world = app.world();
    let transform = world.get::<Transform>(entity).unwrap();
    let velocity = world.get::<Velocity>(entity).unwrap();
    assert!(transform.translation.x <= config.physics.max_world_coord, 
           "Entity should not exceed world boundary");
    // Velocity toward boundary should be stopped
    if transform.translation.x >= config.physics.max_world_coord - 1.0 {
        assert!(velocity.linvel.x <= 0.0, "Velocity toward boundary should be stopped");
    }
}
fn test_negative_boundary_enforcement() {
    // Create entity at negative boundary
            Transform::from_xyz(-config.physics.max_world_coord + 1.0, 1.0, 0.0),
                linvel: Vec3::new(-10.0, 0.0, 0.0), // Moving toward negative boundary
    // Check boundary enforcement
    assert!(transform.translation.x >= -config.physics.max_world_coord, 
           "Entity should not exceed negative world boundary");
    if transform.translation.x <= -config.physics.max_world_coord + 1.0 {
        assert!(velocity.linvel.x >= 0.0, "Velocity toward negative boundary should be stopped");
fn test_height_boundaries() {
    // Test upper height boundary
    let high_entity = {
            Transform::from_xyz(0.0, 1000.0, 0.0), // Very high
                linvel: Vec3::new(0.0, 50.0, 0.0), // Moving up
    // Test lower height boundary (underground)
    let low_entity = {
            Transform::from_xyz(0.0, -10.0, 0.0), // Underground
                linvel: Vec3::new(0.0, -10.0, 0.0), // Moving further down
    // Check height boundaries
    let high_transform = world.get::<Transform>(high_entity).unwrap();
    let high_velocity = world.get::<Velocity>(high_entity).unwrap();
    let low_transform = world.get::<Transform>(low_entity).unwrap();
    let low_velocity = world.get::<Velocity>(low_entity).unwrap();
    // High entity should have reasonable height
    assert!(high_transform.translation.y < 2000.0, "Entity should not go too high");
    // Low entity should not stay far underground
    assert!(low_transform.translation.y > -50.0, "Entity should not stay far underground");
    // Underground entity should have upward velocity correction
    if low_transform.translation.y < 0.0 {
        assert!(low_velocity.linvel.y >= 0.0, "Underground entity should have upward correction");
fn test_vehicle_boundary_behavior() {
    let (_, vehicle_entity, _) = setup_test_scene(&mut app);
    // Move vehicle to boundary
    {
        if let Some(mut transform) = world.get_mut::<Transform>(vehicle_entity) {
            transform.translation.x = config.physics.max_world_coord - 5.0;
        }
        if let Some(mut velocity) = world.get_mut::<Velocity>(vehicle_entity) {
            velocity.linvel = Vec3::new(20.0, 0.0, 0.0); // Fast toward boundary
    run_simulation_duration(&mut app, 1.0);
    // Check vehicle boundary behavior
    let transform = world.get::<Transform>(vehicle_entity).unwrap();
    let velocity = world.get::<Velocity>(vehicle_entity).unwrap();
           "Vehicle should respect world boundary");
    // Vehicle should be slowed or stopped at boundary
    if transform.translation.x >= config.physics.max_world_coord - 2.0 {
        assert!(velocity.linvel.x < 15.0, "Vehicle should slow down near boundary");
fn test_player_boundary_behavior() {
    let (player_entity, _, _) = setup_test_scene(&mut app);
    // Move player to boundary
        if let Some(mut transform) = world.get_mut::<Transform>(player_entity) {
            transform.translation.z = config.physics.max_world_coord - 2.0;
        if let Some(mut velocity) = world.get_mut::<Velocity>(player_entity) {
            velocity.linvel = Vec3::new(0.0, 0.0, 10.0); // Moving toward Z boundary
    // Check player boundary behavior
    let transform = world.get::<Transform>(player_entity).unwrap();
    let velocity = world.get::<Velocity>(player_entity).unwrap();
    assert!(transform.translation.z <= config.physics.max_world_coord, 
           "Player should respect world boundary");
    if transform.translation.z >= config.physics.max_world_coord - 1.0 {
        assert!(velocity.linvel.z <= 0.0, "Player should be stopped at boundary");
fn test_boundary_soft_vs_hard_limits() {
    // Create entities at different distances from boundary
    let entities = [
        (config.physics.max_world_coord - 10.0, "far"),     // Far from boundary
        (config.physics.max_world_coord - 2.0, "near"),     // Near boundary  
        (config.physics.max_world_coord - 0.5, "very_near"), // Very near boundary
    ];
    let mut test_entities = Vec::new();
    for (x_pos, _description) in entities {
        let entity = {
            let mut world = app.world_mut();
            world.spawn((
                Transform::from_xyz(x_pos, 1.0, 0.0),
                GlobalTransform::default(),
                Velocity {
                    linvel: Vec3::new(5.0, 0.0, 0.0), // All moving toward boundary
                    angvel: Vec3::ZERO,
                },
                RigidBody::Dynamic,
                Collider::ball(0.5),
            )).id()
        };
        test_entities.push(entity);
    // Check different boundary behaviors
    for (i, entity) in test_entities.iter().enumerate() {
        let transform = world.get::<Transform>(*entity).unwrap();
        let velocity = world.get::<Velocity>(*entity).unwrap();
        
        // All should respect hard boundary
        assert!(transform.translation.x <= config.physics.max_world_coord, 
               "Entity {} should respect hard boundary", i);
        // Entities very close to boundary should be strongly affected
        if i == 2 { // Very near boundary
            assert!(velocity.linvel.x < 2.0, "Entity very near boundary should be slowed significantly");
fn test_corner_boundary_conditions() {
    // Create entity at corner of world boundary
    let corner_entity = {
            Transform::from_xyz(
                config.physics.max_world_coord - 1.0, 
                1.0, 
                config.physics.max_world_coord - 1.0
            ),
                linvel: Vec3::new(5.0, 0.0, 5.0), // Moving toward both boundaries
            Collider::ball(0.5),
    // Check corner boundary handling
    let transform = world.get::<Transform>(corner_entity).unwrap();
    let velocity = world.get::<Velocity>(corner_entity).unwrap();
    // Should respect both boundaries
           "Entity should respect X boundary");
           "Entity should respect Z boundary");
    // Velocity should be corrected for both axes
        assert!(velocity.linvel.x <= 0.0, "X velocity should be corrected at boundary");
        assert!(velocity.linvel.z <= 0.0, "Z velocity should be corrected at boundary");
fn test_boundary_with_high_speed() {
    // Create very fast entity approaching boundary
    let fast_entity = {
            Transform::from_xyz(config.physics.max_world_coord - 50.0, 1.0, 0.0),
                linvel: Vec3::new(100.0, 0.0, 0.0), // Very fast
    // Run simulation with small timesteps to test high-speed boundary handling
    for _ in 0..120 { // 2 seconds at 60 FPS
        app.update();
        // Check that entity never exceeds boundary during high-speed approach
        let world = app.world();
        let transform = world.get::<Transform>(fast_entity).unwrap();
        assert!(transform.translation.x <= config.physics.max_world_coord + 1.0, 
               "Fast entity should not significantly exceed boundary");
    // Final check
    let final_transform = world.get::<Transform>(fast_entity).unwrap();
    let final_velocity = world.get::<Velocity>(fast_entity).unwrap();
    assert!(final_transform.translation.x <= config.physics.max_world_coord, 
           "Fast entity should respect boundary");
    assert!(final_velocity.linvel.x <= 0.0, 
           "Fast entity should have velocity corrected at boundary");
fn test_boundary_teleportation_prevention() {
    // Create entity and attempt to teleport it beyond boundary
            Transform::from_xyz(0.0, 1.0, 0.0),
            Velocity::default(),
    // Attempt teleportation beyond boundary
        if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            transform.translation.x = config.physics.max_world_coord + 100.0; // Way beyond boundary
    // Run simulation - boundary system should correct position
    // Check that position was corrected
           "Teleported entity should be corrected to within boundary");
fn test_multiple_entities_at_boundary() {
    // Create multiple entities at boundary
    let mut entities = Vec::new();
    for i in 0..5 {
                Transform::from_xyz(
                    config.physics.max_world_coord - 2.0, 
                    1.0, 
                    i as f32 * 2.0
                ),
                    linvel: Vec3::new(10.0, 0.0, 0.0), // All moving toward boundary
        entities.push(entity);
    // Check that all entities respect boundary
    for (i, entity) in entities.iter().enumerate() {
               "Entity {} should respect boundary", i);
        // All should have had their velocity corrected
        if transform.translation.x >= config.physics.max_world_coord - 1.0 {
            assert!(velocity.linvel.x <= 0.0, 
                   "Entity {} velocity should be corrected at boundary", i);
