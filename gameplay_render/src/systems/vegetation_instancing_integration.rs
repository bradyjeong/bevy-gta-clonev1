use bevy::prelude::*;
use game_core::prelude::*;

/// Integration system to demonstrate vegetation instancing
/// This system would be called by your main vegetation spawning system
pub fn integrate_vegetation_with_instancing_system(
    mut commands: Commands,
    vegetation_query: Query<(Entity, &Transform), (With<Cullable>, Without<VegetationBatchable>)>,
    frame_counter: Res<FrameCounter>,
) {
    // Convert existing vegetation entities to use instancing
    for (entity, _transform) in vegetation_query.iter().take(10) { // Process 10 per frame
        // Determine vegetation type based on entity components
        let vegetation_type = determine_vegetation_type(&commands, entity);
        
        // Add vegetation batchable component
        commands.entity(entity).insert(VegetationBatchable {
            vegetation_type,
            mesh_id: None, // Will be populated by the rendering system
            material_id: None,
        });
        // Mark for instancing update
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Normal,
            frame_counter.frame,
        ));
    }
}
/// Helper function to determine vegetation type from entity components
fn determine_vegetation_type(_commands: &Commands, _entity: Entity) -> VegetationType {
    // This is a simplified example - in reality you'd check the entity's components
    // to determine the appropriate vegetation type
    
    // For now, randomly assign types for demonstration
    match rand::random::<u32>() % 4 {
        0 => VegetationType::PalmFrond,
        1 => VegetationType::LeafCluster,
        2 => VegetationType::TreeTrunk,
        _ => VegetationType::Bush,
/// System to spawn test vegetation for instancing demonstration
pub fn spawn_test_vegetation_system(
    config: Res<VegetationInstancingConfig>,
    mut spawned: Local<bool>,
    if *spawned {
        return;
    *spawned = true;
    info!("Spawning test vegetation for instancing...");
    // Spawn test vegetation entities
    for i in 0..100 {
        let x = (i as f32 % 10.0) * 5.0 - 25.0;
        let z = (i as f32 / 10.0) * 5.0 - 25.0;
        let y = 0.0;
        let vegetation_type = match i % 4 {
            0 => VegetationType::PalmFrond,
            1 => VegetationType::LeafCluster,
            2 => VegetationType::TreeTrunk,
            _ => VegetationType::Bush,
        };
        commands.spawn(InstancedVegetationBundle::new(
            &format!("TestVegetation_{}", i),
            Transform::from_translation(Vec3::new(x, y, z)),
        )).insert(Cullable::new(config.culling_distance));
    info!("Spawned 100 test vegetation entities for instancing");
