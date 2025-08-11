//! Dynamic content spawn event handler
//! 
//! Handles RequestDynamicSpawn events by calling the unified entity factory,
//! then emits DynamicContentSpawned events. This replaces direct factory calls.

use bevy::prelude::*;
use crate::events::world::content_events::{RequestDynamicSpawn, DynamicContentSpawned, ContentType as EventContentType};
use crate::components::world::ContentType;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::systems::world::road_network::RoadNetwork;
use crate::GlobalRng;

/// Handle dynamic content spawn requests using the unified factory
/// Named: handle_dynamic_spawn_request (per Oracle requirements)
pub fn handle_dynamic_spawn_request(
    mut commands: Commands,
    mut spawn_reader: EventReader<RequestDynamicSpawn>,
    mut spawn_complete_writer: EventWriter<DynamicContentSpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut unified_factory: ResMut<UnifiedEntityFactory>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    _rng: ResMut<GlobalRng>,
) {
    for request in spawn_reader.read() {
        // Convert event ContentType to components ContentType
        let component_content_type = convert_event_to_component_type(request.kind);
        
        // Use unified factory to spawn the entity
        if let Ok(Some(entity)) = unified_factory.spawn_entity_consolidated(
            &mut commands,
            &mut meshes,
            &mut materials,
            component_content_type,
            request.pos,
            Some(&road_network),
            &[], // No existing content collision check here (handled by validation)
            time.elapsed_secs(),
        ) {
            // Emit completion event
            spawn_complete_writer.write(DynamicContentSpawned::new(
                entity,
                request.pos,
                request.kind,
            ));
            
            println!("DEBUG: Spawned {} using unified factory at {:?}", 
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
