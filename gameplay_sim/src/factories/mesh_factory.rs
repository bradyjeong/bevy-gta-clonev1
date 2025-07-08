use bevy::prelude::*;

/// Unified mesh factory that eliminates duplicate mesh creation patterns
#[derive(Resource)]
pub struct MeshFactory;

impl MeshFactory {
    #[must_use] pub fn new() -> Self {
        Self
    }

    // Vehicle meshes
    pub fn create_sports_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 4.2))
    }

    pub fn create_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 4.2))
    }

    pub fn create_generic_car(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.7, 1.0, 3.8))
    }

    pub fn create_standard_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 0.2))
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

    pub fn create_helicopter_cockpit(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.2, 2.0))
    }

    pub fn create_helicopter_tail_boom(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(0.4, 0.4, 3.0))
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

    pub fn create_rotor_blade(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(0.1, 0.02, 2.0))
    }

    pub fn create_landing_skid(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(0.1, 0.05, 0.8))
    }

    pub fn create_custom_cuboid(meshes: &mut ResMut<Assets<Mesh>>, x: f32, y: f32, z: f32) -> Handle<Mesh> {
        meshes.add(Cuboid::new(x, y, z))
    }
}

impl Default for MeshFactory {
    fn default() -> Self {
        Self::new()
    }
}
