use bevy::prelude::*;

/// Unified mesh factory that eliminates duplicate mesh creation patterns
#[derive(Resource)]
pub struct MeshFactory;

impl MeshFactory {
    pub fn new() -> Self {
        Self
    }

    // Vehicle meshes
    pub fn create_sports_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 4.2))
    }

    pub fn create_suv_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(2.5, 1.5, 5.0))
    }

    pub fn create_truck_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(3.0, 2.0, 8.0))
    }

    // Building meshes
    pub fn create_building_mesh(meshes: &mut ResMut<Assets<Mesh>>, width: f32, height: f32, depth: f32) -> Handle<Mesh> {
        meshes.add(Cuboid::new(width, height, depth))
    }

    // Aircraft meshes
    pub fn create_helicopter_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(2.0, 1.0, 4.0))
    }

    pub fn create_f16_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.5, 1.0, 8.0))
    }

    // Environment meshes
    pub fn create_tree_trunk(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 3.0))
    }

    pub fn create_road_segment(meshes: &mut ResMut<Assets<Mesh>>, width: f32, length: f32) -> Handle<Mesh> {
        meshes.add(Cuboid::new(width, 0.1, length))
    }

    // Water and terrain meshes
    pub fn create_water_plane(meshes: &mut ResMut<Assets<Mesh>>, size: f32) -> Handle<Mesh> {
        meshes.add(Plane3d::default().mesh().size(size, size))
    }

    pub fn create_terrain_chunk(meshes: &mut ResMut<Assets<Mesh>>, size: f32) -> Handle<Mesh> {
        meshes.add(Plane3d::default().mesh().size(size, size))
    }
}

impl Default for MeshFactory {
    fn default() -> Self {
        Self::new()
    }
}
