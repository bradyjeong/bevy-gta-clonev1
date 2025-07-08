#![cfg(feature = "heavy_tests")]
// NPC movement pattern tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use crate::utils::*;

#[test]
fn test_npc_spawning_and_movement() {
    let mut app = create_test_app();
    
    // Create NPCs at various distances
    let npc_entities: Vec<Entity> = (0..5).map(|i| {
        let mut world = app.world_mut();
        world.spawn((
            // NPC marker (using Player component as placeholder for NPC)
            Player::default(),
            ActiveEntity,
            Transform::from_xyz(i as f32 * 10.0, 0.0, 0.0),
            GlobalTransform::default(),
            Velocity::default(),
            RigidBody::Dynamic,
            Collider::capsule_y(0.5, 0.5),
            HumanMovement {
                base_speed: 2.0,
                run_speed: 5.0,
                tired_speed_modifier: 1.0,
            },
            HumanBehavior {
                personality_speed_modifier: 0.8 + (i as f32 * 0.1), // Varying speeds
                reaction_time: 0.1,
                confidence_level: 0.8,
                movement_variation: 1.0,
            HumanAnimation::default(),
        )).id()
    }).collect();
    // Run simulation
    run_simulation_duration(&mut app, 2.0);
    // Check that NPCs exist and have reasonable states
    let world = app.world();
    for npc in &npc_entities {
        let transform = world.get::<Transform>(*npc).unwrap();
        let velocity = world.get::<Velocity>(*npc).unwrap();
        let behavior = world.get::<HumanBehavior>(*npc).unwrap();
        
        // NPCs should have valid positions
        assert!(transform.translation.is_finite(), "NPC position should be finite");
        // NPCs should have reasonable behavior parameters
        assert!(behavior.personality_speed_modifier > 0.0, "Speed modifier should be positive");
        assert!(behavior.reaction_time > 0.0, "Reaction time should be positive");
        assert!(behavior.confidence_level >= 0.0 && behavior.confidence_level <= 1.0, 
               "Confidence should be 0-1");
        // Velocity should be finite
        assert!(velocity.linvel.is_finite(), "NPC velocity should be finite");
    }
}
fn test_npc_movement_variation() {
    // Create NPCs with different movement characteristics
    let slow_npc = {
            Transform::from_xyz(0.0, 0.0, 0.0),
                base_speed: 1.0, // Slow
                run_speed: 2.0,
                personality_speed_modifier: 0.5, // Very slow personality
                reaction_time: 0.2,              // Slow reactions
                confidence_level: 0.3,           // Low confidence
                movement_variation: 1.5,         // High variation
    };
    let fast_npc = {
            Transform::from_xyz(5.0, 0.0, 0.0),
                base_speed: 3.0, // Fast
                run_speed: 7.0,
                personality_speed_modifier: 1.5, // Fast personality
                reaction_time: 0.05,             // Quick reactions
                confidence_level: 0.9,           // High confidence
                movement_variation: 0.8,         // Low variation
    // Give both NPCs some initial movement
    {
        if let Some(mut velocity) = world.get_mut::<Velocity>(slow_npc) {
            velocity.linvel = Vec3::new(0.0, 0.0, 1.0);
        }
        if let Some(mut velocity) = world.get_mut::<Velocity>(fast_npc) {
    // Check movement differences
    let slow_behavior = world.get::<HumanBehavior>(slow_npc).unwrap();
    let fast_behavior = world.get::<HumanBehavior>(fast_npc).unwrap();
    let slow_movement = world.get::<HumanMovement>(slow_npc).unwrap();
    let fast_movement = world.get::<HumanMovement>(fast_npc).unwrap();
    // Verify behavioral differences
    assert!(fast_behavior.personality_speed_modifier > slow_behavior.personality_speed_modifier,
           "Fast NPC should have higher speed modifier");
    assert!(fast_behavior.reaction_time < slow_behavior.reaction_time,
           "Fast NPC should have quicker reactions");
    assert!(fast_behavior.confidence_level > slow_behavior.confidence_level,
           "Fast NPC should have higher confidence");
    // Verify movement speed differences
    assert!(fast_movement.base_speed > slow_movement.base_speed,
           "Fast NPC should have higher base speed");
fn test_npc_collision_avoidance() {
    // Create NPCs on collision course
    let npc1 = {
            Velocity {
                linvel: Vec3::new(0.0, 0.0, 2.0), // Moving forward
                angvel: Vec3::ZERO,
            HumanMovement::default(),
            HumanBehavior::default(),
    let npc2 = {
            Transform::from_xyz(0.0, 0.0, 10.0),
                linvel: Vec3::new(0.0, 0.0, -2.0), // Moving backward (toward npc1)
    run_simulation_duration(&mut app, 3.0);
    // Check that NPCs handled potential collision
    let transform1 = world.get::<Transform>(npc1).unwrap();
    let transform2 = world.get::<Transform>(npc2).unwrap();
    let velocity1 = world.get::<Velocity>(npc1).unwrap();
    let velocity2 = world.get::<Velocity>(npc2).unwrap();
    let distance = (transform1.translation - transform2.translation).length();
    // NPCs should maintain reasonable separation
    assert!(distance > 0.5, "NPCs should not overlap");
    // Velocities should be reasonable
    assert!(velocity1.linvel.is_finite(), "NPC1 velocity should be finite");
    assert!(velocity2.linvel.is_finite(), "NPC2 velocity should be finite");
    assert!(velocity1.linvel.length() < 10.0, "NPC1 velocity should be reasonable");
    assert!(velocity2.linvel.length() < 10.0, "NPC2 velocity should be reasonable");
fn test_npc_pathfinding_behavior() {
    // Create NPC with target destination (simulated)
    let npc_entity = {
                personality_speed_modifier: 1.0,
    // Simulate NPC having a goal (move toward a target)
    let target_position = Vec3::new(10.0, 0.0, 10.0);
    // Give NPC initial direction toward target
        if let Some(mut velocity) = world.get_mut::<Velocity>(npc_entity) {
            let direction = (target_position - Vec3::ZERO).normalize();
            velocity.linvel = direction * 2.0; // Move toward target
    let initial_position = Vec3::ZERO;
    // Check that NPC moved toward target
    let final_transform = world.get::<Transform>(npc_entity).unwrap();
    let initial_distance = (target_position - initial_position).length();
    let final_distance = (target_position - final_transform.translation).length();
    assert!(final_distance < initial_distance, "NPC should move closer to target");
    assert!(final_transform.translation != initial_position, "NPC should have moved");
fn test_npc_group_behavior() {
    // Create a group of NPCs
    let group_size = 4;
    let npcs: Vec<Entity> = (0..group_size).map(|i| {
        let angle = (i as f32 / group_size as f32) * std::f32::consts::TAU;
        let radius = 3.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
            Transform::from_xyz(x, 0.0, z),
                personality_speed_modifier: 0.9 + (i as f32 * 0.05), // Slight variation
                reaction_time: 0.08 + (i as f32 * 0.01),
                confidence_level: 0.7 + (i as f32 * 0.05),
                movement_variation: 0.95 + (i as f32 * 0.025),
    // Give group a common direction
        for npc in &npcs {
            if let Some(mut velocity) = world.get_mut::<Velocity>(*npc) {
                velocity.linvel = Vec3::new(1.0, 0.0, 0.0); // Move east
            }
    // Check group cohesion
    let positions: Vec<Vec3> = npcs.iter()
        .map(|npc| world.get::<Transform>(*npc).unwrap().translation)
        .collect();
    // Calculate group center
    let group_center = positions.iter().fold(Vec3::ZERO, |acc, pos| acc + *pos) / npcs.len() as f32;
    // Check that NPCs stayed relatively close to group center
    for pos in &positions {
        let distance_from_center = (pos - &group_center).length();
        assert!(distance_from_center < 10.0, "NPCs should stay relatively close to group");
    // Check that all NPCs moved in roughly the same direction
    let velocities: Vec<Vec3> = npcs.iter()
        .map(|npc| world.get::<Velocity>(*npc).unwrap().linvel)
    for velocity in &velocities {
        if velocity.length() > 0.1 {
            let direction = velocity.normalize();
            // Should be moving generally eastward
            assert!(direction.x > 0.0, "NPCs should be moving in group direction");
fn test_npc_behavioral_consistency() {
    // Create NPC and test behavior consistency over time
                personality_speed_modifier: 1.2,
                reaction_time: 0.06,
                confidence_level: 0.85,
                movement_variation: 1.1,
    // Record initial behavior parameters
    let initial_behavior = {
        let world = app.world();
        world.get::<HumanBehavior>(npc_entity).unwrap().clone()
    // Run simulation for extended period
    run_simulation_duration(&mut app, 5.0);
    // Check that core personality traits remained consistent
    let final_behavior = world.get::<HumanBehavior>(npc_entity).unwrap();
    // Core personality should remain stable
    assert!(f32_equals(final_behavior.personality_speed_modifier, 
                      initial_behavior.personality_speed_modifier, 0.1),
           "Personality speed modifier should remain consistent");
    assert!(f32_equals(final_behavior.confidence_level, 
                      initial_behavior.confidence_level, 0.1),
           "Confidence level should remain relatively stable");
    // Some parameters may vary slightly due to emotional state changes
    assert!(final_behavior.reaction_time > 0.0, "Reaction time should remain positive");
    assert!(final_behavior.movement_variation > 0.0, "Movement variation should remain positive");
fn test_npc_response_to_environment() {
    // Create NPC
    // Create environmental obstacles
    let obstacle = {
            Transform::from_xyz(2.0, 0.0, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(1.0, 2.0, 1.0), // Wall
    // Give NPC movement toward obstacle
            velocity.linvel = Vec3::new(1.0, 0.0, 0.0); // Moving toward obstacle
    // Check NPC response to obstacle
    let npc_transform = world.get::<Transform>(npc_entity).unwrap();
    let npc_velocity = world.get::<Velocity>(npc_entity).unwrap();
    let obstacle_transform = world.get::<Transform>(obstacle).unwrap();
    let distance_to_obstacle = (npc_transform.translation - obstacle_transform.translation).length();
    // NPC should not have moved through the obstacle
    assert!(distance_to_obstacle > 1.0, "NPC should not pass through obstacle");
    // NPC should have reasonable velocity
    assert!(npc_velocity.linvel.is_finite(), "NPC velocity should remain finite");
    assert!(npc_velocity.linvel.length() < 10.0, "NPC velocity should be reasonable");
