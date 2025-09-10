use bevy::prelude::*;
use crate::systems::terrain_heightfield::GlobalTerrainHeights;

/// System to validate terrain scale configuration at runtime
/// This tests if our scale parameter understanding is correct
pub fn test_physics_visual_alignment_system(
    terrain_heights: Res<GlobalTerrainHeights>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Run test when F10 is pressed
    if !keys.just_pressed(KeyCode::F10) {
        return;
    }

    info!("🧪 RUNNING TERRAIN SCALE VALIDATION TEST...");
    info!("📏 Based on Rapier docs: scale parameter represents FULL SIZE of heightfield rectangle");

    let terrain = &terrain_heights.heightfield;
    
    // Log current configuration
    info!("🗺️ Current terrain config:");
    info!("   - Grid size: {}x{}", terrain.width, terrain.height);  
    info!("   - Scale (full size): {:?}", terrain.scale);
    info!("   - Expected world bounds: X=±{:.0}, Z=±{:.0}", terrain.scale.x/2.0, terrain.scale.z/2.0);

    // Test boundary positions based on our understanding
    let test_positions = [
        Vec2::ZERO,                                    // Center
        Vec2::new(100.0, 100.0),                      // Near center
        Vec2::new(2048.0, 2048.0),                    // Expected corner
        Vec2::new(-2048.0, -2048.0),                  // Expected opposite corner
        Vec2::new(2048.0, 0.0),                       // Expected edge
        Vec2::new(0.0, 2048.0),                       // Expected edge
        Vec2::new(2100.0, 0.0),                       // Outside expected bounds
        Vec2::new(0.0, 2100.0),                       // Outside expected bounds
    ];

    let mut validation_passed = true;

    for (i, world_pos) in test_positions.iter().enumerate() {
        // Test coordinate conversion
        let grid_pos = terrain.world_to_grid(*world_pos);
        let world_pos_back = terrain.grid_to_world(grid_pos);
        
        // Check if position is within expected bounds
        let within_expected_bounds = world_pos.x.abs() <= 2048.0 && world_pos.y.abs() <= 2048.0;
        let within_grid_bounds = grid_pos.x >= 0.0 && grid_pos.x <= (terrain.width - 1) as f32 &&
                                 grid_pos.y >= 0.0 && grid_pos.y <= (terrain.height - 1) as f32;

        let roundtrip_error = (*world_pos - world_pos_back).length();
        
        match i {
            0..=5 => {
                // These should be within bounds
                if !within_expected_bounds {
                    error!("❌ Position {:?} should be within bounds but isn't", world_pos);
                    validation_passed = false;
                } else if !within_grid_bounds {
                    error!("❌ Position {:?} maps to invalid grid coords {:?}", world_pos, grid_pos);
                    validation_passed = false;
                } else if roundtrip_error > 0.01 {
                    error!("❌ Position {:?} has roundtrip error {:.6}", world_pos, roundtrip_error);
                    validation_passed = false;
                } else {
                    info!("✅ Position {:?} -> grid {:?} -> world {:?} (error: {:.6})", 
                          world_pos, grid_pos, world_pos_back, roundtrip_error);
                }
            }
            6..=7 => {
                // These should be outside bounds but handled gracefully
                if within_expected_bounds {
                    warn!("⚠️ Position {:?} is outside expected bounds but within grid bounds", world_pos);
                } else {
                    info!("ℹ️ Position {:?} outside bounds -> clamped to grid {:?}", world_pos, grid_pos);
                }
            }
            _ => {}
        }
    }

    // Test exact corner calculations
    info!("🔍 Testing exact corner calculations...");
    let corners = [
        (0, 0, "bottom-left"),
        (63, 0, "bottom-right"), 
        (0, 63, "top-left"),
        (63, 63, "top-right"),
    ];

    for (grid_x, grid_z, name) in corners {
        let expected_world = terrain.grid_to_world(Vec2::new(grid_x as f32, grid_z as f32));
        info!("📍 Grid corner ({}, {}) [{}] -> World ({:.1}, {:.1})", 
              grid_x, grid_z, name, expected_world.x, expected_world.y);
    }

    if validation_passed {
        info!("🎉 TERRAIN SCALE VALIDATION PASSED");
        info!("✅ Current scale parameter ({:.0}, {:.0}, {:.0}) appears correct", 
              terrain.scale.x, terrain.scale.y, terrain.scale.z);
        info!("✅ Terrain extends from ({:.0}, {:.0}) to ({:.0}, {:.0}) as expected",
              -terrain.scale.x/2.0, -terrain.scale.z/2.0,
              terrain.scale.x/2.0, terrain.scale.z/2.0);
    } else {
        error!("❌ TERRAIN SCALE VALIDATION FAILED - Check coordinate conversion logic!");
    }
}

/// Component to mark the test marker entity
#[derive(Component)]
pub struct PhysicsTestMarker;
