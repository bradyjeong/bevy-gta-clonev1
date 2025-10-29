//! Safe ActiveEntity Transfer System
//!
//! Prevents gaps in ActiveEntity coverage that could cause coordinate explosions
//! by ensuring exactly one entity always has ActiveEntity component.
//!
//! Following AGENT.md principles: isolated, single responsibility system.

use crate::components::ActiveEntity;
use bevy::prelude::*;

/// Component to queue ActiveEntity transfers safely
#[derive(Component)]
pub struct ActiveTransferRequest {
    pub target_entity: Entity,
    pub creation_time: f64,
}

/// System that processes ActiveEntity transfer requests atomically
/// Guarantees exactly one entity has ActiveEntity at any time
/// BUG #46 FIX: Process only ONE transfer per frame, sorted by creation order
pub fn active_transfer_executor_system(
    mut commands: Commands,
    transfer_requests: Query<(Entity, &ActiveTransferRequest)>,
    current_active: Query<Entity, With<ActiveEntity>>,
) {
    let request_count = transfer_requests.iter().count();

    if request_count == 0 {
        return;
    }

    if request_count > 1 {
        warn!(
            "Multiple ActiveTransferRequests detected ({} requests) - processing oldest only",
            request_count
        );
    }

    let mut requests: Vec<(Entity, &ActiveTransferRequest)> = transfer_requests.iter().collect();
    requests.sort_by(|a, b| {
        a.1.creation_time
            .partial_cmp(&b.1.creation_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some((requester_entity, request)) = requests.first() {
        if let Ok(current) = current_active.single() {
            commands.entity(current).remove::<ActiveEntity>();
            commands.entity(request.target_entity).insert(ActiveEntity);

            commands
                .entity(*requester_entity)
                .remove::<ActiveTransferRequest>();

            info!(
                "ActiveEntity transferred from {:?} to {:?}",
                current, request.target_entity
            );
        } else {
            warn!("ActiveEntity transfer request but no current active entity found");
            commands
                .entity(*requester_entity)
                .remove::<ActiveTransferRequest>();
        }
    }

    for (requester_entity, _) in requests.iter().skip(1) {
        commands
            .entity(*requester_entity)
            .remove::<ActiveTransferRequest>();
    }
}

/// Diagnostic system to ensure ActiveEntity integrity
pub fn active_entity_integrity_check(active_query: Query<Entity, With<ActiveEntity>>) {
    let active_count = active_query.iter().count();

    if active_count == 0 {
        error!("No entities have ActiveEntity component - this will break world streaming!");
    } else if active_count > 1 {
        error!(
            "Multiple entities have ActiveEntity component: {:?}",
            active_query.iter().collect::<Vec<_>>()
        );
    }
}

/// Helper function for systems that need to transfer ActiveEntity
pub fn queue_active_transfer(
    commands: &mut Commands,
    requester: Entity,
    target: Entity,
    time: &Res<Time>,
) {
    commands.entity(requester).insert(ActiveTransferRequest {
        target_entity: target,
        creation_time: time.elapsed_secs_f64(),
    });
}
