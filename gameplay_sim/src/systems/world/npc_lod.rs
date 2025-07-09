//! ───────────────────────────────────────────────
//! System:   NPC LOD
//! Purpose:  Manages level-of-detail for NPCs
//! Schedule: Update
//! Reads:    `ActiveEntity`, Transform, `NPCState`
//! Writes:   `NPCState`, Visibility
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;
use game_core::bundles::VisibleBundle;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum NPCLOD {
    High,   // Full model, animations
    Medium, // Simplified model
    Low,    // Billboard or very simple model
    Hidden, // Not visible
}

#[derive(Component, Debug, Clone)]
pub struct NPCState {
    pub current_lod: NPCLOD,
    pub last_lod_update: f32,
}

impl Default for NPCState {
    fn default() -> Self {
        Self {
            current_lod: NPCLOD::High,
            last_lod_update: 0.0,
        }
    }
}

pub fn npc_lod_system(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut npc_query: Query<(Entity, &mut NPCState, &Transform, &mut Visibility), With<NPC>>,
    time: Res<Time>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let current_time = time.elapsed_secs();
        
        for (_entity, mut npc_state, transform, mut visibility) in &mut npc_query {
            // Throttle LOD updates to every 0.1 seconds per NPC
            if current_time - npc_state.last_lod_update < 0.1 {
                continue;
            }
            
            let distance = active_pos.distance(transform.translation);
            let new_lod = determine_npc_lod(distance);
            
            if new_lod != npc_state.current_lod {
                npc_state.current_lod = new_lod;
                npc_state.last_lod_update = current_time;
                
                // Update visibility based on LOD
                *visibility = match new_lod {
                    NPCLOD::Hidden => Visibility::Hidden,
                    _ => Visibility::Visible,
                };
            }
        }
    }
}

/// Determine appropriate LOD level based on distance
fn determine_npc_lod(distance: f32) -> NPCLOD {
    if distance < 50.0 {
        NPCLOD::High
    } else if distance < 100.0 {
        NPCLOD::Medium
    } else if distance < 150.0 {
        NPCLOD::Low
    } else {
        NPCLOD::Hidden
    }
}

// TODO: Re-enable when AnimationPlayer is properly available in Bevy 0.16
// pub fn npc_animation_lod_system(
//     npc_query: Query<&NPCState, With<NPC>>,
//     mut animation_query: Query<&mut AnimationPlayer>,
// ) {
//     for npc_state in npc_query.iter() {
//         // Adjust animation quality based on LOD
//         // This is a simplified example - real implementation would 
//         // need proper entity relationships
//         match npc_state.current_lod {
//             NPCLOD::High => {
//                 // Full animation at normal speed
//             }
//             NPCLOD::Medium => {
//                 // Reduced animation quality or frame rate
//             }
//             NPCLOD::Low => {
//                 // Minimal or no animation
//             }
//             NPCLOD::Hidden => {
//                 // No animation processing needed
//             }
//         }
//     }
// }

pub fn npc_mesh_lod_system(
    npc_query: Query<&NPCState, (With<NPC>, Changed<NPCState>)>,
    // In a real implementation, you'd have access to mesh handles here
) {
    for npc_state in npc_query.iter() {
        // Switch meshes based on LOD level
        match npc_state.current_lod {
            NPCLOD::High => {
                // Use high-detail mesh
            }
            NPCLOD::Medium => {
                // Use medium-detail mesh
            }
            NPCLOD::Low => {
                // Use low-detail mesh or billboard
            }
            NPCLOD::Hidden => {
                // No mesh needed
            }
        }
    }
}

// Helper function to create NPC with LOD component
pub fn spawn_npc_with_lod(
    commands: &mut Commands,
    position: Vec3,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    commands.spawn((
        Transform::from_translation(position),
        VisibleBundle::default(),
        NPC {
            target_position: Vec3::ZERO,
            speed: 1.0,
            last_update: 0.0,
            update_interval: 0.5,
            health: None,
            max_health: None,
            behavior_state: None,
            spawn_time: None,
        },
        NPCState::default(),
        DynamicContent {
            content_type: ContentType::NPC,
        },
        Cullable {
            max_distance: 200.0,
            is_culled: false,
        },
    )).id()
}
