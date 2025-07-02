use bevy::prelude::*;
use gta_game::systems::input::vehicle_control_config::{
    VehicleControlConfig, VehicleType, ControlCategory, InputAction
};
use gta_game::game_state::GameState;

fn main() {
    // Create the vehicle control config
    let config = VehicleControlConfig::default();
    
    println!("=== Vehicle Control Config Demo ===\n");
    
    // Test different vehicle types
    test_vehicle_controls(&config, VehicleType::Walking);
    test_vehicle_controls(&config, VehicleType::Car);
    test_vehicle_controls(&config, VehicleType::SuperCar);
    test_vehicle_controls(&config, VehicleType::Helicopter);
    test_vehicle_controls(&config, VehicleType::F16);
    
    // Test game state conversion
    println!("=== Game State Conversion ===");
    println!("Walking -> {:?}", VehicleControlConfig::game_state_to_vehicle_type(&GameState::Walking));
    println!("Driving -> {:?}", VehicleControlConfig::game_state_to_vehicle_type(&GameState::Driving));
    println!("Flying -> {:?}", VehicleControlConfig::game_state_to_vehicle_type(&GameState::Flying));
    println!("Jetting -> {:?}", VehicleControlConfig::game_state_to_vehicle_type(&GameState::Jetting));
    
    // Test control help generation
    println!("\n=== F16 Control Help ===");
    let help = config.get_control_help(VehicleType::F16);
    for (category, controls) in help {
        println!("{:?} Controls:", category);
        for control in controls {
            println!("  {}", control);
        }
        println!();
    }
}

fn test_vehicle_controls(config: &VehicleControlConfig, vehicle_type: VehicleType) {
    println!("=== {:?} Controls ===", vehicle_type);
    
    if let Some(controls) = config.get_vehicle_controls(vehicle_type) {
        println!("Total controls: {}", controls.len());
        
        // Show controls by category
        for category in [ControlCategory::Primary, ControlCategory::Secondary, ControlCategory::Meta] {
            let category_controls = config.get_vehicle_controls_by_category(vehicle_type, category);
            if !category_controls.is_empty() {
                println!("{:?} ({} controls):", category, category_controls.len());
                for control in category_controls {
                    println!("  {:?}: {} - {}", control.key, control.action, control.description);
                }
            }
        }
        
        // Test specific action lookup
        if let Some(key) = config.get_key_for_vehicle_action(vehicle_type, InputAction::Forward) {
            println!("Forward action key: {:?}", key);
        }
        
        // Show available actions
        let actions = config.get_available_actions(vehicle_type);
        println!("Available actions: {:?}", actions);
    } else {
        println!("No controls found for {:?}", vehicle_type);
    }
    
    println!();
}
