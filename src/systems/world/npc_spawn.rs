use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{
    NPCState, NPCType, NPCLOD, Cullable, NPCBehaviorType, NPCAppearance, NPCGender,
    NPC_LOD_CULL_DISTANCE
};
use crate::systems::timing_service::{TimingService, EntityTimerType, ManagedTiming};
use crate::services::ground_detection::GroundDetectionService;
use rand::prelude::*;

/// Spawn NPCs using the new architecture while maintaining compatibility
/// This system replaces the old spawn_dynamic_npc function
pub fn spawn_new_npc_system(
    mut commands: Commands,
    timing_service: Res<TimingService>,
    npc_query: Query<Entity, With<NPCState>>,
    ground_service: Res<GroundDetectionService>,
) {
    // Limit NPC spawning to avoid performance issues
    if npc_query.iter().count() >= 100 {
        return;
    }
    
    // Spawn new NPCs occasionally
    if timing_service.current_time % 5.0 < 0.1 {
        let mut rng = thread_rng();
        
        // Try to find a valid spawn position (avoid roads, buildings, etc.)
        for _ in 0..10 { // Max 10 attempts to find valid position
            let x = rng.gen_range(-50.0..50.0);
            let z = rng.gen_range(-50.0..50.0);
            let position = Vec2::new(x, z);
            
            if ground_service.is_spawn_position_valid(position) {
                spawn_simple_npc_with_ground_detection_simple(&mut commands, position, &ground_service);
                break; // Found valid position, spawn and exit
            }
        }
    }
}

/// Spawn a single NPC with ground detection (simplified version without RapierContext)
pub fn spawn_simple_npc_with_ground_detection_simple(
    commands: &mut Commands,
    position: Vec2,
    ground_service: &GroundDetectionService,
) -> Entity {
    let mut rng = thread_rng();
    
    // Use simplified ground detection
    let ground_height = ground_service.get_ground_height_simple(position);
    let ground_clearance = 0.02; // Very small clearance to avoid clipping
    let spawn_height = ground_height + ground_clearance; // Place NPC feet on ground
    
    let spawn_position = Vec3::new(position.x, spawn_height, position.y);
    
    // Create NPC with new state-based architecture
    let entity = commands.spawn((
        NPCState {
            npc_type: NPCType::Civilian,
            appearance: NPCAppearance {
                height: 1.8, // Standard NPC height
                build: rng.gen_range(0.8..1.2),
                skin_tone: Color::linear_rgb(0.8, 0.7, 0.6),
                hair_color: Color::linear_rgb(0.4, 0.3, 0.2),
                shirt_color: Color::linear_rgb(rng.gen_range(0.2..0.8), rng.gen_range(0.2..0.8), rng.gen_range(0.2..0.8)),
                pants_color: Color::linear_rgb(rng.gen_range(0.1..0.6), rng.gen_range(0.1..0.6), rng.gen_range(0.1..0.6)),
                gender: if rng.gen_bool(0.5) { NPCGender::Male } else { NPCGender::Female },
            },
            behavior: NPCBehaviorType::Wandering,
            target_position: spawn_position,
            speed: rng.gen_range(2.0..4.0),
            current_lod: NPCLOD::Full,
            last_lod_check: 0.0,
        },
        Transform::from_translation(spawn_position),
        GlobalTransform::default(),
    )).id();
    
    println!("DEBUG: Spawned NPC at {:?} (ground: {:.2})", spawn_position, ground_height);
    entity
}

/// Spawn a single NPC with ground detection (full physics version)
pub fn spawn_simple_npc_with_ground_detection(
    commands: &mut Commands,
    position: Vec2,
    ground_service: &GroundDetectionService,
    rapier_context: &RapierContext,
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
    
    // Get ground height at spawn position
    let ground_y = ground_service.get_spawn_height(position, height, rapier_context);
    let spawn_position = Vec3::new(position.x, ground_y, position.y);
    
    // Use simplified entity creation
    commands.spawn((
        npc_state,
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -height / 2.0, 0.0), Vec3::new(0.0, height / 2.0, 0.0), 0.3),
        Velocity::zero(),
        Transform::from_translation(spawn_position),
        Visibility::Visible,
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Cullable { max_distance: NPC_LOD_CULL_DISTANCE, is_culled: false },
    )).id()
}

/// Legacy spawn a single NPC using the simplified system
pub fn spawn_simple_npc(
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
    
    // Use simplified entity creation
    commands.spawn((
        npc_state,
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -height / 2.0, 0.0), Vec3::new(0.0, height / 2.0, 0.0), 0.3),
        Velocity::zero(),
        Transform::from_translation(position),
        Visibility::Visible,
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Cullable { max_distance: NPC_LOD_CULL_DISTANCE, is_culled: false },
    )).id()
}

/// Legacy spawn function for backward compatibility
#[deprecated(note = "Use spawn_npc_with_unified_factory instead")]
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
    
    #[allow(deprecated)]
    commands.spawn((
        // Use new simplified system
        npc_state,
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -height / 2.0, 0.0), Vec3::new(0.0, height / 2.0, 0.0), 0.3),
        Velocity::zero(),
        Transform::from_translation(position),
        Visibility::Visible,
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Cullable { max_distance: NPC_LOD_CULL_DISTANCE, is_culled: false },
    )).id()
}

/// Legacy compatibility system - converts old NPC entities to new architecture
pub fn migrate_legacy_npcs(
    mut commands: Commands,
    legacy_npc_query: Query<(Entity, &crate::components::NPC, &Transform), Without<NPCState>>,
) {
    for (entity, legacy_npc, _transform) in legacy_npc_query.iter() {
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
pub fn setup_new_npcs(
    mut commands: Commands,
    ground_service: Res<GroundDetectionService>,
) {
    let mut rng = thread_rng();
    
    // Spawn fewer NPCs initially - LOD system will handle performance
    let mut spawned_count = 0;
    let max_attempts = 100; // Prevent infinite loop
    let mut attempts = 0;
    
    while spawned_count < 25 && attempts < max_attempts {
        attempts += 1;
        let x = rng.gen_range(-900.0..900.0);
        let z = rng.gen_range(-900.0..900.0);
        let position = Vec2::new(x, z);
        
        if ground_service.is_spawn_position_valid(position) {
            spawn_simple_npc_with_ground_detection_simple(&mut commands, position, &ground_service);
            spawned_count += 1;
        }
    }
    
    println!("DEBUG: Spawned {} NPCs with ground detection (attempted {} positions)", spawned_count, attempts);
}
