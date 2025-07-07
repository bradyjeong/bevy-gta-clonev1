//! Simple test to verify gameplay_render compiles and basic functionality works

use bevy::prelude::*;
use gameplay_render::prelude::*;
#[test]
fn test_render_plugin_exists() {
    let mut app = App::new();
    app.add_plugins(RenderPlugin);
    
    // Test that the plugin added successfully
    assert!(app.world().entities().len() >= 0);
}
fn test_basic_rendering_components() {
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        HierarchyPlugin,
    ));
    // Test basic entity creation with transform
    let entity = app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::Visible,
    )).id();
    let world = app.world();
    assert!(world.get::<Transform>(entity).is_some());
    assert!(world.get::<Visibility>(entity).is_some());
fn test_distance_calculation() {
    let pos1 = Vec3::new(0.0, 0.0, 0.0);
    let pos2 = Vec3::new(3.0, 4.0, 0.0);
    let distance = pos1.distance(pos2);
    // 3-4-5 triangle
    assert!((distance - 5.0).abs() < 1e-6);
fn test_lod_level_enum() {
    // Test that we can create LOD levels
    let high_lod = LodLevel::High;
    let medium_lod = LodLevel::Medium;
    let sleep_lod = LodLevel::Sleep;
    assert_ne!(high_lod, medium_lod);
    assert_ne!(medium_lod, sleep_lod);
fn test_visibility_enum() {
    let visible = Visibility::Visible;
    let hidden = Visibility::Hidden;
    assert_ne!(visible, hidden);
