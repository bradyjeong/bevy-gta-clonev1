//! ───────────────────────────────────────────────
//! System:   Road Network
//! Purpose:  Manages road splines and network connectivity
//! Schedule: Initialization
//! Reads:    RoadEntity, IntersectionEntity
//! Writes:   RoadNetwork
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum RoadType {
    Highway,
    MainStreet,
    SideStreet,
    Alley,
}

impl RoadType {
    pub fn width(&self) -> f32 {
        match self {
            RoadType::Highway => 14.0,
            RoadType::MainStreet => 10.0,
            RoadType::SideStreet => 6.0,
            RoadType::Alley => 4.0,
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
        
        0.5 * (
            (2.0 * p1) +
            (-p0 + p2) * local_t +
            (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 +
            (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
        )
    }
    
    pub fn length(&self) -> f32 {
        let mut total_length = 0.0;
        let samples = 20;
        
        for i in 0..samples {
            let t1 = i as f32 / samples as f32;
            let t2 = (i + 1) as f32 / samples as f32;
            
            let p1 = self.evaluate(t1);
            let p2 = self.evaluate(t2);
            
            total_length += p1.distance(p2);
        }
        
        total_length
    }
    
    pub fn closest_point(&self, position: Vec3) -> (f32, Vec3) {
        let mut closest_t = 0.0;
        let mut closest_distance = f32::MAX;
        let mut closest_point = Vec3::ZERO;
        
        let samples = 50;
        for i in 0..=samples {
            let t = i as f32 / samples as f32;
            let point = self.evaluate(t);
            let distance = position.distance(point);
            
            if distance < closest_distance {
                closest_distance = distance;
                closest_t = t;
                closest_point = point;
            }
        }
        
        (closest_t, closest_point)
    }
    
    pub fn direction_at(&self, t: f32) -> Vec3 {
        let epsilon = 0.01;
        let t1 = (t - epsilon).max(0.0);
        let t2 = (t + epsilon).min(1.0);
        
        let p1 = self.evaluate(t1);
        let p2 = self.evaluate(t2);
        
        (p2 - p1).normalize()
    }
}

#[derive(Debug, Clone)]
pub struct RoadIntersection {
    pub id: u32,
    pub position: Vec3,
    pub connected_roads: Vec<u32>,
    pub intersection_type: IntersectionType,
}

#[derive(Debug, Clone)]
pub enum IntersectionType {
    TJunction,
    CrossRoads,
    Roundabout,
}

#[derive(Resource, Debug, Default)]
pub struct RoadNetwork {
    pub roads: HashMap<u32, RoadSpline>,
    pub intersections: HashMap<u32, RoadIntersection>,
    pub next_road_id: u32,
    pub next_intersection_id: u32,
}

impl RoadNetwork {
    pub fn new() -> Self {
        Self {
            roads: HashMap::new(),
            intersections: HashMap::new(),
            next_road_id: 1,
            next_intersection_id: 1,
        }
    }
    
    pub fn add_road(&mut self, start: Vec3, end: Vec3, road_type: RoadType) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;
        
        let road = RoadSpline::new(id, start, end, road_type);
        self.roads.insert(id, road);
        
        id
    }
    
    pub fn add_intersection(&mut self, position: Vec3, intersection_type: IntersectionType) -> u32 {
        let id = self.next_intersection_id;
        self.next_intersection_id += 1;
        
        let intersection = RoadIntersection {
            id,
            position,
            connected_roads: Vec::new(),
            intersection_type,
        };
        
        self.intersections.insert(id, intersection);
        
        id
    }
    
    pub fn connect_roads(&mut self, road1_id: u32, road2_id: u32) {
        if let Some(road1) = self.roads.get_mut(&road1_id) {
            if !road1.connections.contains(&road2_id) {
                road1.connections.push(road2_id);
            }
        }
        
        if let Some(road2) = self.roads.get_mut(&road2_id) {
            if !road2.connections.contains(&road1_id) {
                road2.connections.push(road1_id);
            }
        }
    }
    
    pub fn find_nearest_road(&self, position: Vec3) -> Option<(u32, f32, Vec3)> {
        let mut nearest_road = None;
        let mut nearest_distance = f32::MAX;
        let mut nearest_point = Vec3::ZERO;
        
        for (road_id, road) in &self.roads {
            let (_, closest_point) = road.closest_point(position);
            let distance = position.distance(closest_point);
            
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_road = Some(*road_id);
                nearest_point = closest_point;
            }
        }
        
        nearest_road.map(|id| (id, nearest_distance, nearest_point))
    }
    
    pub fn get_road_at_position(&self, position: Vec3, tolerance: f32) -> Option<&RoadSpline> {
        for road in self.roads.values() {
            let (_, closest_point) = road.closest_point(position);
            if position.distance(closest_point) <= tolerance {
                return Some(road);
            }
        }
        None
    }
    
    pub fn roads_in_radius(&self, center: Vec3, radius: f32) -> Vec<&RoadSpline> {
        self.roads.values()
            .filter(|road| {
                // Check if any point of the road is within radius
                let samples = 10;
                for i in 0..=samples {
                    let t = i as f32 / samples as f32;
                    let point = road.evaluate(t);
                    if center.distance(point) <= radius {
                        return true;
                    }
                }
                false
            })
            .collect()
    }
}

pub fn initialize_road_network(
    mut commands: Commands,
) {
    let mut network = RoadNetwork::new();
    
    // Add some basic roads for testing
    network.add_road(
        Vec3::new(-100.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        RoadType::MainStreet
    );
    
    network.add_road(
        Vec3::new(0.0, 0.0, -100.0),
        Vec3::new(0.0, 0.0, 100.0),
        RoadType::MainStreet
    );
    
    commands.insert_resource(network);
}

// NOTE: RoadEntity and IntersectionEntity are defined in components/world.rs
