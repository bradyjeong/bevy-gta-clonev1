//! Content lifecycle tracking for dynamic content entities
//! 
//! Replaces DynamicContentSpawned and DynamicContentDespawned events
//! with more efficient query-based pattern using Added<T> and RemovedComponents<T>.

use bevy::prelude::*;
use crate::components::dynamic_content;
#[allow(unused_imports)] // For legacy compatibility layer
use crate::events::world::content_events;

/// Plugin that registers content lifecycle tracking systems
pub struct ContentObserverPlugin;

impl Plugin for ContentObserverPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ObserverMetrics>()
            // Use query-based lifecycle tracking (more compatible with Bevy 0.16)
            .add_systems(
                PostUpdate,
                (
                    track_dynamic_content_added,
                    track_dynamic_content_removed,
                )
            );
            
        // Compatibility layer for legacy events (can be removed later)
        #[cfg(feature = "legacy-events")]
        {
            app.add_systems(
                PostUpdate,
                (
                    emit_legacy_spawn_events,
                    emit_legacy_despawn_events,
                )
                    .in_set(LegacyCompatibilitySet)
            );
            
            app.configure_sets(PostUpdate, LegacyCompatibilitySet);
        }
    }
}

/// System set for legacy compatibility systems
#[cfg(feature = "legacy-events")]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegacyCompatibilitySet;

/// Track when DynamicContent component is added to entities
/// 
/// This replaces the DynamicContentSpawned event with a more efficient
/// query-based pattern using the Added<T> filter.
/// 
/// # Performance Benefits
/// - No per-frame event clearing overhead
/// - Query-based filtering is cache-efficient
/// - Only processes newly added components
fn track_dynamic_content_added(
    mut commands: Commands,
    query: Query<(Entity, &dynamic_content::DynamicContent, &Transform), Added<dynamic_content::DynamicContent>>,
    mut metrics: ResMut<ObserverMetrics>,
) {
    for (entity, content, transform) in query.iter() {
        // Perform any initialization needed for dynamic content
        trace!(
            "Dynamic content spawned: entity={:?}, type={:?}, pos={:?}",
            entity,
            content.content_type,
            transform.translation
        );
        
        // Mark entity as fully initialized
        commands.entity(entity).insert(ContentInitialized);
        
        // Update metrics
        metrics.spawn_observer_calls += 1;
    }
}

/// Track when DynamicContent component is removed from entities
/// 
/// This replaces the DynamicContentDespawned event with automatic cleanup
/// using RemovedComponents<T>.
fn track_dynamic_content_removed(
    mut removed: RemovedComponents<dynamic_content::DynamicContent>,
    mut metrics: ResMut<ObserverMetrics>,
) {
    for entity in removed.read() {
        // Perform cleanup for dynamic content
        trace!("Dynamic content despawned: entity={:?}", entity);
        
        // Update metrics
        metrics.despawn_observer_calls += 1;
    }
}

/// Marker component indicating content has been fully initialized
#[derive(Component)]
pub struct ContentInitialized;

// ============================================================================
// Legacy Compatibility Layer
// ============================================================================

/// Tracks newly spawned entities for legacy event emission
#[cfg(feature = "legacy-events")]
#[derive(Component)]
pub struct PendingSpawnEvent {
    pub content_type: content_events::ContentType,
}

/// Tracks entities pending despawn for legacy event emission  
#[cfg(feature = "legacy-events")]
#[derive(Component)]
pub struct PendingDespawnEvent;

/// Emit legacy spawn events for compatibility with existing systems
/// 
/// This system bridges the observer pattern with the old event system
/// during migration. It can be removed once all systems are updated.
#[cfg(feature = "legacy-events")]
#[deprecated(
    since = "0.2.0",
    note = "Use on_dynamic_content_added observer instead of DynamicContentSpawned event"
)]
fn emit_legacy_spawn_events(
    query: Query<(Entity, &Transform, &dynamic_content::DynamicContent), Added<ContentInitialized>>,
    mut event_writer: EventWriter<content_events::DynamicContentSpawned>,
) {
    for (entity, transform, content) in query.iter() {
        // Emit legacy event for backward compatibility
        event_writer.write(content_events::DynamicContentSpawned::new(
            entity,
            transform.translation,
            content.content_type.into(),
        ));
        
        trace!(
            "Emitted legacy DynamicContentSpawned event for entity {:?}",
            entity
        );
    }
}

/// Emit legacy despawn events for compatibility with existing systems
#[cfg(feature = "legacy-events")]
#[deprecated(
    since = "0.2.0",
    note = "Use on_dynamic_content_removed observer instead of DynamicContentDespawned event"
)]
fn emit_legacy_despawn_events(
    mut removed: RemovedComponents<dynamic_content::DynamicContent>,
    mut event_writer: EventWriter<content_events::DynamicContentDespawned>,
) {
    for entity in removed.read() {
        // Emit legacy event for backward compatibility
        event_writer.write(content_events::DynamicContentDespawned::new(entity));
        
        trace!(
            "Emitted legacy DynamicContentDespawned event for entity {:?}",
            entity
        );
    }
}

// ============================================================================
// Migration Helpers
// ============================================================================

/// Helper trait to convert between event ContentType and component ContentType
impl From<dynamic_content::ContentType> for content_events::ContentType {
    fn from(ct: dynamic_content::ContentType) -> Self {
        match ct {
            dynamic_content::ContentType::Road => content_events::ContentType::Road,
            dynamic_content::ContentType::Building => content_events::ContentType::Building,
            dynamic_content::ContentType::Tree => content_events::ContentType::Tree,
            dynamic_content::ContentType::Vehicle => content_events::ContentType::Vehicle,
            dynamic_content::ContentType::NPC => content_events::ContentType::NPC,
        }
    }
}

// ============================================================================
// Performance Monitoring
// ============================================================================

/// Resource tracking observer performance metrics
#[derive(Resource, Default)]
pub struct ObserverMetrics {
    pub spawn_observer_calls: u64,
    pub despawn_observer_calls: u64,
    pub average_spawn_time_us: f32,
    pub average_despawn_time_us: f32,
}

/// Update observer metrics (called from observers with timing)
pub fn update_observer_metrics(
    metrics: &mut ObserverMetrics,
    observer_type: ObserverType,
    duration_us: f32,
) {
    match observer_type {
        ObserverType::Spawn => {
            metrics.spawn_observer_calls += 1;
            // Running average
            metrics.average_spawn_time_us = 
                (metrics.average_spawn_time_us * (metrics.spawn_observer_calls - 1) as f32 
                + duration_us) / metrics.spawn_observer_calls as f32;
        }
        ObserverType::Despawn => {
            metrics.despawn_observer_calls += 1;
            metrics.average_despawn_time_us = 
                (metrics.average_despawn_time_us * (metrics.despawn_observer_calls - 1) as f32 
                + duration_us) / metrics.despawn_observer_calls as f32;
        }
    }
}

pub enum ObserverType {
    Spawn,
    Despawn,
}
