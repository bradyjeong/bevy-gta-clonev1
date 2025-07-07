use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;
use super::input_config::InputAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VehicleType {
    Walking,
    Car,
    SuperCar,
    Helicopter,
    F16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlCategory {
    Primary,    // Movement, steering
    Secondary,  // Turbo, afterburner, vertical movement
    Meta,       // Exit vehicle, debug
}

#[derive(Debug, Clone)]
pub struct ControlBinding {
    pub action: InputAction,
    pub key: KeyCode,
    pub description: String,
    pub category: ControlCategory,
}

#[derive(Debug, Clone, Resource)]
pub struct VehicleControlConfig {
    // Vehicle-specific control mappings
    vehicle_controls: HashMap<VehicleType, Vec<ControlBinding>>,
    // Quick lookup: Vehicle -> Action -> KeyCode
    lookup_cache: HashMap<VehicleType, HashMap<InputAction, KeyCode>>,
}

impl Default for VehicleControlConfig {
    fn default() -> Self {
        let mut config = VehicleControlConfig {
            vehicle_controls: HashMap::new(),
            lookup_cache: HashMap::new(),
        };

        config.setup_default_controls();
        config.rebuild_lookup_cache();
        config
    }
}

impl VehicleControlConfig {
    fn setup_default_controls(&mut self) {
        use VehicleType::*;
        // ---- Walking ----
        self.vehicle_controls.insert(Walking, vec![
            ControlBinding { action: InputAction::Forward,  key: KeyCode::ArrowUp,    description: "Walk forward".into(),  category: ControlCategory::Primary },
            ControlBinding { action: InputAction::Backward, key: KeyCode::ArrowDown,  description: "Walk backward".into(), category: ControlCategory::Primary },
            ControlBinding { action: InputAction::TurnLeft, key: KeyCode::ArrowLeft,  description: "Turn left".into(),     category: ControlCategory::Primary },
            ControlBinding { action: InputAction::TurnRight,key: KeyCode::ArrowRight, description: "Turn right".into(),    category: ControlCategory::Primary },
            ControlBinding { action: InputAction::Run,      key: KeyCode::ShiftLeft,  description: "Run / Sprint".into(),  category: ControlCategory::Secondary },
            ControlBinding { action: InputAction::Interact, key: KeyCode::KeyF,       description: "Enter vehicle".into(), category: ControlCategory::Meta },
            ControlBinding { action: InputAction::DebugInfo,key: KeyCode::F1,         description: "Toggle debug info".into(), category: ControlCategory::Meta },
            ControlBinding { action: InputAction::EmergencyReset, key: KeyCode::F2,  description: "Emergency reset".into(), category: ControlCategory::Meta },
        ]);

        // ---- Car ----
        self.vehicle_controls.insert(Car, vec![
            ControlBinding { action: InputAction::Forward,  key: KeyCode::ArrowUp,    description: "Accelerate".into(), category: ControlCategory::Primary },
            ControlBinding { action: InputAction::Backward, key: KeyCode::ArrowDown,  description: "Brake / Reverse".into(), category: ControlCategory::Primary },
            ControlBinding { action: InputAction::TurnLeft, key: KeyCode::ArrowLeft,  description: "Steer left".into(), category: ControlCategory::Primary },
            ControlBinding { action: InputAction::TurnRight,key: KeyCode::ArrowRight, description: "Steer right".into(), category: ControlCategory::Primary },
            ControlBinding { action: InputAction::Turbo,    key: KeyCode::Space,      description: "Turbo boost".into(), category: ControlCategory::Secondary },
            ControlBinding { action: InputAction::Interact, key: KeyCode::KeyF,       description: "Exit vehicle".into(), category: ControlCategory::Meta },
        ]);

        // (SuperCar, Helicopter, F16 default controls can be filled in later in the same style)
    }

    fn rebuild_lookup_cache(&mut self) {
        self.lookup_cache.clear();
        for (vehicle_type, controls) in &self.vehicle_controls {
            let mut vehicle_lookup = HashMap::new();
            for control in controls {
                vehicle_lookup.insert(control.action, control.key);
            }
            self.lookup_cache.insert(*vehicle_type, vehicle_lookup);
        }
    }

    // --- Public helpers ----------------------------------------------------
    pub fn get_key_for_vehicle_action(
        &self,
        vehicle_type: VehicleType,
        action: InputAction,
    ) -> Option<KeyCode> {
        self.lookup_cache
            .get(&vehicle_type)
            .and_then(|m| m.get(&action))
            .copied()
    }

    pub fn get_vehicle_controls(
        &self,
        vehicle_type: VehicleType,
    ) -> Option<&Vec<ControlBinding>> {
        self.vehicle_controls.get(&vehicle_type)
    }

    pub fn get_vehicle_controls_by_category(
        &self,
        vehicle_type: VehicleType,
        category: ControlCategory,
    ) -> Vec<&ControlBinding> {
        self.vehicle_controls
            .get(&vehicle_type)
            .map(|c| c.iter().filter(|b| b.category == category).collect())
            .unwrap_or_default()
    }

    pub fn get_available_actions(
        &self,
        vehicle_type: VehicleType,
    ) -> Vec<InputAction> {
        self.vehicle_controls
            .get(&vehicle_type)
            .map(|c| c.iter().map(|b| b.action).collect())
            .unwrap_or_default()
    }

    pub fn is_action_available_for_vehicle(
        &self,
        vehicle_type: VehicleType,
        action: InputAction,
    ) -> bool {
        self.lookup_cache
            .get(&vehicle_type)
            .map(|m| m.contains_key(&action))
            .unwrap_or(false)
    }

    // Convenience translation from GameState (in game_core) to VehicleType
    pub fn game_state_to_vehicle_type(state: &GameState) -> VehicleType {
        match state {
            GameState::Walking  => VehicleType::Walking,
            GameState::Driving  => VehicleType::Car,
            GameState::Flying   => VehicleType::Helicopter,
            GameState::Jetting  => VehicleType::F16,
        }
    }

    pub fn get_controls_for_state(
        &self,
        state: &GameState,
    ) -> Option<&Vec<ControlBinding>> {
        let vt = Self::game_state_to_vehicle_type(state);
        self.get_vehicle_controls(vt)
    }

    // ---- Mutation helpers ----
    pub fn update_control(
        &mut self,
        vehicle_type: VehicleType,
        action: InputAction,
        new_key: KeyCode,
    ) -> Result<(), String> {
        // Conflict detection
        if let Some(controls) = self.vehicle_controls.get(&vehicle_type) {
            if controls.iter().any(|c| c.key == new_key && c.action != action) {
                return Err(format!(
                    "Key {:?} already bound to another action for {:?}",
                    new_key, vehicle_type
                ));
            }
        }

        // Update binding
        if let Some(controls) = self.vehicle_controls.get_mut(&vehicle_type) {
            if let Some(binding) = controls.iter_mut().find(|c| c.action == action) {
                binding.key = new_key;
                self.rebuild_lookup_cache();
                return Ok(());
            }
        }
        Err(format!("Action {:?} not found for {:?}", action, vehicle_type))
    }

    pub fn add_control(
        &mut self,
        vehicle_type: VehicleType,
        binding: ControlBinding,
    ) -> Result<(), String> {
        // Conflict checks
        if let Some(existing) = self.vehicle_controls.get(&vehicle_type) {
            if existing.iter().any(|c| c.key == binding.key) {
                return Err(format!(
                    "Key {:?} already bound for {:?}",
                    binding.key, vehicle_type
                ));
            }
            if existing.iter().any(|c| c.action == binding.action) {
                return Err(format!(
                    "Action {:?} already exists for {:?}",
                    binding.action, vehicle_type
                ));
            }
        }

        self.vehicle_controls
            .entry(vehicle_type)
            .or_insert_with(Vec::new)
            .push(binding);
        self.rebuild_lookup_cache();
        Ok(())
    }

    pub fn reset_to_defaults(&mut self) {
        self.vehicle_controls.clear();
        self.setup_default_controls();
        self.rebuild_lookup_cache();
    }
}
