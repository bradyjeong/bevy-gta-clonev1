use bevy::prelude::*;
use gta_game::components::*;
use gta_game::systems::world::vegetation_lod::*;

#[test]
fn test_vegetation_lod_distance_thresholds() {
    // Test that VegetationLOD correctly calculates detail levels based on distance
    
    // Test full detail (< 50m)
    let close_lod = VegetationLOD::from_distance(30.0);
    assert_eq!(close_lod.detail_level, VegetationDetailLevel::Full);
    assert!(close_lod.should_be_visible());
    
    // Test medium detail (50m-150m)
    let medium_lod = VegetationLOD::from_distance(100.0);
    assert_eq!(medium_lod.detail_level, VegetationDetailLevel::Medium);
    assert!(medium_lod.should_be_visible());
    
    // Test billboard (150m-300m)
    let billboard_lod = VegetationLOD::from_distance(200.0);
    assert_eq!(billboard_lod.detail_level, VegetationDetailLevel::Billboard);
    assert!(billboard_lod.should_be_visible());
    
    // Test culled (> 300m)
    let culled_lod = VegetationLOD::from_distance(400.0);
    assert_eq!(culled_lod.detail_level, VegetationDetailLevel::Culled);
    assert!(!culled_lod.should_be_visible());
}

#[test]
fn test_vegetation_lod_updates() {
    // Test that LOD level updates correctly when distance changes
    let mut lod = VegetationLOD::new();
    
    // Start close
    lod.update_from_distance(25.0, 1);
    assert_eq!(lod.detail_level, VegetationDetailLevel::Full);
    
    // Move to medium range
    lod.update_from_distance(75.0, 2);
    assert_eq!(lod.detail_level, VegetationDetailLevel::Medium);
    assert_eq!(lod.last_update_frame, 2);
    
    // Move to billboard range
    lod.update_from_distance(175.0, 3);
    assert_eq!(lod.detail_level, VegetationDetailLevel::Billboard);
    assert_eq!(lod.last_update_frame, 3);
    
    // Move to culled range
    lod.update_from_distance(350.0, 4);
    assert_eq!(lod.detail_level, VegetationDetailLevel::Culled);
    assert_eq!(lod.last_update_frame, 4);
}

#[test]
fn test_vegetation_mesh_lod_selection() {
    // Test that the correct mesh is selected for each LOD level
    let mut app = App::new();
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    
    let full_mesh = meshes.add(Cylinder::new(0.3, 8.0).mesh());
    let medium_mesh = meshes.add(Cylinder::new(0.25, 6.0).mesh());
    let billboard_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 3.0));
    
    let mesh_lod = VegetationMeshLOD::new(
        full_mesh.clone(),
        Some(medium_mesh.clone()),
        billboard_mesh.clone(),
    );
    
    // Test mesh selection for each detail level
    assert_eq!(
        mesh_lod.get_mesh_for_level(VegetationDetailLevel::Full),
        Some(&full_mesh)
    );
    assert_eq!(
        mesh_lod.get_mesh_for_level(VegetationDetailLevel::Medium),
        Some(&medium_mesh)
    );
    assert_eq!(
        mesh_lod.get_mesh_for_level(VegetationDetailLevel::Billboard),
        Some(&billboard_mesh)
    );
    assert_eq!(
        mesh_lod.get_mesh_for_level(VegetationDetailLevel::Culled),
        None
    );
}

#[test]
fn test_lod_frame_counter() {
    // Test that the frame counter advances correctly
    let mut counter = LODFrameCounter::default();
    assert_eq!(counter.frame, 0);
    
    counter.frame += 1;
    assert_eq!(counter.frame, 1);
}
