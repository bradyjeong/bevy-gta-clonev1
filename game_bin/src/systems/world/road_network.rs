//! ───────────────────────────────────────────────
//! System:   Road Network
//! Purpose:  Handles user interface display and interaction
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * Distance calculations are cached for performance
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;

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
            RoadType::Highway => 16.0,     // 4 lanes + shoulders (typical US highway)
            RoadType::MainStreet => 12.0,  // 3-4 lanes (main city street)
            RoadType::SideStreet => 8.0,   // 2 lanes (residential street)
            RoadType::Alley => 4.0,        // 1 lane (narrow alley)
        }
    }
    
    pub fn height(&self) -> f32 {
        0.0  // All roads at same height - proper intersection handling prevents overlap
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
    pub id: u32,
    pub control_points: Vec<Vec3>,
    pub road_type: RoadType,
    pub connections: Vec<u32>, // Connected road IDs
}

impl RoadSpline {
    pub fn new(id: u32, start: Vec3, end: Vec3, road_type: RoadType) -> Self {
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
            self.control_points[0].lerp(self.control_points[1], t)
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
            return points[0].lerp(points[n-1], t);
        }
        
        // Find the segment
        let segment_t = t * (n - 3) as f32;
        let segment = segment_t.floor() as usize;
        let local_t = segment_t.fract();
        
        let p0 = points[segment.max(0)];
        let p1 = points[(segment + 1).min(n-1)];
        let p2 = points[(segment + 2).min(n-1)];
        let p3 = points[(segment + 3).min(n-1)];
        
        // Catmull-Rom interpolation
        let t2 = local_t * local_t;
        let t3 = t2 * local_t;
        
        0.5 * ((2.0 * p1) + 
               (-p0 + p2) * local_t +
               (2.0*p0 - 5.0*p1 + 4.0*p2 - p3) * t2 +
               (-p0 + 3.0*p1 - 3.0*p2 + p3) * t3)
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
    Cross,       // 4-way intersection
    TJunction,  // 3-way intersection
    Curve,       // 2-way curved connection
    HighwayOnramp, // Highway merge
}

#[derive(Debug, Clone)]
pub struct RoadIntersection {
    pub position: Vec3,
    pub connected_roads: Vec<u32>,
    pub intersection_type: IntersectionType,
    pub radius: f32,
}

#[derive(Resource)]
pub struct RoadNetwork {
    pub roads: HashMap<u32, RoadSpline>,
    pub intersections: HashMap<u32, RoadIntersection>,
    pub next_road_id: u32,
    pub next_intersection_id: u32,
    pub generated_chunks: std::collections::HashSet<(i32, i32)>, // Track generated areas
}

impl Default for RoadNetwork {
    fn default() -> Self {
        Self {
            roads: HashMap::new(),
            intersections: HashMap::new(),
            next_road_id: 0,
            next_intersection_id: 0,
            generated_chunks: std::collections::HashSet::new(),
        }
    }
}

impl RoadNetwork {
    pub fn clear_cache(&mut self) {
        self.generated_chunks.clear();
        println!("DEBUG: Road network cache cleared!");
    }
    
    pub fn reset(&mut self) {
        self.roads.clear();
        self.intersections.clear();
        self.generated_chunks.clear();
        self.next_road_id = 0;
        self.next_intersection_id = 0;
        println!("DEBUG: Road network completely reset!");
    }
}

impl RoadNetwork {
    pub fn add_road(&mut self, start: Vec3, end: Vec3, road_type: RoadType) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;
        
