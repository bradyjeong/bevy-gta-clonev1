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

        // Manhattan-realistic block dimensions
        let avenue_spacing = 260.0; // North-South avenues (wide spacing)
        let street_spacing = 80.0; // East-West streets (narrow spacing)

        info!(
            "Generating Manhattan grid for island at ({}, {}) with avenue_spacing={}m, street_spacing={}m",
            grid_x, grid_z, avenue_spacing, street_spacing
        );

        // Generate VERTICAL roads (North-South avenues) - wide MainStreet (34m)
        let mut x = min_x;
        let mut road_count = 0;
        while x <= max_x {
            let y = env.land_elevation + RoadType::MainStreet.height();
            let start = Vec3::new(x, y, min_z);
            let end = Vec3::new(x, y, max_z);

            let road_id = road_network.add_road(start, end, RoadType::MainStreet);
            road_ids.push(road_id);
            road_count += 1;

            x += avenue_spacing;
        }

        info!("Generated {} vertical avenues", road_count);

        // Generate HORIZONTAL roads (East-West streets) - narrow SideStreet (19m)
        let mut z = min_z;
        road_count = 0;
        while z <= max_z {
            let y = env.land_elevation + RoadType::SideStreet.height();
            let start = Vec3::new(min_x, y, z);
            let end = Vec3::new(max_x, y, z);

            let road_id = road_network.add_road(start, end, RoadType::SideStreet);
            road_ids.push(road_id);
            road_count += 1;

            z += street_spacing;
        }

        info!(
            "✓ Manhattan grid generation complete: {} vertical avenues + {} horizontal streets = {} total roads for grid island at ({}, {})",
            ((max_x - min_x) / avenue_spacing + 1.0) as u32,
            ((max_z - min_z) / street_spacing + 1.0) as u32,
            road_ids.len(),
            grid_x,
            grid_z
        );

        road_ids
    }
}
