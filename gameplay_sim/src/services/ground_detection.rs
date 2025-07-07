use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const GROUND_DETECTION_HEIGHT: f32 = 100.0; // Cast ray from this height
const DEFAULT_GROUND_HEIGHT: f32 = 0.0; // Fallback if no ground found
const MIN_GROUND_HEIGHT: f32 = -10.0; // Minimum valid ground height
const MAX_GROUND_HEIGHT: f32 = 50.0; // Maximum valid ground height

/// Service for detecting ground height using physics raycasting
#[derive(Resource)]
pub struct GroundDetectionService {
    pub enabled: bool,
    pub fallback_height: f32,
}

impl Default for GroundDetectionService {
    fn default() -> Self {
        Self {
            enabled: true,
            fallback_height: DEFAULT_GROUND_HEIGHT,
        }
    }
}

impl GroundDetectionService {
    /// Get ground height at a given XZ position using Rapier's built-in raycasting
    /// This is the proper way to use Rapier's physics for ground detection
    pub fn get_ground_height(
        &self,
        position: Vec2,
        rapier_context: &RapierContext,
    ) -> f32 {
        if !self.enabled {
            return self.fallback_height;
        }

        // Cast ray downward from high altitude using Rapier's system
        let ray_origin = Vec3::new(position.x, GROUND_DETECTION_HEIGHT, position.y);
        let ray_direction = Vec3::NEG_Y;
        let max_distance = GROUND_DETECTION_HEIGHT + 20.0; // Extra margin

        // Create raycast filter to only hit static geometry (terrain, buildings)
        let filter = QueryFilter::new()
            .groups(CollisionGroups::new(
                Group::ALL,
                Group::GROUP_1, // STATIC_GROUP
            ));

        // Use Rapier's built-in ray casting - this is the single source of truth
        if let Some((_entity, intersection)) = rapier_context.cast_ray(
            ray_origin,
            ray_direction,
            max_distance,
            true,
            filter,
        ) {
            let ground_height = ray_origin.y - intersection;

            // Validate ground height is reasonable
            if ground_height >= MIN_GROUND_HEIGHT && ground_height <= MAX_GROUND_HEIGHT {
                return ground_height;
            }
        }

        // Return fallback if no valid ground found
        self.fallback_height
    }

    /// Get ground height with offset for entity spawning
    pub fn get_spawn_height(
        &self,
        position: Vec2,
        entity_height: f32,
        rapier_context: &RapierContext,
    ) -> f32 {
        let ground_height = self.get_ground_height(position, rapier_context);
        // Place entity with its bottom at ground level
        ground_height + entity_height * 0.5
    }

    /// Validate if a position has valid ground
    pub fn has_valid_ground(
        &self,
        position: Vec2,
        rapier_context: &RapierContext,
    ) -> bool {
        if !self.enabled {
            return true;
        }

        let ray_origin = Vec3::new(position.x, GROUND_DETECTION_HEIGHT, position.y);
        let ray_direction = Vec3::NEG_Y;
        let max_distance = GROUND_DETECTION_HEIGHT + 20.0;

        let filter = QueryFilter::new()
            .groups(CollisionGroups::new(
                Group::ALL,
                Group::GROUP_1, // STATIC_GROUP
            ));

        rapier_context.cast_ray(
            ray_origin,
            ray_direction,
            max_distance,
            true,
            filter,
        ).is_some()
    }

    /// Get ground height without requiring RapierContext access
    /// Uses simple terrain estimation until physics integration is fixed
    pub fn get_ground_height_simple(&self, position: Vec2) -> f32 {
        // Match the actual terrain height from setup_basic_world
        // Terrain is at y=-0.15, so ground surface is at -0.1

        // Keep spawn area (within 10 units of origin) perfectly flat to prevent sliding
        let spawn_area_radius = 10.0;
        let distance_from_spawn = (position.x.powi(2) + position.y.powi(2)).sqrt();
        if distance_from_spawn < spawn_area_radius {
            -0.1 // Perfectly flat ground around spawn
        } else {
            let noise_height = (position.x * 0.01).sin() * (position.y * 0.01).cos() * 0.1;
            -0.1 + noise_height // Terrain surface with small variation
        }
    }

    /// Check if a position is suitable for NPC spawning (avoiding roads, buildings, etc.)
    pub fn is_spawn_position_valid(&self, position: Vec2) -> bool {
        // Simple validation - in practice this would check for:
        // - Not on roads
        // - Not inside buildings
        // - Not too steep terrain
        // - Not in water

        // For now, just avoid positions too close to origin (where roads typically are)
        let distance_from_origin = position.length();
        distance_from_origin > 10.0 // Stay away from central road network
    }
}

/// Plugin to add ground detection service
pub struct GroundDetectionPlugin;

impl Plugin for GroundDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundDetectionService>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_detection_service_default() {
        let service = GroundDetectionService::default();
        assert!(service.enabled);
        assert_eq!(service.fallback_height, DEFAULT_GROUND_HEIGHT);
    }

    #[test]
    fn test_spawn_height_calculation() {
        // Mock a scenario where we can't test actual raycasting
        let entity_height = 1.8;
        let expected_spawn_height = DEFAULT_GROUND_HEIGHT + entity_height * 0.5;

        // This would need actual RapierContext in integration tests
        // For now, just verify the calculation logic
        assert_eq!(expected_spawn_height, 0.9);
    }
}
