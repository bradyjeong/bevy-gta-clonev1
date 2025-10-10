use crate::util::safe_math::safe_lerp;
use bevy::prelude::*;
use std::collections::HashMap;

pub const ROAD_CELL_SIZE: f32 = 400.0;

fn zigzag32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

pub fn generate_unique_road_id(cell_coord: IVec2, local_index: u16) -> u64 {
    let zx = zigzag32(cell_coord.x) as u64;
    let zy = zigzag32(cell_coord.y) as u64;
    let cell_key = (zx << 32) | zy;
    (cell_key << 16) | (local_index as u64)
}

// NEW GTA-STYLE ROAD NETWORK SYSTEM

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
            RoadType::Highway => 16.0,    // 4 lanes + shoulders (typical US highway)
            RoadType::MainStreet => 12.0, // 3-4 lanes (main city street)
            RoadType::SideStreet => 8.0,  // 2 lanes (residential street)
            RoadType::Alley => 4.0,       // 1 lane (narrow alley)
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

#[derive(Resource, Default)]
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
        self.next_road_id = 0;
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
        let tolerance = 5.0;

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

    pub fn generate_roads_for_cell(
        &mut self,
        cell_coord: IVec2,
        cell_size: f32,
        _rng: &mut impl rand::Rng,
    ) -> Vec<u64> {
        use rand::SeedableRng;
        let cell_seed = ((cell_coord.x as u64) << 32) | ((cell_coord.y as u64) & 0xFFFFFFFF);
        let mut rng = rand::rngs::StdRng::seed_from_u64(cell_seed ^ 0x524F414453);
        if self.generated_cells.contains(&cell_coord) {
            return Vec::new();
        }
        self.generated_cells.insert(cell_coord);

        let base_x = cell_coord.x as f32 * cell_size;
        let base_z = cell_coord.y as f32 * cell_size;

        const WORLD_HALF_SIZE: f32 = 3000.0;
        let buffer = cell_size;
        if base_x.abs() > (WORLD_HALF_SIZE - buffer) || base_z.abs() > (WORLD_HALF_SIZE - buffer) {
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
            let height = road_type.height();
            let x_pos = base_x + cell_size * 0.5;
            let start = Vec3::new(x_pos, height, base_z);
            let end = Vec3::new(x_pos, height, base_z + cell_size);

            let road_id = generate_unique_road_id(cell_coord, local_index);
            local_index += 1;
            let road = RoadSpline::new(road_id, start, end, road_type);
            self.roads.insert(road_id, road);
            new_roads.push(road_id);
            debug!(
                "Generated VERTICAL arterial {:?} in cell {:?}",
                road_type, cell_coord
            );
        }

        if is_horizontal_arterial {
            let road_type = if cell_coord.y % (arterial_spacing * 2) == 0 {
                RoadType::Highway
            } else {
                RoadType::MainStreet
            };
            let height = road_type.height();
            let z_pos = base_z + cell_size * 0.5;
            let start = Vec3::new(base_x, height, z_pos);
            let end = Vec3::new(base_x + cell_size, height, z_pos);

            let road_id = generate_unique_road_id(cell_coord, local_index);
            local_index += 1;
            let road = RoadSpline::new(road_id, start, end, road_type);
            self.roads.insert(road_id, road);
            new_roads.push(road_id);
            debug!(
                "Generated HORIZONTAL arterial {:?} in cell {:?}",
                road_type, cell_coord
            );
        }

        if !is_vertical_arterial && !is_horizontal_arterial {
            let road_type = RoadType::SideStreet;
            let height = road_type.height();
            let start = Vec3::new(base_x + cell_size * 0.2, height, base_z + cell_size * 0.2);
            let end = Vec3::new(base_x + cell_size * 0.8, height, base_z + cell_size * 0.8);
            let road_id = generate_unique_road_id(cell_coord, local_index);
            local_index += 1;
            let road = RoadSpline::new(road_id, start, end, road_type);
            self.roads.insert(road_id, road);
            new_roads.push(road_id);
            debug!(
                "Generated SideStreet in cell {:?} - no main roads",
                cell_coord
            );
        }

        if !is_vertical_arterial && !is_horizontal_arterial {
            let road_type = RoadType::SideStreet;
            let height = road_type.height();

            let nearest_vertical = ((cell_coord.x as f32 / arterial_spacing as f32).round()
                * arterial_spacing as f32)
                * cell_size
                + cell_size * 0.5;
            let nearest_horizontal = ((cell_coord.y as f32 / arterial_spacing as f32).round()
                * arterial_spacing as f32)
                * cell_size
                + cell_size * 0.5;

            let start = Vec3::new(base_x + cell_size * 0.5, height, base_z + cell_size * 0.5);
            let end_v = Vec3::new(nearest_vertical, height, base_z + cell_size * 0.5);
            let end_h = Vec3::new(base_x + cell_size * 0.5, height, nearest_horizontal);

            let road_id_v = generate_unique_road_id(cell_coord, local_index);
            local_index += 1;
            let connector_v = RoadSpline::new(road_id_v, start, end_v, road_type);
            self.roads.insert(road_id_v, connector_v);
            new_roads.push(road_id_v);

            let road_id_h = generate_unique_road_id(cell_coord, local_index);
            let connector_h = RoadSpline::new(road_id_h, start, end_h, road_type);
            self.roads.insert(road_id_h, connector_h);
            new_roads.push(road_id_h);

            debug!("Generated connectors to arterials in cell {:?}", cell_coord);
        }

        self.track_boundary_points(cell_coord, cell_size, &new_roads);
        self.connect_to_neighbors(cell_coord);

        new_roads
    }

    #[deprecated(note = "Use generate_roads_for_cell")]
    pub fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u64> {
        use rand::SeedableRng;
        let cell = IVec2::new(chunk_x, chunk_z);
        let seed = ((chunk_x as u64) << 32) | ((chunk_z as u64) & 0xFFFFFFFF);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        self.generate_roads_for_cell(cell, ROAD_CELL_SIZE, &mut rng)
    }

    fn generate_premium_spawn_roads(
        &mut self,
        base_x: f32,
        base_z: f32,
        cell_size: f32,
        cell_coord: IVec2,
        _rng: &mut impl rand::Rng,
        local_index: &mut u16,
    ) -> Vec<u64> {
        let mut spawn_roads = Vec::new();
        let height = 0.0;

        let cell_min_x = base_x;
        let cell_max_x = base_x + cell_size;
        let cell_min_z = base_z;
        let cell_max_z = base_z + cell_size;

        let highway_configs = [
            (
                Vec3::new(cell_min_x, height, base_z + cell_size * 0.5),
                Vec3::new(base_x + cell_size * 0.3, height, base_z + cell_size * 0.6),
                Vec3::new(cell_max_x, height, base_z + cell_size * 0.5),
                RoadType::Highway,
            ),
            (
                Vec3::new(base_x + cell_size * 0.5, height, cell_min_z),
                Vec3::new(base_x + cell_size * 0.6, height, base_z + cell_size * 0.3),
                Vec3::new(base_x + cell_size * 0.5, height, cell_max_z),
                RoadType::Highway,
            ),
        ];

        let main_street_configs = [
            (
                Vec3::new(cell_min_x, height, base_z + cell_size * 0.25),
                Vec3::new(base_x + cell_size * 0.3, height, base_z + cell_size * 0.3),
                Vec3::new(cell_max_x, height, base_z + cell_size * 0.25),
                RoadType::MainStreet,
            ),
            (
                Vec3::new(base_x + cell_size * 0.25, height, cell_min_z),
                Vec3::new(base_x + cell_size * 0.3, height, base_z + cell_size * 0.3),
                Vec3::new(base_x + cell_size * 0.25, height, cell_max_z),
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

                let start = Vec3::new(sub_x - 30.0, height, sub_z);
                let end = Vec3::new(sub_x + 30.0, height, sub_z);
                let road_id = generate_unique_road_id(cell_coord, *local_index);
                *local_index += 1;
                let road = RoadSpline::new(road_id, start, end, RoadType::SideStreet);
                self.roads.insert(road_id, road);
                spawn_roads.push(road_id);

                let start = Vec3::new(sub_x, height, sub_z - 30.0);
                let end = Vec3::new(sub_x, height, sub_z + 30.0);
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
