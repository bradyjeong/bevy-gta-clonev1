use crate::config::GameConfig;
use crate::util::safe_math::safe_lerp;
use bevy::prelude::*;
use std::collections::HashMap;

fn zigzag32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

pub fn generate_unique_road_id(cell_coord: IVec2, local_index: u16) -> u64 {
    let zx = zigzag32(cell_coord.x) as u64;
    let zy = zigzag32(cell_coord.y) as u64;
    // 20/20/16 bit packing: [x:20 bits][y:20 bits][index:16 bits]
    // Ensures all cell-based IDs are < 1<<56 to avoid collision with Manhattan IDs
    debug_assert!(
        (zx >> 20) == 0,
        "Cell X coordinate {} out of 20-bit range (max ±524287)",
        cell_coord.x
    );
    debug_assert!(
        (zy >> 20) == 0,
        "Cell Y coordinate {} out of 20-bit range (max ±524287)",
        cell_coord.y
    );
    ((zx & 0xFFFFF) << 36) | ((zy & 0xFFFFF) << 16) | (local_index as u64)
}

// NEW GTA-STYLE ROAD NETWORK SYSTEM
//
// ROAD ID ALLOCATION STRATEGY:
// - Cell-based roads (organic): use generate_unique_road_id() → packs cell coords in lower bits (0 to ~2^48)
// - Manhattan grid roads: use add_road() → sequential IDs starting from 1 << 56 (~7e16)
// - ID spaces are separated by design to prevent collision

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoadType {
    Highway,    // Wide, 4+ lanes, barriers
    MainStreet, // Medium, 2-4 lanes, intersections
    SideStreet, // Narrow, 2 lanes, residential
    Alley,      // Very narrow, 1 lane, behind buildings
}

