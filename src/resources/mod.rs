pub mod material_registry;
pub mod npc_asset_cache;
pub mod vehicle_specs_assets;
pub mod world_rng;

pub use material_registry::{MaterialKey, MaterialRegistry};
pub use npc_asset_cache::{MeshShape, NPCAssetCache};
pub use vehicle_specs_assets::VehicleSpecsAssets;
pub use world_rng::WorldRng;
