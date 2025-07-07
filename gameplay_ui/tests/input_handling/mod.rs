//! Tests for input handling - user input processing, menu navigation

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;
use gameplay_ui::systems::ui::bugatti_telemetry::*;
use gameplay_ui::systems::performance_monitor::*;
use crate::utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test F4 key toggle for telemetry
    #[test]
    fn test_f4_telemetry_toggle() {
        let mut app = create_ui_test_app();
        let (_player, _supercar) = setup_ui_test_scene(&mut app);
        
        // Initial state
        let state = get_telemetry_state(&app);
        assert!(!state.visible);
        // Press F4 to enable
        simulate_key_press(&mut app, KeyCode::F4);
        assert!(state.visible);
        // Press F4 again to disable
    }
    /// Test F3 key toggle for performance monitor
    fn test_f3_performance_toggle() {
        // Press F3 to toggle performance overlay
        simulate_key_press(&mut app, KeyCode::F3);
        // Check that overlay was created
        let overlay_exists = ui_element_exists::<PerformanceOverlay>(&app);
        assert!(overlay_exists, "Performance overlay should be created");
    /// Test rapid key presses
    fn test_rapid_key_presses() {
        // Rapidly press F4 multiple times
        for _ in 0..10 {
            simulate_key_press(&mut app, KeyCode::F4);
        }
        // Final state should be visible (odd number of presses)
        assert!(!state.visible); // 10 presses = even, so should be off
    /// Test key hold vs press
    fn test_key_hold_vs_press() {
        // Hold F4 for multiple frames
        simulate_key_hold(&mut app, KeyCode::F4, 60); // 1 second hold
        // Should only toggle once despite holding
    /// Test input buffering
    fn test_input_buffering() {
        // Press multiple keys in sequence
        let input_sequence = create_mock_input_sequence();
        for key in input_sequence {
            simulate_key_press(&mut app, key);
        // App should handle all inputs without crashing
        assert!(app.world().entities().len() > 0);
    /// Test input validation
    fn test_input_validation() {
        // Test invalid key codes (should not crash)
        let invalid_keys = vec![
            KeyCode::F1, KeyCode::F2, KeyCode::F5, KeyCode::F6,
            KeyCode::F7, KeyCode::F8, KeyCode::F9, KeyCode::F10,
            KeyCode::F11, KeyCode::F12
        ];
        for key in invalid_keys {
        // App should remain stable
    /// Test input priority
    fn test_input_priority() {
        // Press F3 and F4 simultaneously
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::F3);
        input.press(KeyCode::F4);
        app.update();
        // Both systems should process their inputs
        let telemetry_state = get_telemetry_state(&app);
        assert!(telemetry_state.visible || overlay_exists, 
                "At least one UI system should respond to input");
    /// Test context-sensitive input
    fn test_context_sensitive_input() {
        // Test without supercar (should not show telemetry)
        let mut world = app.world_mut();
        world.spawn((
            Player::default(),
            ActiveEntity,
            Transform::default(),
            GlobalTransform::default(),
        ));
        assert!(!state.visible, "Telemetry should not show without supercar");
        // Add supercar and test again
            Car::default(),
            SuperCar::default(),
        assert!(state.visible, "Telemetry should show with supercar");
    /// Test game state sensitivity
    fn test_game_state_sensitivity() {
        // Set game state to menu
        app.world_mut().insert_resource(NextState::Pending(GameState::Menu));
        // Enable telemetry
        // Change to driving state
        app.world_mut().insert_resource(NextState::Pending(GameState::Driving));
        // Telemetry should be active in driving state
    /// Test input debouncing
    fn test_input_debouncing() {
        // Press F4 rapidly within single frame
        input.release(KeyCode::F4);
        // Should only register one press
    /// Test input queue overflow
    fn test_input_queue_overflow() {
        // Generate excessive input events
        for _ in 0..1000 {
        // App should handle gracefully
    /// Test input during frame drops
    fn test_input_during_frame_drops() {
        // Simulate frame drops by running multiple updates per input
        for _ in 0..5 {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::F4);
            
            // Multiple updates while key is pressed
            for _ in 0..10 {
                app.update();
            }
            input.release(KeyCode::F4);
        // Should still work correctly
        assert!(state.visible); // Odd number of presses
    /// Test input system ordering
    fn test_input_system_ordering() {
        // Press multiple UI toggle keys in same frame
        // Both systems should process without conflicts
        assert!(telemetry_state.visible);
        assert!(overlay_exists);
    /// Test input persistence across frames
    fn test_input_persistence() {
        // Run many frames without input
        run_ui_updates(&mut app, 5.0);
        // State should persist
    /// Test input cleanup
    fn test_input_cleanup() {
        // Remove supercar
        let supercars: Vec<Entity> = world.query_filtered::<Entity, With<SuperCar>>()
            .iter(&world)
            .collect();
        for entity in supercars {
            world.entity_mut(entity).remove::<SuperCar>();
        // Try to toggle again - should handle gracefully
}
