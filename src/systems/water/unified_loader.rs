use bevy::prelude::*;
use crate::components::unified_water::*;

/// Asset loader system for unified water regions  
pub fn load_unified_water_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load unified water region assets
    let ocean_handle: Handle<UnifiedWaterAsset> = asset_server.load("config/water/ocean.ron");
    let lake_handle: Handle<UnifiedWaterAsset> = asset_server.load("config/water/lake.ron");

    // Store handles for processing when loaded
    commands.insert_resource(UnifiedWaterAssetHandles {
        ocean: ocean_handle,
        lake: lake_handle,
    });

    info!("Started loading unified water region assets");
}

/// Process loaded unified water assets and spawn water region entities
pub fn process_loaded_unified_water_assets(
    mut commands: Commands,
    _handles: Option<Res<UnifiedWaterAssetHandles>>,
    water_assets: Res<Assets<UnifiedWaterAsset>>,
    mut asset_events: EventReader<AssetEvent<UnifiedWaterAsset>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                if let Some(asset) = water_assets.get(*id) {
                    // Convert asset to component and spawn entity
                    let unified_water_body = UnifiedWaterBody {
                        name: asset.name.clone(),
                        bounds: asset.bounds,
                        surface_level: asset.surface_level,
                        depth: asset.depth,
                        tide: asset.tide.clone(),
                        wave_params: asset.wave_params.clone(),
                        surface_color: asset.surface_color,
                    };

                    commands.spawn((
                        unified_water_body,
                        Transform::from_translation(Vec3::ZERO),
                        Visibility::default(),
                        Name::new(asset.name.clone()),
                    ));

                    info!("Spawned unified water region: {} at surface level {:.2}", 
                          asset.name, asset.surface_level);
                }
            }
            AssetEvent::Modified { id } => {
                // Handle hot-reloading of water assets
                if let Some(asset) = water_assets.get(*id) {
                    info!("Unified water asset modified: {}", asset.name);
                    // Could update existing entities here
                }
            }
            _ => {}
        }
    }
}

/// Resource to track unified water asset handles
#[derive(Resource)]
pub struct UnifiedWaterAssetHandles {
    pub ocean: Handle<UnifiedWaterAsset>,
    pub lake: Handle<UnifiedWaterAsset>,
}
