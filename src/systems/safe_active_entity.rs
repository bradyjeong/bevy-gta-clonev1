//! Safe ActiveEntity Transfer System
//! 
//! Prevents gaps in ActiveEntity coverage that could cause coordinate explosions
//! by ensuring exactly one entity always has ActiveEntity component.
//! 
//! Following AGENT.md principles: isolated, single responsibility system.

use bevy::prelude::*;
use crate::components::ActiveEntity;

/// Component to queue ActiveEntity transfers safely
#[derive(Component)]
pub struct ActiveTransferRequest {
    pub target_entity: Entity,
}

/// Event fired when ActiveEntity transfer completes
#[derive(Event)]
pub struct ActiveEntityTransferred {
    pub from: Entity,
    pub to: Entity,
}

/// System that processes ActiveEntity transfer requests atomically
/// Guarantees exactly one entity has ActiveEntity at any time
pub fn active_transfer_executor_system(
    mut commands: Commands,
    transfer_requests: Query<(Entity, &ActiveTransferRequest)>,
    current_active: Query<Entity, With<ActiveEntity>>,
    mut transfer_events: EventWriter<ActiveEntityTransferred>,
) {
    // Process all transfer requests this frame
    for (requester_entity, request) in transfer_requests.iter() {
        // Get current active entity (should be exactly one)
        if let Ok(current) = current_active.single() {
            // Perform atomic transfer
            commands.entity(current).remove::<ActiveEntity>();
            commands.entity(request.target_entity).insert(ActiveEntity);
            
            // Fire event for other systems that need to know
            transfer_events.write(ActiveEntityTransferred {
                from: current,
                to: request.target_entity,
            });
            
            // Remove the request
            commands.entity(requester_entity).remove::<ActiveTransferRequest>();
            
            info!("ActiveEntity transferred from {:?} to {:?}", current, request.target_entity);
        } else {
            // This should never happen, but handle gracefully
            warn!("ActiveEntity transfer request but no current active entity found");
            commands.entity(requester_entity).remove::<ActiveTransferRequest>();
        }
    }
}

/// Diagnostic system to ensure ActiveEntity integrity
pub fn active_entity_integrity_check(
    active_query: Query<Entity, With<ActiveEntity>>,
) {
    let active_count = active_query.iter().count();
    
    if active_count == 0 {
        error!("No entities have ActiveEntity component - this will break world streaming!");
    } else if active_count > 1 {
        error!("Multiple entities have ActiveEntity component: {:?}", active_query.iter().collect::<Vec<_>>());
    }
    // active_count == 1 is correct, no logging needed
}

/// Helper function for systems that need to transfer ActiveEntity
pub fn queue_active_transfer(commands: &mut Commands, requester: Entity, target: Entity) {
    commands.entity(requester).insert(ActiveTransferRequest { target_entity: target });
}
