//! Dynamic content spawn event handler
//! 
//! Handles RequestDynamicSpawn events by calling the unified entity factory.
//! The DynamicContent component addition triggers observers automatically.

use bevy::prelude::*;
use crate::events::world::content_events::{RequestDynamicSpawn, ContentType as EventContentType};
#[allow(unused_imports)] // Will be removed after full migration
use crate::events::world::content_events::DynamicContentSpawned;
use crate::components::world::ContentType;
use crate::components::dynamic_content::DynamicContent;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::world::RoadNetwork;
use crate::GlobalRng;

#[cfg(feature = "debug-ui")]
use crate::events::EventCounters;

/// Handle dynamic content spawn requests using the unified factory
/// Named: handle_request_dynamic_spawn (per architectural_shift.md §80)
/// 
/// This system now relies on the Observer pattern - adding DynamicContent
/// component automatically triggers the on_dynamic_content_added observer
pub fn handle_request_dynamic_spawn(
    mut commands: Commands,
    mut spawn_reader: EventReader<RequestDynamicSpawn>,
    #[cfg(feature = "legacy-events")]
    mut spawn_complete_writer: EventWriter<DynamicContentSpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut unified_factory: ResMut<UnifiedEntityFactory>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    _rng: ResMut<GlobalRng>,
    #[cfg(feature = "debug-ui")]
    mut event_counters: Option<ResMut<EventCounters>>,
) {
    for request in spawn_reader.read() {
        #[cfg(feature = "debug-ui")]
        if let Some(ref mut counters) = event_counters {
            counters.record_received("RequestDynamicSpawn");
        }
        
        // Convert event ContentType to components ContentType
        let component_content_type = convert_event_to_component_type(request.kind);
        
        // Use unified factory to spawn the entity
        if let Ok(Some(entity)) = unified_factory.spawn_entity_consolidated(
            &mut commands,
            &mut meshes,
            &mut materials,
            component_content_type,
            request.pos,
            Some(&*road_network),
            &[], // No existing content collision check here (handled by validation)
            time.elapsed_secs(),
        ) {
            // Add DynamicContent component to trigger observer
            commands.entity(entity).insert(DynamicContent::new(
                crate::components::dynamic_content::ContentType::from(component_content_type)
            ));
            
            // Legacy event emission for compatibility
            #[cfg(feature = "legacy-events")]
            spawn_complete_writer.write(DynamicContentSpawned::new(
                entity,
                request.pos,
                request.kind,
            ));
            
            #[cfg(all(feature = "legacy-events", feature = "debug-ui"))]
            if let Some(ref mut counters) = event_counters {
                counters.record_sent("DynamicContentSpawned");
            }
            
            trace!("DEBUG: Spawned {} using unified factory at {:?}", 
                event_content_type_name(request.kind), request.pos);
        }
    }
}

/// Convert event ContentType to component ContentType
fn convert_event_to_component_type(event_type: EventContentType) -> ContentType {
    match event_type {
        EventContentType::Road => ContentType::Road,
        EventContentType::Building => ContentType::Building,
        EventContentType::Tree => ContentType::Tree,
        EventContentType::Vehicle => ContentType::Vehicle,
        EventContentType::NPC => ContentType::NPC,
    }
}

/// Convert event ContentType to human-readable name for debug output
fn event_content_type_name(content_type: EventContentType) -> &'static str {
    match content_type {
        EventContentType::Road => "road",
        EventContentType::Building => "building",
        EventContentType::Tree => "tree",
        EventContentType::Vehicle => "vehicle",
        EventContentType::NPC => "NPC",
    }
}
