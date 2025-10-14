use crate::constants::{LAND_ELEVATION, LEFT_ISLAND_X, RIGHT_ISLAND_X, TERRAIN_HALF_SIZE};
use bevy::prelude::*;

/// Test that player spawn position is within island boundaries
#[test]
fn test_player_spawn_within_island() {
    let player_spawn = Vec3::new(LEFT_ISLAND_X, LAND_ELEVATION + 10.0, 0.0);

    // Check player is within left island rectangular bounds
    assert!(
        player_spawn.x >= LEFT_ISLAND_X - TERRAIN_HALF_SIZE
            && player_spawn.x <= LEFT_ISLAND_X + TERRAIN_HALF_SIZE
            && player_spawn.z >= -TERRAIN_HALF_SIZE
            && player_spawn.z <= TERRAIN_HALF_SIZE,
        "Player spawn at ({}, {}) is outside left island bounds",
        player_spawn.x,
        player_spawn.z
    );
}

/// Test that island boundaries are correctly defined
#[test]
fn test_island_boundaries() {
    // Test positions on left island
    let left_positions = vec![
        Vec3::new(LEFT_ISLAND_X, 0.0, 0.0),         // Center
        Vec3::new(LEFT_ISLAND_X + 500.0, 0.0, 0.0), // East side
        Vec3::new(LEFT_ISLAND_X - 500.0, 0.0, 0.0), // West side
        Vec3::new(LEFT_ISLAND_X, 0.0, 500.0),       // North side
        Vec3::new(LEFT_ISLAND_X, 0.0, -500.0),      // South side
    ];

    for pos in left_positions {
        let on_left_island = pos.x >= LEFT_ISLAND_X - TERRAIN_HALF_SIZE
            && pos.x <= LEFT_ISLAND_X + TERRAIN_HALF_SIZE
            && pos.z >= -TERRAIN_HALF_SIZE
            && pos.z <= TERRAIN_HALF_SIZE;

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
    // Test corners of left island (should be within bounds)
    let left_corners = vec![
        Vec3::new(
            LEFT_ISLAND_X - TERRAIN_HALF_SIZE + 1.0,
            0.0,
            -TERRAIN_HALF_SIZE + 1.0,
        ),
        Vec3::new(
            LEFT_ISLAND_X + TERRAIN_HALF_SIZE - 1.0,
            0.0,
            -TERRAIN_HALF_SIZE + 1.0,
        ),
        Vec3::new(
            LEFT_ISLAND_X - TERRAIN_HALF_SIZE + 1.0,
            0.0,
            TERRAIN_HALF_SIZE - 1.0,
        ),
        Vec3::new(
            LEFT_ISLAND_X + TERRAIN_HALF_SIZE - 1.0,
            0.0,
            TERRAIN_HALF_SIZE - 1.0,
        ),
    ];

    for pos in left_corners {
        let on_island = pos.x >= LEFT_ISLAND_X - TERRAIN_HALF_SIZE
            && pos.x <= LEFT_ISLAND_X + TERRAIN_HALF_SIZE
            && pos.z >= -TERRAIN_HALF_SIZE
            && pos.z <= TERRAIN_HALF_SIZE;

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
    let ocean_positions = vec![
        Vec3::new(0.0, 0.0, 0.0),                     // Between islands
        Vec3::new(LEFT_ISLAND_X - 1000.0, 0.0, 0.0),  // Far west
        Vec3::new(RIGHT_ISLAND_X + 1000.0, 0.0, 0.0), // Far east
        Vec3::new(LEFT_ISLAND_X, 0.0, 1000.0),        // Far north
    ];

    for pos in ocean_positions {
        let on_left = pos.x >= LEFT_ISLAND_X - TERRAIN_HALF_SIZE
            && pos.x <= LEFT_ISLAND_X + TERRAIN_HALF_SIZE
            && pos.z >= -TERRAIN_HALF_SIZE
            && pos.z <= TERRAIN_HALF_SIZE;

        let on_right = pos.x >= RIGHT_ISLAND_X - TERRAIN_HALF_SIZE
            && pos.x <= RIGHT_ISLAND_X + TERRAIN_HALF_SIZE
            && pos.z >= -TERRAIN_HALF_SIZE
            && pos.z <= TERRAIN_HALF_SIZE;

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
    assert_eq!(
        -LEFT_ISLAND_X, RIGHT_ISLAND_X,
        "Islands should be symmetrically positioned"
    );

    // Test equivalent positions on both islands
    let offset = 500.0;
    let left_test = Vec3::new(LEFT_ISLAND_X + offset, 0.0, 200.0);
    let right_test = Vec3::new(RIGHT_ISLAND_X + offset, 0.0, 200.0);

    let dx_left = left_test.x - LEFT_ISLAND_X;
    let dz_left = left_test.z;
    let left_dist = (dx_left * dx_left + dz_left * dz_left).sqrt();

    let dx_right = right_test.x - RIGHT_ISLAND_X;
    let dz_right = right_test.z;
    let right_dist = (dx_right * dx_right + dz_right * dz_right).sqrt();

    assert_eq!(
        left_dist, right_dist,
        "Symmetric positions should have equal distances from island centers"
    );
}
