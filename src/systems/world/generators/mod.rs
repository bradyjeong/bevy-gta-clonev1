// Focused chunk generators following AGENT.md simplicity principles
// Each generator has single responsibility and minimal coupling

pub mod building_generator;
pub mod road_generator;
pub mod vehicle_generator;

pub use building_generator::BuildingGenerator;
pub use road_generator::RoadGenerator;
pub use vehicle_generator::VehicleGenerator;
