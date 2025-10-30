/// Marker component added to vehicles missing their asset-driven specs.
/// Prevents repeated warning logs for the same vehicle.
/// Added by validation systems on startup, removed when specs are loaded.
#[derive(bevy::prelude::Component)]
pub struct MissingSpecsWarned;
