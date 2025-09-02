use crate::components::water_new::{WaterRegion, WaterRegionAsset};
use bevy::prelude::*;

/// Asset loader system for water regions
pub fn load_water_regions_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load water region assets
    let ocean_handle: Handle<WaterRegionAsset> = asset_server.load("config/water/ocean.ron");
    let lake_handle: Handle<WaterRegionAsset> = asset_server.load("config/water/lake.ron");

    // Store handles for processing when loaded
    commands.insert_resource(WaterAssetHandles {
        ocean: ocean_handle,
        lake: lake_handle,
    });

    info!("Started loading water region assets");
}

/// Process loaded water assets and spawn water region entities
pub fn process_loaded_water_assets(
    mut commands: Commands,
    _handles: Option<Res<WaterAssetHandles>>,
    water_assets: ResMut<Assets<WaterRegionAsset>>,
    mut asset_events: EventReader<AssetEvent<WaterRegionAsset>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                if let Some(asset) = water_assets.get(*id) {
                    // Convert asset to component and spawn entity
                    let water_region = WaterRegion {
                        name: asset.name.clone(),
                        bounds: asset.bounds,
                        base_level: asset.base_level,
                        current_level: asset.base_level, // Initialize to base level
                        tide: asset.tide.clone(),
                        wave_params: asset.wave_params.clone(),
                        surface_color: asset.surface_color,
                    };

                    commands.spawn((
                        water_region,
                        Transform::from_translation(Vec3::ZERO),
                        Visibility::default(),
                        Name::new(asset.name.clone()),
                    ));

                    info!("Spawned water region: {}", asset.name);
                }
            }
            AssetEvent::Modified { id } => {
                // Handle hot-reloading of water assets
                if let Some(asset) = water_assets.get(*id) {
                    info!("Water asset modified: {}", asset.name);
                    // Could update existing entities here
                }
            }
            _ => {}
        }
    }
}

/// Resource to track water asset handles
#[derive(Resource)]
pub struct WaterAssetHandles {
    pub ocean: Handle<WaterRegionAsset>,
    pub lake: Handle<WaterRegionAsset>,
}
