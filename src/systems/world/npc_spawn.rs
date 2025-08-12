use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{
    NPCCore, NPCType, NPCLOD, Cullable, NPCBehaviorType, NPCAppearance, NPCGender,
    NPC_LOD_CULL_DISTANCE, NPCVisuals
};
use crate::events::world::chunk_events::ChunkLoaded;
use crate::config::GameConfig;
use crate::services::timing_service::{EntityTimerType, ManagedTiming};
use crate::services::ground_detection::GroundDetectionService;
use rand::prelude::*;

/// Observer-based NPC spawning that responds to chunk loading events
/// Replaces timer-based polling with reactive spawning per architectural_shift.md §59-63
pub fn on_npc_spawn_request(
    trigger: Trigger<ChunkLoaded>,
    mut commands: Commands,
    npc_query: Query<Entity, With<NPCCore>>,
    ground_service: Res<GroundDetectionService>,
    _config: Res<GameConfig>,
) {
    // Limit NPC spawning to avoid performance issues (unified entity limits)
    if npc_query.iter().count() >= 20 {  // REDUCED: From 100 to 20 NPCs max
        return;
    }
    
    let chunk_loaded = trigger.event();
    let chunk_coord = chunk_loaded.coord;
    let mut rng = thread_rng();
    
    // Calculate spawn area based on chunk coordinates
    let chunk_center_x = (chunk_coord.x as f32) * 200.0 + 100.0;
    let chunk_center_z = (chunk_coord.z as f32) * 200.0 + 100.0;
    
    // Spawn 1-3 NPCs per chunk with low probability
    let spawn_count = if rng.gen_range(0.0..1.0) < 0.3 { // 30% chance to spawn NPCs
        rng.gen_range(1..=3)
    } else {
        0
    };
    
    for _ in 0..spawn_count {
        // Try to find a valid spawn position within chunk bounds
        for _ in 0..5 { // REDUCED: From 10 to 5 attempts
            let x = chunk_center_x + rng.gen_range(-80.0..80.0);
            let z = chunk_center_z + rng.gen_range(-80.0..80.0);
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
        NPCCore {
            npc_type: NPCType::Civilian,
            behavior: NPCBehaviorType::Wandering,
            target_position: spawn_position,
            speed: rng.gen_range(2.0..4.0),
            current_lod: NPCLOD::Full,
            last_lod_check: 0.0,
        },
        NPCVisuals {
            appearance: NPCAppearance {
                height: 1.8, // Standard NPC height
                build: rng.gen_range(0.8..1.2),
                skin_tone: Color::linear_rgb(0.8, 0.7, 0.6),
                hair_color: Color::linear_rgb(0.4, 0.3, 0.2),
                shirt_color: Color::linear_rgb(rng.gen_range(0.2..0.8), rng.gen_range(0.2..0.8), rng.gen_range(0.2..0.8)),
                pants_color: Color::linear_rgb(rng.gen_range(0.1..0.6), rng.gen_range(0.1..0.6), rng.gen_range(0.1..0.6)),
                gender: if rng.gen_bool(0.5) { NPCGender::Male } else { NPCGender::Female },
            },
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
    
    let npc_state = NPCCore::new(npc_type);
    let height = 1.8; // Default NPC height - appearance is in separate component
    
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
    
    let npc_state = NPCCore::new(npc_type);
    let height = 1.8; // Default NPC height - appearance is in separate component
    
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

/// NPC spawn using unified factory (replaces legacy functions)
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
    
    let npc_state = NPCCore::new(npc_type);
    let height = 1.8; // Default NPC height - appearance is in separate component
    
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

/// Migration system - converts old NPC entities to unified architecture
pub fn migrate_legacy_npcs(
    mut commands: Commands,
    legacy_npc_query: Query<(Entity, &crate::components::NPC, &Transform), Without<NPCCore>>,
) {
    for (entity, legacy_npc, _transform) in legacy_npc_query.iter() {
        // Create new state component based on legacy data
        let mut npc_state = NPCCore::new(NPCType::Civilian);
        npc_state.target_position = legacy_npc.target_position;
        npc_state.speed = legacy_npc.speed;
        npc_state.current_lod = NPCLOD::StateOnly; // Start with no rendering
        
        // Add new components while keeping the old one for compatibility
        commands.entity(entity).insert((
            npc_state,
            ManagedTiming::new(EntityTimerType::NPCLOD),
        ));
        
        println!("DEBUG: Migrated NPC entity {:?} to unified architecture", entity);
    }
}


