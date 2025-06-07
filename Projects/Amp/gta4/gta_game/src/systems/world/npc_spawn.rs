use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{
    NPCState, NPCType, NPCLOD, Cullable, DynamicContent, ContentType,
    NPC_LOD_CULL_DISTANCE
};
use crate::systems::timing_service::{TimingService, EntityTimerType, ManagedTiming};
use crate::factories::BundleFactory;
use rand::prelude::*;

/// Spawn NPCs using the new architecture while maintaining compatibility
/// This system replaces the old spawn_dynamic_npc function
pub fn spawn_new_npc_system(
    mut commands: Commands,
    timing_service: Res<TimingService>,
    npc_query: Query<Entity, With<NPCState>>,
) {
    // Limit NPC spawning to avoid performance issues
    if npc_query.iter().count() >= 100 {
        return;
    }
    
    // Spawn new NPCs occasionally
    if timing_service.current_time % 5.0 < 0.1 {
        spawn_npc_with_new_architecture(&mut commands, Vec3::ZERO);
    }
}

/// Spawn a single NPC using the new component architecture
pub fn spawn_npc_with_new_architecture(
    commands: &mut Commands,
    position: Vec3,
) -> Entity {
    let mut rng = thread_rng();
    
    // Create NPC with new state-based architecture
    let npc_type = match rng.gen_range(0..4) {
        0 => NPCType::Civilian,
        1 => NPCType::Worker,
        2 => NPCType::Police,
        _ => NPCType::Emergency,
    };
    
    let npc_state = NPCState::new(npc_type);
    let height = npc_state.appearance.height;
    
    commands.spawn((
        // Core state component (always present)
        npc_state,
        
        // Physics components using bundle factory
        BundleFactory::create_npc_physics_bundle(Vec3::new(position.x, 1.0, position.z), height),
        
        // Performance and content management
        BundleFactory::create_npc_content_bundle(NPC_LOD_CULL_DISTANCE),
        
        // Note: NPCRendering will be added by the LOD system based on distance
    )).id()
}

/// Legacy compatibility system - converts old NPC entities to new architecture
pub fn migrate_legacy_npcs(
    mut commands: Commands,
    legacy_npc_query: Query<(Entity, &crate::components::NPC, &Transform), Without<NPCState>>,
) {
    for (entity, legacy_npc, transform) in legacy_npc_query.iter() {
        // Create new state component based on legacy data
        let mut npc_state = NPCState::new(NPCType::Civilian);
        npc_state.target_position = legacy_npc.target_position;
        npc_state.speed = legacy_npc.speed;
        npc_state.current_lod = NPCLOD::StateOnly; // Start with no rendering
        
        // Add new components while keeping the old one for compatibility
        commands.entity(entity).insert((
            npc_state,
            ManagedTiming::new(EntityTimerType::NPCLOD),
        ));
        
        println!("DEBUG: Migrated legacy NPC entity {:?} to new architecture", entity);
    }
}

/// Setup system to spawn initial NPCs with new architecture
pub fn setup_new_npcs(mut commands: Commands) {
    let mut rng = thread_rng();
    
    // Spawn fewer NPCs initially - LOD system will handle performance
    for _ in 0..25 {
        let x = rng.gen_range(-900.0..900.0);
        let z = rng.gen_range(-900.0..900.0);
        let position = Vec3::new(x, 1.0, z);
        
        spawn_npc_with_new_architecture(&mut commands, position);
    }
    
    println!("DEBUG: Spawned 25 NPCs with new visual architecture");
}
