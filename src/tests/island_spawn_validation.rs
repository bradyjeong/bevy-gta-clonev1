use crate::constants::WorldEnvConfig;
use bevy::prelude::*;

/// Test that player spawn position is within island boundaries
#[test]
fn test_player_spawn_within_island() {
    let env = WorldEnvConfig::default();
    let player_spawn = Vec3::new(env.islands.left_x, env.land_elevation + 10.0, 0.0);

    // Check player is within left island rectangular bounds
    assert!(
        player_spawn.x >= env.islands.left_x - env.terrain.half_size
            && player_spawn.x <= env.islands.left_x + env.terrain.half_size
            && player_spawn.z >= -env.terrain.half_size
            && player_spawn.z <= env.terrain.half_size,
        "Player spawn at ({}, {}) is outside left island bounds",
        player_spawn.x,
        player_spawn.z
    );
}

/// Test that all three island boundaries are correctly defined and don't overlap
#[test]
fn test_island_boundaries() {
    let env = WorldEnvConfig::default();

    // Left island: X ∈ [-2100, -900], Z ∈ [-600, 600]
    let left_min_x = env.islands.left_x - env.terrain.half_size;
    let left_max_x = env.islands.left_x + env.terrain.half_size;

    // Right island: X ∈ [900, 2100], Z ∈ [-600, 600]
    let right_min_x = env.islands.right_x - env.terrain.half_size;
    let right_max_x = env.islands.right_x + env.terrain.half_size;

    // Grid island: X ∈ [-600, 600], Z ∈ [1200, 2400]
    let grid_min_x = env.islands.grid_x - env.terrain.half_size;
    let grid_max_x = env.islands.grid_x + env.terrain.half_size;
    let grid_min_z = env.islands.grid_z - env.terrain.half_size;
    let _grid_max_z = env.islands.grid_z + env.terrain.half_size;

    // Verify islands don't overlap in X-Z plane
    assert!(
        left_max_x < right_min_x,
        "Left and right islands overlap in X axis"
    );
    assert!(
        left_max_x < grid_min_x || left_min_x > grid_max_x || grid_min_z > env.terrain.half_size,
        "Left and grid islands overlap"
    );
    assert!(
        right_min_x > grid_max_x || right_max_x < grid_min_x || grid_min_z > env.terrain.half_size,
        "Right and grid islands overlap"
    );
}

/// Test that the old test name still works
#[test]
fn test_old_island_boundaries() {
    let env = WorldEnvConfig::default();

    // Test positions on left island
    let left_positions = vec![
        Vec3::new(env.islands.left_x, 0.0, 0.0),         // Center
        Vec3::new(env.islands.left_x + 500.0, 0.0, 0.0), // East side
        Vec3::new(env.islands.left_x - 500.0, 0.0, 0.0), // West side
        Vec3::new(env.islands.left_x, 0.0, 500.0),       // North side
        Vec3::new(env.islands.left_x, 0.0, -500.0),      // South side
    ];

    for pos in left_positions {
        let on_left_island = pos.x >= env.islands.left_x - env.terrain.half_size
            && pos.x <= env.islands.left_x + env.terrain.half_size
            && pos.z >= -env.terrain.half_size
            && pos.z <= env.terrain.half_size;

        assert!(
            on_left_island,
            "Position ({}, {}) should be on left island",
            pos.x, pos.z
        );
    }
}

/// Test that rectangular islands work correctly
#[test]
fn test_rectangular_island_shape() {
    let env = WorldEnvConfig::default();

    // Test corners of left island (should be within bounds)
    let left_corners = vec![
        Vec3::new(
            env.islands.left_x - env.terrain.half_size + 1.0,
            0.0,
            -env.terrain.half_size + 1.0,
        ),
        Vec3::new(
            env.islands.left_x + env.terrain.half_size - 1.0,
            0.0,
            -env.terrain.half_size + 1.0,
        ),
        Vec3::new(
            env.islands.left_x - env.terrain.half_size + 1.0,
            0.0,
            env.terrain.half_size - 1.0,
        ),
        Vec3::new(
            env.islands.left_x + env.terrain.half_size - 1.0,
            0.0,
            env.terrain.half_size - 1.0,
        ),
    ];

    for pos in left_corners {
        let on_island = pos.x >= env.islands.left_x - env.terrain.half_size
            && pos.x <= env.islands.left_x + env.terrain.half_size
            && pos.z >= -env.terrain.half_size
            && pos.z <= env.terrain.half_size;

        assert!(
            on_island,
            "Corner ({}, {}) should be on island",
            pos.x, pos.z
        );
    }
}

/// Test that ocean positions are correctly excluded
#[test]
fn test_ocean_exclusion() {
    let env = WorldEnvConfig::default();

    let ocean_positions = vec![
        Vec3::new(0.0, 0.0, 0.0),                          // Between islands
        Vec3::new(env.islands.left_x - 1000.0, 0.0, 0.0),  // Far west
        Vec3::new(env.islands.right_x + 1000.0, 0.0, 0.0), // Far east
        Vec3::new(env.islands.left_x, 0.0, 1000.0),        // Far north
    ];

    for pos in ocean_positions {
        let on_left = pos.x >= env.islands.left_x - env.terrain.half_size
            && pos.x <= env.islands.left_x + env.terrain.half_size
            && pos.z >= -env.terrain.half_size
            && pos.z <= env.terrain.half_size;

        let on_right = pos.x >= env.islands.right_x - env.terrain.half_size
            && pos.x <= env.islands.right_x + env.terrain.half_size
            && pos.z >= -env.terrain.half_size
            && pos.z <= env.terrain.half_size;

        assert!(
            !on_left && !on_right,
            "Ocean position ({}, {}) incorrectly detected as on island",
            pos.x,
            pos.z
        );
    }
}

/// Test symmetric island layout
#[test]
fn test_island_symmetry() {
    let env = WorldEnvConfig::default();

    assert_eq!(
        -env.islands.left_x, env.islands.right_x,
        "Islands should be symmetrically positioned"
    );

    // Test equivalent positions on both islands
    let offset = 500.0;
    let left_test = Vec3::new(env.islands.left_x + offset, 0.0, 200.0);
    let right_test = Vec3::new(env.islands.right_x + offset, 0.0, 200.0);

    let dx_left = left_test.x - env.islands.left_x;
    let dz_left = left_test.z;
    let left_dist = (dx_left * dx_left + dz_left * dz_left).sqrt();

    let dx_right = right_test.x - env.islands.right_x;
    let dz_right = right_test.z;
    let right_dist = (dx_right * dx_right + dz_right * dz_right).sqrt();

    assert_eq!(
        left_dist, right_dist,
        "Symmetric positions should have equal distances from island centers"
    );
}