impl RoadType {
    pub fn width(&self) -> f32 {
        match self {
            RoadType::Highway => 30.0,    // Manhattan highways
            RoadType::MainStreet => 34.0, // Manhattan avenues (wide)
            RoadType::SideStreet => 19.0, // Manhattan cross streets
            RoadType::Alley => 8.0,       // Manhattan alleys/service roads
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            RoadType::Highway => 0.04,
            RoadType::MainStreet => 0.03,
            RoadType::SideStreet => 0.02,
            RoadType::Alley => 0.01,
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            RoadType::Highway => 4,
            RoadType::MainStreet => 3,
            RoadType::SideStreet => 2,
            RoadType::Alley => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoadSpline {
    pub id: u64,
    pub control_points: Vec<Vec3>,
    pub road_type: RoadType,
    pub connections: Vec<u64>,
}

impl RoadSpline {
    pub fn new(id: u64, start: Vec3, end: Vec3, road_type: RoadType) -> Self {
        Self {
            id,
            control_points: vec![start, end],
            road_type,
            connections: Vec::new(),
        }
    }

    pub fn add_curve(&mut self, control_point: Vec3) {
        // Insert curve control point
        let len = self.control_points.len();
        if len > 1 {
            self.control_points.insert(len - 1, control_point);
        }
    }

    // Evaluate spline at parameter t (0.0 to 1.0)
    pub fn evaluate(&self, t: f32) -> Vec3 {
        if self.control_points.len() < 2 {
            return Vec3::ZERO;
        }

        if self.control_points.len() == 2 {
            // Linear interpolation for simple roads
            safe_lerp(self.control_points[0], self.control_points[1], t)
        } else {
            // Catmull-Rom spline for curved roads
            self.catmull_rom_spline(t)
        }
    }

    fn catmull_rom_spline(&self, t: f32) -> Vec3 {
        let points = &self.control_points;
        let n = points.len();

        if n < 4 {
            // Fall back to linear interpolation
            return safe_lerp(points[0], points[n - 1], t);
        }

        // Find the segment
        let segment_t = t * (n - 3) as f32;
        let segment = segment_t.floor() as usize;
        let local_t = segment_t.fract();

        let p0 = points[segment.max(0)];
        let p1 = points[(segment + 1).min(n - 1)];
        let p2 = points[(segment + 2).min(n - 1)];
        let p3 = points[(segment + 3).min(n - 1)];

        // Catmull-Rom interpolation
        let t2 = local_t * local_t;
        let t3 = t2 * local_t;

        0.5 * ((2.0 * p1)
            + (-p0 + p2) * local_t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
    }

    pub fn length(&self) -> f32 {
        let mut length = 0.0;
        let samples = 50;

        for i in 0..samples {
            let t1 = i as f32 / samples as f32;
            let t2 = (i + 1) as f32 / samples as f32;
            length += self.evaluate(t1).distance(self.evaluate(t2));
        }

        length
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IntersectionType {
    Cross,         // 4-way intersection
    TJunction,     // 3-way intersection
    Curve,         // 2-way curved connection
    HighwayOnramp, // Highway merge
}

#[derive(Debug, Clone)]
pub struct RoadIntersection {
    pub position: Vec3,
    pub connected_roads: Vec<u64>,
    pub intersection_type: IntersectionType,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellEdge {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone)]
pub struct BoundaryPoint {
    pub position: Vec3,
    pub road_id: u64,
    pub is_start: bool,
    pub edge: CellEdge,
}

#[derive(Resource)]
pub struct RoadNetwork {
    pub roads: HashMap<u64, RoadSpline>,
    pub intersections: HashMap<u32, RoadIntersection>,
    pub next_road_id: u64,
    pub next_intersection_id: u32,
    pub generated_cells: std::collections::HashSet<IVec2>,
    pub boundary_points: HashMap<IVec2, Vec<BoundaryPoint>>,
}

// NOTE: RoadOwnership removed - it was only used for streaming, which is no longer active.
// If streaming is reintroduced, add this back under a "streaming" feature flag.

impl Default for RoadNetwork {
    fn default() -> Self {
        Self {
            roads: HashMap::default(),
            intersections: HashMap::default(),
            // Start Manhattan grid IDs at 1 << 56 to avoid collision with cell-based IDs
            // generate_unique_road_id() uses packed cell coordinates in lower bits
            next_road_id: 1 << 56,
            next_intersection_id: 0,
            generated_cells: std::collections::HashSet::default(),
            boundary_points: HashMap::default(),
        }
    }
}

impl RoadNetwork {
    pub fn clear_cache(&mut self) {
        self.generated_cells.clear();
        self.boundary_points.clear();
        debug!("Road network cache cleared");
    }

    pub fn reset(&mut self) {
        self.roads.clear();
        self.intersections.clear();
        self.generated_cells.clear();
        self.boundary_points.clear();
        // Start Manhattan grid IDs at 1 << 56 to avoid collision with cell-based IDs
        // generate_unique_road_id() uses packed cell coordinates in lower bits
        self.next_road_id = 1 << 56;
        self.next_intersection_id = 0;
        debug!("Road network completely reset");
    }

    fn determine_cell_edge(
        &self,
        position: Vec3,
        cell_coord: IVec2,
        cell_size: f32,
    ) -> Option<CellEdge> {
        let base_x = cell_coord.x as f32 * cell_size;
        let base_z = cell_coord.y as f32 * cell_size;
        // Relative tolerance: 1% of cell size, clamped between 0.5m (small cells) and 5m (large cells)
        let tolerance = (0.01 * cell_size).clamp(0.5, 5.0);

        if (position.z - base_z).abs() < tolerance {
            Some(CellEdge::South)
        } else if (position.z - (base_z + cell_size)).abs() < tolerance {
            Some(CellEdge::North)
        } else if (position.x - base_x).abs() < tolerance {
            Some(CellEdge::West)
        } else if (position.x - (base_x + cell_size)).abs() < tolerance {
            Some(CellEdge::East)
        } else {
            None
        }
    }

    fn track_boundary_points(&mut self, cell_coord: IVec2, cell_size: f32, road_ids: &[u64]) {
        let mut boundary_points = Vec::new();

        for &road_id in road_ids {
            if let Some(road) = self.roads.get(&road_id) {
                if road.control_points.len() < 2 {
                    continue;
                }

                let start = road.control_points[0];
                let end = road.control_points[road.control_points.len() - 1];

                if let Some(edge) = self.determine_cell_edge(start, cell_coord, cell_size) {
                    boundary_points.push(BoundaryPoint {
                        position: start,
                        road_id,
                        is_start: true,
                        edge,
                    });
                }

                if let Some(edge) = self.determine_cell_edge(end, cell_coord, cell_size) {
                    boundary_points.push(BoundaryPoint {
                        position: end,
                        road_id,
                        is_start: false,
                        edge,
                    });
                }
            }
        }

        if !boundary_points.is_empty() {
            self.boundary_points.insert(cell_coord, boundary_points);
        }
    }

    fn connect_to_neighbors(&mut self, cell_coord: IVec2) {
        let neighbors = [
            (IVec2::new(0, 1), CellEdge::North, CellEdge::South),
            (IVec2::new(0, -1), CellEdge::South, CellEdge::North),
            (IVec2::new(1, 0), CellEdge::East, CellEdge::West),
            (IVec2::new(-1, 0), CellEdge::West, CellEdge::East),
        ];

        let current_boundaries = self
            .boundary_points
            .get(&cell_coord)
            .cloned()
            .unwrap_or_default();
        let mut connections_to_make = Vec::new();

        for (offset, our_edge, their_edge) in neighbors {
            let neighbor_coord = cell_coord + offset;
            if let Some(neighbor_boundaries) = self.boundary_points.get(&neighbor_coord).cloned() {
                for our_point in &current_boundaries {
                    if our_point.edge != our_edge {
                        continue;
                    }
                    for neighbor_point in &neighbor_boundaries {
                        if neighbor_point.edge != their_edge {
                            continue;
                        }
                        let distance = our_point.position.distance(neighbor_point.position);
                        if distance < 10.0 {
                            connections_to_make.push((
                                our_point.road_id,
                                neighbor_point.road_id,
                                distance,
                                cell_coord,
                                neighbor_coord,
                            ));
                        }
                    }
                }
            }
        }

        for (road1, road2, distance, coord1, coord2) in connections_to_make {
            self.connect_roads(road1, road2);
            debug!(
                "Connected roads {} and {} at distance {:.2}m (cells {:?} <-> {:?})",
                road1, road2, distance, coord1, coord2
            );
        }
    }
}

impl RoadNetwork {
    /// Generate Manhattan-style orthogonal grid roads for a single cell (grid island only)
    /// Creates straight N-S and E-W roads at 400m spacing without curves
    pub fn generate_grid_roads_for_cell(
        &mut self,
        cell_coord: IVec2,
        cell_size: f32,
        config: &GameConfig,
    ) -> Vec<u64> {
        // Prevent duplicate generation (same guard as organic generator)
        if self.generated_cells.contains(&cell_coord) {
            return Vec::new();
        }
        self.generated_cells.insert(cell_coord);

        let base_x = cell_coord.x as f32 * cell_size;
        let base_z = cell_coord.y as f32 * cell_size;

        // GUARD: Only generate grid roads on grid island (prevents spillover to left/right islands)
        let cell_center = Vec3::new(base_x + 0.5 * cell_size, 0.0, base_z + 0.5 * cell_size);
        if !self.is_on_grid_island(cell_center, config) {
            return Vec::new();
        }

        let mut local_index: u16 = 0;
        let mut new_roads = Vec::new();
        let y = config.world_env.land_elevation + RoadType::MainStreet.height();

        // Vertical segment (N-S) along cell center
        let v_start = Vec3::new(base_x + cell_size * 0.5, y, base_z);
        let v_end = Vec3::new(base_x + cell_size * 0.5, y, base_z + cell_size);
        if self.segment_on_island(v_start, v_end, config) {
            let road_id = generate_unique_road_id(cell_coord, local_index);
            local_index += 1;
            let road = RoadSpline::new(road_id, v_start, v_end, RoadType::MainStreet);
            self.insert_road_checked(road);
            new_roads.push(road_id);
        }

        // Horizontal segment (E-W) along cell center
        let h_start = Vec3::new(base_x, y, base_z + cell_size * 0.5);
        let h_end = Vec3::new(base_x + cell_size, y, base_z + cell_size * 0.5);
        if self.segment_on_island(h_start, h_end, config) {
            let road_id = generate_unique_road_id(cell_coord, local_index);
            let road = RoadSpline::new(road_id, h_start, h_end, RoadType::MainStreet);
            self.insert_road_checked(road);
            new_roads.push(road_id);
        }

        self.track_boundary_points(cell_coord, cell_size, &new_roads);
        self.connect_to_neighbors(cell_coord);
        new_roads
    }

    /// Generate grid roads for a chunk (wrapper for chunk-based workflow)
    pub fn generate_grid_chunk_roads(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        config: &GameConfig,
    ) -> Vec<u64> {
        let cell_coord = IVec2::new(chunk_x, chunk_z);
        self.generate_grid_roads_for_cell(cell_coord, config.world_streaming.road_cell_size, config)
    }

    pub fn add_road(&mut self, start: Vec3, end: Vec3, road_type: RoadType) -> u64 {
        let id = self.next_road_id;
        self.next_road_id += 1;

        let road = RoadSpline::new(id, start, end, road_type);
        self.roads.insert(id, road);
        id
    }

    pub fn add_curved_road(
        &mut self,
        start: Vec3,
        control: Vec3,
        end: Vec3,
        road_type: RoadType,
    ) -> u64 {
        let id = self.next_road_id;
        self.next_road_id += 1;

        let mut road = RoadSpline::new(id, start, end, road_type);
        road.add_curve(control);
        self.roads.insert(id, road);
        id
    }

    pub fn connect_roads(&mut self, road1_id: u64, road2_id: u64) {
        if let Some(road1) = self.roads.get_mut(&road1_id) {
            road1.connections.push(road2_id);
        }
        if let Some(road2) = self.roads.get_mut(&road2_id) {
            road2.connections.push(road1_id);
        }
    }

    /// Insert a road with collision detection to catch ID conflicts
    fn insert_road_checked(&mut self, road: RoadSpline) {
        let road_id = road.id;
        if let Some(prev) = self.roads.insert(road_id, road.clone()) {
            warn!(
                "⚠️ Road ID collision detected: ID {} overwrote existing road (start {:?} -> {:?})",
                road_id, prev.control_points[0], road.control_points[0]
            );
            debug_assert!(
                false,
                "Road ID collision - this indicates a bug in ID generation"
            );
        }
    }

    pub fn add_intersection(
        &mut self,
        position: Vec3,
        connected_roads: Vec<u64>,
        intersection_type: IntersectionType,
    ) -> u32 {
        let id = self.next_intersection_id;
        self.next_intersection_id += 1;

        let radius = match intersection_type {
            IntersectionType::Cross => 20.0,
            IntersectionType::TJunction => 15.0,
            IntersectionType::Curve => 12.0,
            IntersectionType::HighwayOnramp => 30.0,
        };

        let intersection = RoadIntersection {
            position,
            connected_roads,
            intersection_type,
            radius,
        };

        self.intersections.insert(id, intersection);
        id
    }

    /// Check if position is on a rectangular terrain island (left, right, or grid)
    fn is_position_on_island(&self, position: Vec3, config: &GameConfig) -> bool {
        let left_island_x = config.world_env.islands.left_x;
        let right_island_x = config.world_env.islands.right_x;
        let grid_island_x = config.world_env.islands.grid_x;
        let grid_island_z = config.world_env.islands.grid_z;
        let terrain_half_size = config.world_env.terrain.half_size;

        // Check left island (X=-1500, Z centered at 0)
        let on_left = position.x >= (left_island_x - terrain_half_size)
            && position.x <= (left_island_x + terrain_half_size)
            && position.z >= -terrain_half_size
            && position.z <= terrain_half_size;

        // Check right island (X=1500, Z centered at 0)
        let on_right = position.x >= (right_island_x - terrain_half_size)
            && position.x <= (right_island_x + terrain_half_size)
            && position.z >= -terrain_half_size
            && position.z <= terrain_half_size;

        // Check grid island (X=0, Z=1800)
        let on_grid = position.x >= (grid_island_x - terrain_half_size)
            && position.x <= (grid_island_x + terrain_half_size)
            && position.z >= (grid_island_z - terrain_half_size)
            && position.z <= (grid_island_z + terrain_half_size);

        on_left || on_right || on_grid
    }

    /// Check if entire road segment (both endpoints) is on island
    fn segment_on_island(&self, start: Vec3, end: Vec3, config: &GameConfig) -> bool {
        self.is_position_on_island(start, config) && self.is_position_on_island(end, config)
    }

    /// Check if position is on the grid island (X=0, Z=1800)
    fn is_on_grid_island(&self, position: Vec3, config: &GameConfig) -> bool {
        let grid_x = config.world_env.islands.grid_x;
        let grid_z = config.world_env.islands.grid_z;
        let half_size = config.world_env.terrain.half_size;

        position.x >= (grid_x - half_size)
            && position.x <= (grid_x + half_size)
            && position.z >= (grid_z - half_size)
            && position.z <= (grid_z + half_size)
    }

    /// Check if a cell coordinate falls within the grid island bounds
    /// Used to prevent per-chunk road generation from duplicating Manhattan grid roads
    fn is_cell_on_grid_island(
        &self,
        cell_coord: IVec2,
        cell_size: f32,
        config: &GameConfig,
    ) -> bool {
        let base_x = cell_coord.x as f32 * cell_size;
        let base_z = cell_coord.y as f32 * cell_size;
        let cell_center = Vec3::new(base_x + 0.5 * cell_size, 0.0, base_z + 0.5 * cell_size);
        self.is_on_grid_island(cell_center, config)
    }

    pub fn generate_roads_for_cell(
        &mut self,
        cell_coord: IVec2,
        cell_size: f32,
        _rng: &mut impl rand::Rng,
        config: &GameConfig,
    ) -> Vec<u64> {
        use rand::SeedableRng;
        let cell_seed = ((cell_coord.x as u64) << 32) | ((cell_coord.y as u64) & 0xFFFFFFFF);
        let mut rng = rand::rngs::StdRng::seed_from_u64(cell_seed ^ 0x524F414453);

        // GUARD: Prevent duplication with Manhattan grid generator
        // If this cell is on the grid island, ManhattanGridGenerator handles all roads
        if self.is_cell_on_grid_island(cell_coord, cell_size, config) {
            self.generated_cells.insert(cell_coord);
            debug!(
                "Skipping per-chunk road generation for cell ({}, {}) - on grid island (Manhattan grid handles this)",
                cell_coord.x, cell_coord.y
            );
            return Vec::new();
        }

        if self.generated_cells.contains(&cell_coord) {
            return Vec::new();
        }
        self.generated_cells.insert(cell_coord);

        let base_x = cell_coord.x as f32 * cell_size;
        let base_z = cell_coord.y as f32 * cell_size;

        let world_half_size = config.world_bounds.world_half_size;
        let buffer = cell_size;
        if base_x.abs() > (world_half_size - buffer) || base_z.abs() > (world_half_size - buffer) {
            return Vec::new();
        }

        let mut new_roads = Vec::new();
        let mut local_index: u16 = 0;

        if cell_coord == IVec2::ZERO {
            let roads = self.generate_premium_spawn_roads(
                base_x,
                base_z,
                cell_size,
                cell_coord,
                &mut rng,
                &mut local_index,
                config,
            );
            self.track_boundary_points(cell_coord, cell_size, &roads);
            self.connect_to_neighbors(cell_coord);
            return roads;
        }

        let arterial_spacing = 3;
        let is_vertical_arterial = cell_coord.x % arterial_spacing == 0;
        let is_horizontal_arterial = cell_coord.y % arterial_spacing == 0;

        if is_vertical_arterial {
            let road_type = if cell_coord.x % (arterial_spacing * 2) == 0 {
                RoadType::Highway
            } else {
                RoadType::MainStreet
            };
            let y = config.world_env.land_elevation + road_type.height();
            let x_pos = base_x + cell_size * 0.5;
            let start = Vec3::new(x_pos, y, base_z);
            let end = Vec3::new(x_pos, y, base_z + cell_size);

            // Only generate if BOTH endpoints are on island (prevents ocean roads)
            if self.segment_on_island(start, end, config) {
                let road_id = generate_unique_road_id(cell_coord, local_index);
                local_index += 1;
                let road = RoadSpline::new(road_id, start, end, road_type);

                // DEBUG ASSERTION: Verify vertical road is axis-aligned (same X, different Z)
                debug_assert!(
                    (start.x - end.x).abs() < 1e-3,
                    "VERTICAL road must be axis-aligned in X: start.x={} end.x={} diff={}",
                    start.x,
                    end.x,
                    (start.x - end.x).abs()
                );
                debug_assert!(
                    (start.z - end.z).abs() > 1e-3,
                    "VERTICAL road must differ in Z: start.z={} end.z={} diff={}",
                    start.z,
                    end.z,
                    (start.z - end.z).abs()
                );

                self.insert_road_checked(road);
                new_roads.push(road_id);
                debug!(
                    "Generated VERTICAL arterial {:?} in cell {:?}",
                    road_type, cell_coord
                );
            }
        }

        if is_horizontal_arterial {
            let road_type = if cell_coord.y % (arterial_spacing * 2) == 0 {
                RoadType::Highway
            } else {
                RoadType::MainStreet
            };
            let y = config.world_env.land_elevation + road_type.height();
            let z_pos = base_z + cell_size * 0.5;
            let start = Vec3::new(base_x, y, z_pos);
            let end = Vec3::new(base_x + cell_size, y, z_pos);

            // Only generate if BOTH endpoints are on island (prevents ocean roads)
            if self.segment_on_island(start, end, config) {
                let road_id = generate_unique_road_id(cell_coord, local_index);
                local_index += 1;
                let road = RoadSpline::new(road_id, start, end, road_type);

                // DEBUG ASSERTION: Verify horizontal road is axis-aligned (same Z, different X)
                debug_assert!(
                    (start.z - end.z).abs() < 1e-3,
                    "HORIZONTAL road must be axis-aligned in Z: start.z={} end.z={} diff={}",
                    start.z,
                    end.z,
                    (start.z - end.z).abs()
                );
                debug_assert!(
                    (start.x - end.x).abs() > 1e-3,
                    "HORIZONTAL road must differ in X: start.x={} end.x={} diff={}",
                    start.x,
                    end.x,
                    (start.x - end.x).abs()
                );

                self.insert_road_checked(road);
                new_roads.push(road_id);
                debug!(
                    "Generated HORIZONTAL arterial {:?} in cell {:?}",
                    road_type, cell_coord
                );
            }
        }

        if !is_vertical_arterial && !is_horizontal_arterial {
            let road_type = RoadType::SideStreet;
            let y = config.world_env.land_elevation + road_type.height();

            let nearest_vertical = ((cell_coord.x as f32 / arterial_spacing as f32).round()
                * arterial_spacing as f32)
                * cell_size
                + cell_size * 0.5;
            let nearest_horizontal = ((cell_coord.y as f32 / arterial_spacing as f32).round()
                * arterial_spacing as f32)
                * cell_size
                + cell_size * 0.5;

            let start = Vec3::new(base_x + cell_size * 0.5, y, base_z + cell_size * 0.5);
            let end_v = Vec3::new(nearest_vertical, y, base_z + cell_size * 0.5);
            let end_h = Vec3::new(base_x + cell_size * 0.5, y, nearest_horizontal);

            // Only generate connectors if BOTH endpoints are on island (prevents ocean roads)
            if self.segment_on_island(start, end_v, config) {
                let road_id_v = generate_unique_road_id(cell_coord, local_index);
                local_index += 1;
                let connector_v = RoadSpline::new(road_id_v, start, end_v, road_type);
                self.roads.insert(road_id_v, connector_v);
                new_roads.push(road_id_v);
            }

            if self.segment_on_island(start, end_h, config) {
                let road_id_h = generate_unique_road_id(cell_coord, local_index);
                let connector_h = RoadSpline::new(road_id_h, start, end_h, road_type);
                self.roads.insert(road_id_h, connector_h);
                new_roads.push(road_id_h);
            }

            debug!("Generated connectors to arterials in cell {:?}", cell_coord);
        }

        self.track_boundary_points(cell_coord, cell_size, &new_roads);
        self.connect_to_neighbors(cell_coord);

        new_roads
    }

    #[deprecated(note = "Use generate_roads_for_cell")]
    #[allow(unreachable_code)]
    pub fn generate_chunk_roads(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        config: &GameConfig,
    ) -> Vec<u64> {
        use rand::{Rng, SeedableRng};

        let cell = IVec2::new(chunk_x, chunk_z);
        let seed = ((chunk_x as u64) << 32) | ((chunk_z as u64) & 0xFFFFFFFF);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let roads = self.generate_roads_for_cell(
            cell,
            config.world_streaming.road_cell_size,
            &mut rng,
            config,
        );

        // SAFETY GUARD: Return early to prevent legacy diagonal road generation
        // The generate_roads_for_cell function already creates proper axis-aligned roads
        // All code below this point is deprecated legacy logic that could create diagonal roads
        return roads;

        // Generate local roads for island chunks
        const CHUNK_SIZE: f32 = 200.0; // Default chunk size
        let chunk_center_x = chunk_x as f32 * CHUNK_SIZE + CHUNK_SIZE * 0.5;
        let chunk_center_z = chunk_z as f32 * CHUNK_SIZE + CHUNK_SIZE * 0.5;
        let chunk_center = Vec3::new(chunk_center_x, 0.0, chunk_center_z);

        if self.is_position_on_island(chunk_center, config) {
            // Generate 1-2 local roads per island chunk
            let num_local_roads = rng.gen_range(1..=2);
            for _ in 0..num_local_roads {
                let y = config.world_env.land_elevation + RoadType::SideStreet.height();
                let offset1_x = rng.gen_range(-80.0..80.0);
                let offset1_z = rng.gen_range(-80.0..80.0);
                let offset2_x = rng.gen_range(-80.0..80.0);
                let offset2_z = rng.gen_range(-80.0..80.0);

                let start = Vec3::new(chunk_center_x + offset1_x, y, chunk_center_z + offset1_z);
                let end = Vec3::new(chunk_center_x + offset2_x, y, chunk_center_z + offset2_z);

                // Verify both ends are on island (prevents ocean roads)
                if self.segment_on_island(start, end, config) {
                    let road_id = self.next_road_id;
                    self.next_road_id += 1;
                    let road = RoadSpline::new(road_id, start, end, RoadType::SideStreet);
                    self.roads.insert(road_id, road);
                    roads.push(road_id);
                }
            }
        }

        roads
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_premium_spawn_roads(
        &mut self,
        base_x: f32,
        base_z: f32,
        cell_size: f32,
        cell_coord: IVec2,
        _rng: &mut impl rand::Rng,
        local_index: &mut u16,
        config: &GameConfig,
    ) -> Vec<u64> {
        // GUARD: Don't generate premium roads in ocean cells (between islands)
        let cell_center = Vec3::new(base_x + cell_size * 0.5, 0.0, base_z + cell_size * 0.5);
        if !self.is_position_on_island(cell_center, config) {
            return Vec::new();
        }

        let mut spawn_roads = Vec::new();
        let highway_y = config.world_env.land_elevation + RoadType::Highway.height();
        let main_y = config.world_env.land_elevation + RoadType::MainStreet.height();
        let side_y = config.world_env.land_elevation + RoadType::SideStreet.height();

        let cell_min_x = base_x;
        let cell_max_x = base_x + cell_size;
        let cell_min_z = base_z;
        let cell_max_z = base_z + cell_size;

        let highway_configs = [
            (
                Vec3::new(cell_min_x, highway_y, base_z + cell_size * 0.5),
                Vec3::new(
                    base_x + cell_size * 0.3,
                    highway_y,
                    base_z + cell_size * 0.6,
                ),
                Vec3::new(cell_max_x, highway_y, base_z + cell_size * 0.5),
                RoadType::Highway,
            ),
            (
                Vec3::new(base_x + cell_size * 0.5, highway_y, cell_min_z),
                Vec3::new(
                    base_x + cell_size * 0.6,
                    highway_y,
                    base_z + cell_size * 0.3,
                ),
                Vec3::new(base_x + cell_size * 0.5, highway_y, cell_max_z),
                RoadType::Highway,
            ),
        ];

        let main_street_configs = [
            (
                Vec3::new(cell_min_x, main_y, base_z + cell_size * 0.25),
                Vec3::new(base_x + cell_size * 0.3, main_y, base_z + cell_size * 0.3),
                Vec3::new(cell_max_x, main_y, base_z + cell_size * 0.25),
                RoadType::MainStreet,
            ),
            (
                Vec3::new(base_x + cell_size * 0.25, main_y, cell_min_z),
                Vec3::new(base_x + cell_size * 0.3, main_y, base_z + cell_size * 0.3),
                Vec3::new(base_x + cell_size * 0.25, main_y, cell_max_z),
                RoadType::MainStreet,
            ),
        ];

        for (start, control, end, road_type) in
            highway_configs.iter().chain(main_street_configs.iter())
        {
            let road_id = generate_unique_road_id(cell_coord, *local_index);
            *local_index += 1;
            let mut road = RoadSpline::new(road_id, *start, *end, *road_type);
            road.add_curve(*control);
            self.roads.insert(road_id, road);
            spawn_roads.push(road_id);
        }

        for i in 0..3 {
            for j in 0..3 {
                if i == 1 && j == 1 {
                    continue;
                }

                let sub_x = base_x + (i as f32 + 0.5) * cell_size / 4.0;
                let sub_z = base_z + (j as f32 + 0.5) * cell_size / 4.0;

                let start = Vec3::new(sub_x - 30.0, side_y, sub_z);
                let end = Vec3::new(sub_x + 30.0, side_y, sub_z);
                let road_id = generate_unique_road_id(cell_coord, *local_index);
                *local_index += 1;
                let road = RoadSpline::new(road_id, start, end, RoadType::SideStreet);
                self.roads.insert(road_id, road);
                spawn_roads.push(road_id);

                let start = Vec3::new(sub_x, side_y, sub_z - 30.0);
                let end = Vec3::new(sub_x, side_y, sub_z + 30.0);
                let road_id = generate_unique_road_id(cell_coord, *local_index);
                *local_index += 1;
                let road = RoadSpline::new(road_id, start, end, RoadType::SideStreet);
                self.roads.insert(road_id, road);
                spawn_roads.push(road_id);
            }
        }

        debug!(
            "Generated {} premium roads in cell (0,0)",
            spawn_roads.len()
        );
        spawn_roads
    }
}

// NOTE: RoadEntity and IntersectionEntity are defined in components/world.rs
