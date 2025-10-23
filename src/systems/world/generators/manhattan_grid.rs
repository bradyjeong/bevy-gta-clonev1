use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;
use crate::systems::world::road_network::{RoadNetwork, RoadType};
use bevy::prelude::*;

/// Simple, data-driven Manhattan grid generator
/// Creates straight roads in a grid pattern for the grid island
#[derive(Default)]
pub struct ManhattanGridGenerator {
    /// Has the grid been generated yet?
    generated: bool,
}

impl ManhattanGridGenerator {
    /// Generate the entire Manhattan grid for the grid island
    /// This is called ONCE, not per-chunk
    pub fn generate_grid(
        &mut self,
        road_network: &mut RoadNetwork,
        _config: &GameConfig,
        env: &WorldEnvConfig,
    ) -> Vec<u64> {
        // Only generate once
        if self.generated {
            return Vec::new();
        }
        self.generated = true;

        let mut road_ids = Vec::new();

        // Grid island boundaries
        let grid_x = env.islands.grid_x; // 0.0
        let grid_z = env.islands.grid_z; // 1800.0
        let half_size = env.terrain.half_size; // 600.0

        let min_x = grid_x - half_size; // -600
        let max_x = grid_x + half_size; // 600
        let min_z = grid_z - half_size; // 1200
        let max_z = grid_z + half_size; // 2400

        // Block size (configurable - makes 12x12 blocks for 1200m island)
        let block_size = 100.0;

        let road_y = env.land_elevation + RoadType::MainStreet.height();

        info!(
            "Generating Manhattan grid for island at ({}, {}) with block_size={}m",
            grid_x, grid_z, block_size
        );

        // Generate VERTICAL roads (North-South)
        let mut x = min_x;
        let mut road_count = 0;
        while x <= max_x {
            let start = Vec3::new(x, road_y, min_z);
            let end = Vec3::new(x, road_y, max_z);

            let road_id = road_network.add_road(start, end, RoadType::MainStreet);
            road_ids.push(road_id);
            road_count += 1;

            x += block_size;
        }

        info!("Generated {} vertical roads", road_count);

        // Generate HORIZONTAL roads (East-West)
        let mut z = min_z;
        road_count = 0;
        while z <= max_z {
            let start = Vec3::new(min_x, road_y, z);
            let end = Vec3::new(max_x, road_y, z);

            let road_id = road_network.add_road(start, end, RoadType::MainStreet);
            road_ids.push(road_id);
            road_count += 1;

            z += block_size;
        }

        info!(
            "Generated {} horizontal roads - Total {} Manhattan grid roads",
            road_count,
            road_ids.len()
        );

        road_ids
    }
}