        let road = RoadSpline::new(id, start, end, road_type);
        self.roads.insert(id, road);
        id
    }
    
    pub fn add_curved_road(&mut self, start: Vec3, control: Vec3, end: Vec3, road_type: RoadType) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;
        
        let mut road = RoadSpline::new(id, start, end, road_type);
        road.add_curve(control);
        self.roads.insert(id, road);
        id
    }
    
    pub fn connect_roads(&mut self, road1_id: u32, road2_id: u32) {
        if let Some(road1) = self.roads.get_mut(&road1_id) {
            road1.connections.push(road2_id);
        }
        if let Some(road2) = self.roads.get_mut(&road2_id) {
            road2.connections.push(road1_id);
        }
    }
    
    pub fn add_intersection(&mut self, position: Vec3, connected_roads: Vec<u32>, intersection_type: IntersectionType) -> u32 {
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
    
    // Generate road network for a chunk
    pub fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32> {
        if self.generated_chunks.contains(&(chunk_x, chunk_z)) {
            // println!("DEBUG: Chunk ({}, {}) already generated, skipping", chunk_x, chunk_z);
            return Vec::new();
        }
        // println!("DEBUG: Generating roads for NEW chunk ({}, {})", chunk_x, chunk_z);
        self.generated_chunks.insert((chunk_x, chunk_z));
        
        let chunk_size = 400.0; // Much larger chunks for better performance
        let base_x = chunk_x as f32 * chunk_size;
        let base_z = chunk_z as f32 * chunk_size;
        
        let mut new_roads = Vec::new();
        
        // UNIFIED GENERATION: Handle spawn chunk (0,0) with premium roads
        if chunk_x == 0 && chunk_z == 0 {
            return self.generate_premium_spawn_roads(base_x, base_z, chunk_size);
        }
        
        // Generate non-overlapping road grid to prevent z-fighting
        // Use alternating pattern to ensure roads don't overlap
        let mut roads_added = Vec::new();
        
        if chunk_x % 2 == 0 && chunk_z % 2 != 0 {
            // Vertical roads only on even X, odd Z chunks
            let road_type = RoadType::MainStreet;
            let height = road_type.height();
            let start = Vec3::new(base_x, height, base_z - chunk_size * 0.5);
            let control = Vec3::new(base_x + rand::random::<f32>() * 20.0 - 10.0, height, base_z + chunk_size * 0.2);
            let end = Vec3::new(base_x, height, base_z + chunk_size * 1.5);
            
            let road_id = self.add_curved_road(start, control, end, road_type);
            new_roads.push(road_id);
            roads_added.push("vertical");
            println!("🛣️ DEBUG: Generated VERTICAL MainStreet in chunk ({}, {})", chunk_x, chunk_z);
        }
        
        if chunk_z % 2 == 0 && chunk_x % 2 != 0 {
            // Horizontal roads only on odd X, even Z chunks  
            let road_type = RoadType::MainStreet;
            let height = road_type.height();
            let start = Vec3::new(base_x - chunk_size * 0.5, height, base_z);
            let control = Vec3::new(base_x + chunk_size * 0.2, height, base_z + rand::random::<f32>() * 20.0 - 10.0);
            let end = Vec3::new(base_x + chunk_size * 1.5, height, base_z);
            
            let road_id = self.add_curved_road(start, control, end, road_type);
            new_roads.push(road_id);
            roads_added.push("horizontal");
            println!("🛣️ DEBUG: Generated HORIZONTAL MainStreet in chunk ({}, {})", chunk_x, chunk_z);
        }
        
        // Add side streets only where no main roads exist
        if roads_added.is_empty() {
            let road_type = RoadType::SideStreet;
            let height = road_type.height();
            let start = Vec3::new(base_x + chunk_size * 0.2, height, base_z + chunk_size * 0.2);
            let end = Vec3::new(base_x + chunk_size * 0.8, height, base_z + chunk_size * 0.8);
            let road_id = self.add_road(start, end, road_type);
            new_roads.push(road_id);
            println!("🛣️ DEBUG: Generated SideStreet in chunk ({}, {}) - no main roads", chunk_x, chunk_z);
        }
        
        // Generate side streets (much fewer)
        for i in 0..2 {
            for j in 0..2 {
                let sub_x = base_x + (i as f32 + 0.5) * chunk_size / 3.0;
                let sub_z = base_z + (j as f32 + 0.5) * chunk_size / 3.0;
                
                // Add some randomness to break the grid
                let offset_x = (rand::random::<f32>() - 0.5) * 30.0;
                let offset_z = (rand::random::<f32>() - 0.5) * 30.0;
                
                if rand::random::<f32>() < 0.8 { // 80% chance for side street
                    let road_type = RoadType::SideStreet;
                    let height = road_type.height();
                    let start = Vec3::new(sub_x + offset_x, height, sub_z - 40.0);
                    let end = Vec3::new(sub_x + offset_x, height, sub_z + 40.0);
                    
                    let road_id = self.add_road(start, end, road_type);
                    new_roads.push(road_id);
                }
                
                if rand::random::<f32>() < 0.8 { // 80% chance for side street
                    let road_type = RoadType::SideStreet;
                    let height = road_type.height();
                    let start = Vec3::new(sub_x - 40.0, height, sub_z + offset_z);
                    let end = Vec3::new(sub_x + 40.0, height, sub_z + offset_z);
                    
                    let road_id = self.add_road(start, end, road_type);
                    new_roads.push(road_id);
                }
            }
        }
        
        new_roads
    }
    
    // Generate premium roads for spawn chunk (0,0) - respects chunk boundaries
    fn generate_premium_spawn_roads(&mut self, base_x: f32, base_z: f32, chunk_size: f32) -> Vec<u32> {
        let mut spawn_roads = Vec::new();
        let height = 0.0; // Unified ground level for all roads
        
        // Calculate chunk boundaries to prevent overlap
        let chunk_min_x = base_x;
        let chunk_max_x = base_x + chunk_size;
        let chunk_min_z = base_z;
        let chunk_max_z = base_z + chunk_size;
        
        // Premium highways within chunk boundaries only
        let highway_configs = [
            // Main highway (horizontal through center)
            (Vec3::new(chunk_min_x, height, base_z + chunk_size * 0.5), 
             Vec3::new(base_x + chunk_size * 0.3, height, base_z + chunk_size * 0.6), 
             Vec3::new(chunk_max_x, height, base_z + chunk_size * 0.5), RoadType::Highway),
            
            // Cross highway (vertical through center)
            (Vec3::new(base_x + chunk_size * 0.5, height, chunk_min_z),
             Vec3::new(base_x + chunk_size * 0.6, height, base_z + chunk_size * 0.3),
             Vec3::new(base_x + chunk_size * 0.5, height, chunk_max_z), RoadType::Highway),
        ];
        
        // Premium main streets within chunk boundaries
        let main_street_configs = [
            // Main street parallel to highway
            (Vec3::new(chunk_min_x, height, base_z + chunk_size * 0.25),
             Vec3::new(base_x + chunk_size * 0.3, height, base_z + chunk_size * 0.3),
             Vec3::new(chunk_max_x, height, base_z + chunk_size * 0.25), RoadType::MainStreet),
            
            // Cross main street
            (Vec3::new(base_x + chunk_size * 0.25, height, chunk_min_z),
             Vec3::new(base_x + chunk_size * 0.3, height, base_z + chunk_size * 0.3),
             Vec3::new(base_x + chunk_size * 0.25, height, chunk_max_z), RoadType::MainStreet),
        ];
        
        // Generate roads within chunk boundaries
        for (start, control, end, road_type) in highway_configs.iter().chain(main_street_configs.iter()) {
            let road_id = self.add_curved_road(*start, *control, *end, *road_type);
            spawn_roads.push(road_id);
        }
        
        // Add some side streets for density
        for i in 0..3 {
            for j in 0..3 {
                if i == 1 && j == 1 { continue; } // Skip center where highways cross
                
                let sub_x = base_x + (i as f32 + 0.5) * chunk_size / 4.0;
                let sub_z = base_z + (j as f32 + 0.5) * chunk_size / 4.0;
                
                // Horizontal side street
                let start = Vec3::new(sub_x - 30.0, height, sub_z);
                let end = Vec3::new(sub_x + 30.0, height, sub_z);
                let road_id = self.add_road(start, end, RoadType::SideStreet);
                spawn_roads.push(road_id);
                
                // Vertical side street 
                let start = Vec3::new(sub_x, height, sub_z - 30.0);
                let end = Vec3::new(sub_x, height, sub_z + 30.0);
                let road_id = self.add_road(start, end, RoadType::SideStreet);
                spawn_roads.push(road_id);
            }
        }
        
        println!("🛣️ Generated {} premium roads within chunk (0,0) boundaries", spawn_roads.len());
        spawn_roads
    }
}

// NOTE: RoadEntity and IntersectionEntity are defined in components/world.rs
