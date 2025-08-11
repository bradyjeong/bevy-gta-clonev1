//! Optimized NPC components with proper size boundaries
//! 
//! Demonstrates component splitting for cache efficiency

use bevy::prelude::*;

/// Hot-path NPC state (≤64 bytes) - accessed every frame
#[derive(Component, Default, Debug, Clone)]
pub struct NPCCore {
    /// Current position (cached for quick access)
    pub position: Vec3,           // 12 bytes
    /// Current velocity
    pub velocity: Vec3,           // 12 bytes
    /// Current health
    pub health: f32,              // 4 bytes
    /// AI state enum (packed)
    pub ai_state: NPCAIState,     // 1 byte
    /// Behavior flags (bit-packed)
    pub flags: NPCFlags,          // 1 byte
    /// Target entity (if any)
    pub target: Option<Entity>,   // 12 bytes (Option<Entity> is 8 bytes + discriminant)
    /// Alert level (0-255)
    pub alert_level: u8,          // 1 byte
    /// Team/faction ID
    pub faction: u8,              // 1 byte
    // Total: 44 bytes (well under 64)
}

/// Packed AI state enum (1 byte)
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NPCAIState {
    #[default]
    Idle = 0,
    Patrolling = 1,
    Chasing = 2,
    Attacking = 3,
    Fleeing = 4,
    Dead = 5,
}

/// Bit-packed flags (1 byte)
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct NPCFlags(u8);

impl NPCFlags {
    pub const HOSTILE: u8 = 0b00000001;
    pub const ARMED: u8 = 0b00000010;
    pub const ALERT: u8 = 0b00000100;
    pub const INJURED: u8 = 0b00001000;
    pub const RUNNING: u8 = 0b00010000;
    pub const IN_VEHICLE: u8 = 0b00100000;
    pub const IS_LEADER: u8 = 0b01000000;
    pub const INVULNERABLE: u8 = 0b10000000;
    
    pub fn set(&mut self, flag: u8, value: bool) {
        if value {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
    
    pub fn has(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

/// Cold-path NPC configuration (size not critical) - rarely accessed
#[derive(Component, Default, Debug, Clone)]
#[component(immutable)]  // Marks as immutable for optimization
pub struct NPCConfig {
    /// Display name
    pub name: String,
    /// Appearance configuration
    pub appearance: NPCAppearance,
    /// Spawn point for respawning
    pub spawn_point: Vec3,
    /// Patrol route (if any)
    pub patrol_route: Vec<Vec3>,
    /// Inventory items
    pub inventory: Vec<ItemId>,
    /// Dialog tree ID
    pub dialog_id: Option<u32>,
    /// Special abilities
    pub abilities: Vec<AbilityId>,
}

/// Appearance data (kept separate as it's visual-only)
#[derive(Component, Default, Debug, Clone)]
#[component(immutable)]
pub struct NPCAppearance {
    pub model_id: u32,
    pub skin_tone: Color,
    pub hair_color: Color,
    pub clothing_variant: u8,
    pub accessories: u8,
}

/// Item and ability IDs (small types)
#[derive(Debug, Clone, Copy)]
pub struct ItemId(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct AbilityId(pub u16);

// ============================================================================
// SIZE VALIDATION
// ============================================================================

// Compile-time size assertion for hot-path component
const _: () = assert!(
    std::mem::size_of::<NPCCore>() <= 64,
    "NPCCore exceeds 64 bytes for hot-path component"
);

// ============================================================================
// USAGE EXAMPLE
// ============================================================================

/// Example system showing how to work with split components
pub fn update_npc_ai(
    mut query: Query<(&mut NPCCore, &NPCConfig), Without<Player>>,
    time: Res<Time>,
) {
    for (mut core, config) in query.iter_mut() {
        // Hot-path: Update AI state based on core data
        match core.ai_state {
            NPCAIState::Idle => {
                // Check alert level
                if core.alert_level > 50 {
                    core.ai_state = NPCAIState::Alert;
                }
            }
            NPCAIState::Patrolling => {
                // Only access cold config when needed
                if !config.patrol_route.is_empty() {
                    // Update position along route
                }
            }
            _ => {}
        }
    }
}

/// Example of spawning with split components
pub fn spawn_optimized_npc(
    mut commands: Commands,
    position: Vec3,
) {
    commands.spawn((
        // Hot-path component
        NPCCore {
            position,
            velocity: Vec3::ZERO,
            health: 100.0,
            ai_state: NPCAIState::Idle,
            flags: NPCFlags::default(),
            target: None,
            alert_level: 0,
            faction: 0,
        },
        // Cold-path configuration (only accessed when needed)
        NPCConfig {
            name: "Guard".to_string(),
            appearance: NPCAppearance::default(),
            spawn_point: position,
            patrol_route: vec![],
            inventory: vec![],
            dialog_id: None,
            abilities: vec![],
        },
        // Transform component (managed by Bevy)
        TransformBundle::from_transform(Transform::from_translation(position)),
    ));
}

// ============================================================================
// MIGRATION UTILITIES
// ============================================================================

/// Trait to help migrate from old NPCState to new split components
pub trait FromOldNPCState {
    fn split_components(self) -> (NPCCore, NPCConfig);
}

// This would be implemented for the old NPCState type during migration
