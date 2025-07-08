#![cfg(feature = "heavy_tests")]
// Human behavior system tests
use bevy::prelude::*;
use game_core::prelude::*;
use crate::utils::*;
use crate::tests::ai_behavior::emotional_state::HumanEmotions;

#[test]
fn test_human_emotional_state_system() {
    let mut app = create_test_app();
    
    // Create a player with human components
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
            HumanEmotions::default(),
        )).id()
    };
    // Run simulation to update emotional state
    run_simulation_duration(&mut app, 2.0);
    // Check that emotional state was updated
    let world = app.world();
    let emotions = world.get::<HumanEmotions>(player_entity).unwrap();
    let behavior = world.get::<HumanBehavior>(player_entity).unwrap();
    // Initial state should be reasonable
    assert!(emotions.energy_level >= 0.0 && emotions.energy_level <= 100.0);
    assert!(emotions.stress_level >= 0.0 && emotions.stress_level <= 100.0);
    // Behavior should be influenced by emotional state
    assert!(behavior.personality_speed_modifier > 0.0);
    assert!(behavior.reaction_time > 0.0);
    assert!(behavior.confidence_level >= 0.0 && behavior.confidence_level <= 1.0);
}
fn test_activity_affects_energy() {
    // Create player with known initial state
            HumanAnimation {
                is_running: true, // Running drains energy
                is_walking: false,
                idle_fidget_timer: 0.0,
                next_fidget_time: 5.0,
            },
            HumanEmotions {
                energy_level: 100.0, // Start with full energy
                stress_level: 0.0,
                mood: crate::tests::ai_behavior::emotional_state::Mood::Calm,
                last_mood_change: 0.0,
    let initial_energy = 100.0;
    // Run simulation for extended period
    run_simulation_duration(&mut app, 5.0);
    // Check that running drained energy
    assert!(emotions.energy_level < initial_energy, "Running should drain energy");
    assert!(emotions.stress_level > 0.0, "Running should increase stress");
fn test_mood_changes_behavior() {
    // Test different moods and their effects
    let moods = [
        crate::tests::ai_behavior::emotional_state::Mood::Tired,
        crate::tests::ai_behavior::emotional_state::Mood::Anxious,
        crate::tests::ai_behavior::emotional_state::Mood::Confident,
        crate::tests::ai_behavior::emotional_state::Mood::Excited,
        crate::tests::ai_behavior::emotional_state::Mood::Calm,
    ];
    for mood in moods {
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
                    energy_level: match mood {
                        crate::tests::ai_behavior::emotional_state::Mood::Tired => 15.0,
                        crate::tests::ai_behavior::emotional_state::Mood::Confident => 90.0,
                        _ => 50.0,
                    },
                    stress_level: match mood {
                        crate::tests::ai_behavior::emotional_state::Mood::Anxious => 80.0,
                        crate::tests::ai_behavior::emotional_state::Mood::Calm => 10.0,
                        _ => 40.0,
                    mood: mood.clone(),
                    last_mood_change: 0.0,
                },
            )).id()
        };
        
        // Run simulation
        run_simulation_duration(&mut app, 1.0);
        // Check behavior adjustments
        let world = app.world();
        let behavior = world.get::<HumanBehavior>(player_entity).unwrap();
        let movement = world.get::<HumanMovement>(player_entity).unwrap();
        match mood {
            crate::tests::ai_behavior::emotional_state::Mood::Tired => {
                assert!(behavior.personality_speed_modifier < 1.0, "Tired should be slower");
                assert!(behavior.reaction_time > 0.15, "Tired should have slower reactions");
                assert!(movement.tired_speed_modifier < 1.0, "Tired should affect movement");
            }
            crate::tests::ai_behavior::emotional_state::Mood::Anxious => {
                assert!(behavior.personality_speed_modifier > 1.0, "Anxious should be faster");
                assert!(behavior.reaction_time < 0.1, "Anxious should have quick reactions");
                assert!(behavior.confidence_level < 0.5, "Anxious should have low confidence");
            crate::tests::ai_behavior::emotional_state::Mood::Confident => {
                assert!(behavior.confidence_level >= 1.0, "Confident should have high confidence");
                assert!(behavior.movement_variation < 1.1, "Confident should be steady");
            crate::tests::ai_behavior::emotional_state::Mood::Excited => {
                assert!(behavior.personality_speed_modifier > 1.0, "Excited should be faster");
                assert!(behavior.movement_variation > 1.0, "Excited should vary movement");
            crate::tests::ai_behavior::emotional_state::Mood::Calm => {
                assert!(behavior.personality_speed_modifier == 1.0, "Calm should be baseline speed");
                assert!(behavior.confidence_level == 0.8, "Calm should have normal confidence");
        }
    }
fn test_fidget_system() {
    // Create player in idle state
            HumanBehavior {
                confidence_level: 0.3, // Low confidence for frequent fidgeting
                personality_speed_modifier: 1.0,
                reaction_time: 0.1,
                movement_variation: 1.0,
                is_running: false,
                is_walking: false, // Idle
                next_fidget_time: 1.0, // Short fidget interval for testing
    // Run simulation beyond fidget time
    // Check that fidget timer was reset
    let animation = world.get::<HumanAnimation>(player_entity).unwrap();
    // Timer should have been reset and new fidget time set
    assert!(animation.idle_fidget_timer < 1.5, "Fidget timer should have been reset");
    assert!(animation.next_fidget_time > 0.0, "Next fidget time should be set");
fn test_confidence_affects_fidgeting() {
    // Test high confidence (should fidget less)
    let confident_player = {
                confidence_level: 0.9, // High confidence
                next_fidget_time: 1.0,
    // Test low confidence (should fidget more)
    let anxious_player = {
            Transform::from_xyz(5.0, 0.0, 0.0),
                confidence_level: 0.2, // Low confidence
    // Run simulation
    run_simulation_duration(&mut app, 3.0);
    // Check fidget frequencies
    let confident_animation = world.get::<HumanAnimation>(confident_player).unwrap();
    let anxious_animation = world.get::<HumanAnimation>(anxious_player).unwrap();
    // Confident player should have longer fidget intervals
    assert!(confident_animation.next_fidget_time >= anxious_animation.next_fidget_time,
           "Confident players should fidget less frequently");
fn test_energy_recovery_while_resting() {
    // Create tired player
                is_walking: false, // Resting
                energy_level: 20.0, // Low energy
                stress_level: 60.0,  // High stress
                mood: crate::tests::ai_behavior::emotional_state::Mood::Tired,
    let initial_energy = 20.0;
    let initial_stress = 60.0;
    // Run simulation for recovery period
    // Check that energy recovered and stress reduced
    assert!(emotions.energy_level > initial_energy, "Energy should recover while resting");
    assert!(emotions.stress_level < initial_stress, "Stress should reduce while resting");
fn test_mood_persistence() {
    // Create player with specific mood
                energy_level: 50.0,
                stress_level: 50.0,
                mood: crate::tests::ai_behavior::emotional_state::Mood::Excited,
                last_mood_change: 0.0, // Recent mood change
    // Run short simulation (less than mood persistence time)
    // Check that mood didn't change too quickly
    // Mood should persist for some time
    assert_eq!(emotions.mood, crate::tests::ai_behavior::emotional_state::Mood::Excited,
              "Mood should persist for minimum duration");
fn test_behavioral_variation() {
    // Create multiple players with different emotional states
    let mut players = Vec::new();
    for i in 0..5 {
        let player = {
                Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
                    energy_level: 20.0 + i as f32 * 20.0, // Varying energy levels
                    stress_level: i as f32 * 20.0,         // Varying stress levels
                    mood: crate::tests::ai_behavior::emotional_state::Mood::Calm,
        players.push(player);
    // Check that behaviors vary between players
    let mut speed_modifiers = Vec::new();
    let mut reaction_times = Vec::new();
    for player in &players {
        let behavior = world.get::<HumanBehavior>(*player).unwrap();
        speed_modifiers.push(behavior.personality_speed_modifier);
        reaction_times.push(behavior.reaction_time);
    // Should have variation in behaviors
    let max_speed = speed_modifiers.iter().fold(0.0f32, |a, &b| a.max(b));
    let min_speed = speed_modifiers.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    assert!(max_speed > min_speed, "Should have variation in speed modifiers");
    let max_reaction = reaction_times.iter().fold(0.0f32, |a, &b| a.max(b));
    let min_reaction = reaction_times.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    assert!(max_reaction > min_reaction, "Should have variation in reaction times");
