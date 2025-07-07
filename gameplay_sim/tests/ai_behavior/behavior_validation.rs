// AI behavior validation tests using property-based testing
use proptest::prelude::*;
use bevy::prelude::*;
use game_core::prelude::*;
use crate::utils::*;
use crate::tests::ai_behavior::emotional_state::{HumanEmotions, Mood};

// Property-based test strategies for AI behavior
fn behavior_strategy() -> impl Strategy<Value = HumanBehavior> {
    (
        0.1f32..2.0f32,  // personality_speed_modifier
        0.01f32..1.0f32, // reaction_time
        0.0f32..1.0f32,  // confidence_level
        0.5f32..2.0f32,  // movement_variation
    ).prop_map(|(speed, reaction, confidence, variation)| {
        HumanBehavior {
            personality_speed_modifier: speed,
            reaction_time: reaction,
            confidence_level: confidence,
            movement_variation: variation,
        }
    })
}
fn emotions_strategy() -> impl Strategy<Value = HumanEmotions> {
        0.0f32..100.0f32, // stress_level
        0.0f32..100.0f32, // energy_level
        prop::sample::select(vec![
            Mood::Calm,
            Mood::Excited,
            Mood::Tired,
            Mood::Anxious,
            Mood::Confident,
        ]),
        0.0f32..100.0f32, // last_mood_change
    ).prop_map(|(stress, energy, mood, last_change)| {
        HumanEmotions {
            stress_level: stress,
            energy_level: energy,
            mood,
            last_mood_change: last_change,
fn movement_strategy() -> impl Strategy<Value = HumanMovement> {
        0.5f32..10.0f32, // base_speed
        1.0f32..15.0f32, // run_speed
        0.1f32..1.0f32,  // tired_speed_modifier
    ).prop_map(|(base, run, tired)| {
        HumanMovement {
            base_speed: base,
            run_speed: run.max(base), // Ensure run_speed >= base_speed
            tired_speed_modifier: tired,
proptest! {
    #[test]
    fn test_behavior_parameters_stay_in_bounds(
        behavior in behavior_strategy()
    ) {
        // Behavior parameters should stay within reasonable bounds
        prop_assert!(behavior.personality_speed_modifier > 0.0);
        prop_assert!(behavior.personality_speed_modifier < 5.0);
        prop_assert!(behavior.reaction_time > 0.0);
        prop_assert!(behavior.reaction_time < 2.0);
        prop_assert!(behavior.confidence_level >= 0.0);
        prop_assert!(behavior.confidence_level <= 1.0);
        prop_assert!(behavior.movement_variation > 0.0);
        prop_assert!(behavior.movement_variation < 3.0);
    }
    
    fn test_emotional_state_bounds(
        emotions in emotions_strategy()
        // Emotional states should stay within valid ranges
        prop_assert!(emotions.stress_level >= 0.0);
        prop_assert!(emotions.stress_level <= 100.0);
        prop_assert!(emotions.energy_level >= 0.0);
        prop_assert!(emotions.energy_level <= 100.0);
        prop_assert!(emotions.last_mood_change >= 0.0);
    fn test_movement_speed_relationships(
        movement in movement_strategy()
        // Movement speeds should have logical relationships
        prop_assert!(movement.base_speed > 0.0);
        prop_assert!(movement.run_speed >= movement.base_speed);
        prop_assert!(movement.tired_speed_modifier > 0.0);
        prop_assert!(movement.tired_speed_modifier <= 1.0);
        
        // Effective tired speed should be slower than base speed
        let tired_speed = movement.base_speed * movement.tired_speed_modifier;
        prop_assert!(tired_speed <= movement.base_speed);
#[test]
fn test_mood_affects_behavior_consistently() {
    let mut app = create_test_app();
    // Test each mood type
    let moods = [
        (Mood::Tired, 15.0, 10.0),      // Low energy, low stress
        (Mood::Anxious, 50.0, 80.0),   // Medium energy, high stress
        (Mood::Confident, 90.0, 20.0), // High energy, low stress
        (Mood::Excited, 70.0, 50.0),   // High energy, medium stress
        (Mood::Calm, 50.0, 30.0),      // Medium energy, low stress
    ];
    for (mood, energy, stress) in moods {
        let player_entity = {
            let mut world = app.world_mut();
            world.spawn((
                Player::default(),
                ActiveEntity,
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                HumanMovement::default(),
                HumanBehavior::default(),
                HumanAnimation::default(),
                HumanEmotions {
                    energy_level: energy,
                    stress_level: stress,
                    mood: mood.clone(),
                    last_mood_change: 0.0,
                },
            )).id()
        };
        // Run simulation to update behavior based on mood
        run_simulation_duration(&mut app, 1.0);
        let world = app.world();
        let behavior = world.get::<HumanBehavior>(player_entity).unwrap();
        let movement = world.get::<HumanMovement>(player_entity).unwrap();
        // Verify mood affects behavior in expected ways
        match mood {
            Mood::Tired => {
                assert!(behavior.personality_speed_modifier < 1.0, "Tired should be slower");
                assert!(behavior.reaction_time > 0.12, "Tired should have slower reactions");
                assert!(movement.tired_speed_modifier < 1.0, "Tired should affect movement");
            }
            Mood::Anxious => {
                assert!(behavior.personality_speed_modifier > 1.0, "Anxious should be faster");
                assert!(behavior.reaction_time < 0.1, "Anxious should have quick reactions");
                assert!(behavior.confidence_level < 0.5, "Anxious should have low confidence");
            Mood::Confident => {
                assert!(behavior.confidence_level >= 0.9, "Confident should have high confidence");
                assert!(behavior.movement_variation < 1.1, "Confident should be steady");
            Mood::Excited => {
                assert!(behavior.personality_speed_modifier > 1.0, "Excited should be faster");
                assert!(behavior.movement_variation > 1.0, "Excited should vary movement");
            Mood::Calm => {
                assert!(behavior.personality_speed_modifier == 1.0, "Calm should be baseline");
                assert!(behavior.confidence_level == 0.8, "Calm should have normal confidence");
        // Clean up for next test
        app.world_mut().despawn(player_entity);
fn test_behavior_validation_under_extreme_values() {
    // Test extreme but valid values
    let extreme_cases = [
        // (energy, stress, expected_stable)
        (0.0, 100.0, true),   // Exhausted and panicked
        (100.0, 0.0, true),   // Fully energized and calm
        (50.0, 50.0, true),   // Balanced
        (5.0, 95.0, true),    // Very tired and very stressed
        (95.0, 5.0, true),    // Very energized and very calm
    for (energy, stress, should_be_stable) in extreme_cases {
                    mood: Mood::Calm,
        // Run simulation
        run_simulation_duration(&mut app, 2.0);
        // Check that behavior remains stable and valid
        let emotions = world.get::<HumanEmotions>(player_entity).unwrap();
        if should_be_stable {
            // Behavior should be valid
            assert!(behavior.personality_speed_modifier > 0.0, "Speed modifier should be positive");
            assert!(behavior.personality_speed_modifier < 5.0, "Speed modifier should be reasonable");
            assert!(behavior.reaction_time > 0.0, "Reaction time should be positive");
            assert!(behavior.reaction_time < 1.0, "Reaction time should be reasonable");
            assert!(behavior.confidence_level >= 0.0, "Confidence should be non-negative");
            assert!(behavior.confidence_level <= 1.0, "Confidence should not exceed 1.0");
            
            // Emotions should be clamped
            assert!(emotions.energy_level >= 0.0, "Energy should be non-negative");
            assert!(emotions.energy_level <= 100.0, "Energy should not exceed 100");
            assert!(emotions.stress_level >= 0.0, "Stress should be non-negative");
            assert!(emotions.stress_level <= 100.0, "Stress should not exceed 100");
        // Clean up
fn test_behavior_consistency_across_updates() {
    // Create NPC with specific behavior
    let npc_entity = {
        let mut world = app.world_mut();
        world.spawn((
            Player::default(),
            ActiveEntity,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            HumanMovement::default(),
            HumanBehavior {
                personality_speed_modifier: 1.2,
                reaction_time: 0.08,
                confidence_level: 0.7,
                movement_variation: 1.1,
            },
            HumanAnimation::default(),
            HumanEmotions {
                energy_level: 60.0,
                stress_level: 40.0,
                mood: Mood::Calm,
                last_mood_change: 0.0,
        )).id()
    };
    // Record behavior over multiple simulation steps
    let mut behavior_history = Vec::new();
    for _ in 0..60 { // 1 second at 60 FPS
        let behavior = world.get::<HumanBehavior>(npc_entity).unwrap().clone();
        behavior_history.push(behavior);
        app.update();
    // Check for reasonable consistency
    let initial_behavior = &behavior_history[0];
    let final_behavior = &behavior_history[behavior_history.len() - 1];
    // Core personality traits should remain relatively stable
    let speed_diff = (final_behavior.personality_speed_modifier - initial_behavior.personality_speed_modifier).abs();
    assert!(speed_diff < 0.5, "Personality speed shouldn't change drastically");
    let confidence_diff = (final_behavior.confidence_level - initial_behavior.confidence_level).abs();
    assert!(confidence_diff < 0.3, "Confidence shouldn't change drastically");
    // All values should remain within valid ranges throughout
    for behavior in &behavior_history {
        assert!(behavior.personality_speed_modifier > 0.0, "Speed modifier should stay positive");
        assert!(behavior.reaction_time > 0.0, "Reaction time should stay positive");
        assert!(behavior.confidence_level >= 0.0 && behavior.confidence_level <= 1.0, 
               "Confidence should stay in 0-1 range");
        assert!(behavior.movement_variation > 0.0, "Movement variation should stay positive");
fn test_behavior_response_to_activity() {
    // Test different activity states
    let activity_tests = [
        (true, false, "running"),      // Running
        (false, true, "walking"),      // Walking
        (false, false, "idle"),        // Idle
    for (is_running, is_walking, activity_name) in activity_tests {
                HumanAnimation {
                    is_running,
                    is_walking,
                    idle_fidget_timer: 0.0,
                    next_fidget_time: 5.0,
                    energy_level: 80.0,
                    stress_level: 20.0,
        let initial_energy = 80.0;
        let initial_stress = 20.0;
        run_simulation_duration(&mut app, 3.0);
        // Check activity effects
        match activity_name {
            "running" => {
                assert!(emotions.energy_level < initial_energy, "Running should drain energy");
                assert!(emotions.stress_level > initial_stress, "Running should increase stress");
            "walking" => {
                assert!(emotions.energy_level < initial_energy, "Walking should drain some energy");
                // Stress might increase slightly or stay same
                assert!(emotions.stress_level >= initial_stress - 5.0, "Walking shouldn't reduce stress much");
            "idle" => {
                assert!(emotions.energy_level >= initial_energy, "Idle should maintain or increase energy");
                assert!(emotions.stress_level <= initial_stress, "Idle should maintain or reduce stress");
            _ => {}
fn test_invalid_behavior_correction() {
    // Create entity with invalid behavior values
    let player_entity = {
            HumanMovement {
                base_speed: -1.0,     // Invalid
                run_speed: 0.0,       // Invalid
                tired_speed_modifier: 2.0, // Invalid
                personality_speed_modifier: -0.5, // Invalid
                reaction_time: 0.0,               // Invalid
                confidence_level: 1.5,            // Invalid
                movement_variation: -1.0,         // Invalid
                energy_level: -50.0,  // Invalid
                stress_level: 150.0,  // Invalid
                last_mood_change: -10.0, // Invalid
    // Run simulation - validation should correct invalid values
    run_simulation_duration(&mut app, 1.0);
    // Check that values were corrected
    let world = app.world();
    let movement = world.get::<HumanMovement>(player_entity).unwrap();
    let behavior = world.get::<HumanBehavior>(player_entity).unwrap();
    let emotions = world.get::<HumanEmotions>(player_entity).unwrap();
    // All values should now be valid
    assert!(movement.base_speed > 0.0, "Base speed should be positive");
    assert!(movement.run_speed > 0.0, "Run speed should be positive");
    assert!(movement.tired_speed_modifier > 0.0 && movement.tired_speed_modifier <= 1.0, 
           "Tired speed modifier should be 0-1");
    assert!(behavior.personality_speed_modifier > 0.0, "Speed modifier should be positive");
    assert!(behavior.reaction_time > 0.0, "Reaction time should be positive");
    assert!(behavior.confidence_level >= 0.0 && behavior.confidence_level <= 1.0, 
           "Confidence should be 0-1");
    assert!(behavior.movement_variation > 0.0, "Movement variation should be positive");
    assert!(emotions.energy_level >= 0.0 && emotions.energy_level <= 100.0, 
           "Energy should be 0-100");
    assert!(emotions.stress_level >= 0.0 && emotions.stress_level <= 100.0, 
           "Stress should be 0-100");
    assert!(emotions.last_mood_change >= 0.0, "Last mood change should be non-negative");
