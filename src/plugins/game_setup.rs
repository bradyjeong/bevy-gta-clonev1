use bevy::prelude::*;
use rand::prelude::*;

use crate::system_sets::GameSystemSets;
use crate::services::{initialize_simple_services, update_timing_service_system, ground_detection::GroundDetectionService};
use crate::systems::{
service_example_vehicle_creation, service_example_config_validation, 
service_example_timing_check
   // setup_unified_entity_factory // Function doesn't exist
};
use crate::setup::{
setup_basic_world,
// setup_initial_aircraft_unified,
// setup_initial_npcs_unified,
// setup_palm_trees,
   // setup_initial_vehicles_unified
};
use crate::setup::world::setup_dubai_noon_lighting;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
// use crate::components::world::ContentType;
use crate::GameConfig;

/// Plugin for organizing all startup and runtime systems with proper ordering
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app
            // Configure system sets
            .configure_sets(Startup, (
                GameSystemSets::ServiceInit,
                GameSystemSets::WorldSetup.after(GameSystemSets::ServiceInit),
                GameSystemSets::SecondarySetup.after(GameSystemSets::WorldSetup),
            ))
            .configure_sets(Update, GameSystemSets::ServiceUpdates)
            
            // Service initialization
            .add_systems(Startup, 
                initialize_simple_services.in_set(GameSystemSets::ServiceInit)
            )
            
            // Core world setup (keep light-only in Startup)
            .add_systems(Startup, (
                // setup_unified_entity_factory, // Function doesn't exist
                setup_basic_world,
                setup_dubai_noon_lighting,
                // Move heavy aircraft spawn out of Startup if needed
                // setup_initial_aircraft_unified,
            ).in_set(GameSystemSets::WorldSetup))
            
            // Secondary setup (migrated to throttled Update systems)
            // .add_systems(Startup, (
            //     setup_palm_trees,
            //     setup_initial_npcs_unified,
            //     setup_initial_vehicles_unified,
            // ).in_set(GameSystemSets::SecondarySetup))
            
            // Throttled bootstrap spawners (spread work across frames)
            .init_resource::<BootstrapProgress>()
            .add_systems(Update, (
                spawn_palm_trees_over_time,
                spawn_npcs_over_time,
                spawn_vehicles_over_time,
            ).in_set(GameSystemSets::SecondarySetup))
            
            // Runtime service systems
            .add_systems(Update, (
                update_timing_service_system,
                service_example_vehicle_creation,
                service_example_config_validation,
                service_example_timing_check,
            ).in_set(GameSystemSets::ServiceUpdates));
    }
}

#[derive(Resource, Default)]
struct BootstrapProgress {
    npcs_spawned: usize,
    vehicles_spawned: usize,
    trees_spawned: usize,
}

const NPC_TARGET: usize = 40;      // tune to desired totals
const VEHICLE_TARGET: usize = 12;  // tune to desired totals
const TREE_TARGET: usize = 150;    // tune to desired totals
const BOOTSTRAP_BUDGET_MS: u128 = 3; // per frame work budget

fn spawn_npcs_over_time(
    mut progress: ResMut<BootstrapProgress>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground: Res<GroundDetectionService>,
    config: Res<GameConfig>,
) {
    let start = std::time::Instant::now();
    let mut rng = thread_rng();
    let mut entity_factory = UnifiedEntityFactory::with_config(config.clone());
    
    while progress.npcs_spawned < NPC_TARGET {
        let x = rng.gen_range(-200.0..200.0);
        let z = rng.gen_range(-200.0..200.0);
        let pos2 = Vec2::new(x, z);

        if ground.is_spawn_position_valid(pos2) {
            let y = ground.get_ground_height_simple(pos2) + 0.1;
            let pos3 = Vec3::new(x, y, z);
            let _ = entity_factory.spawn_npc_consolidated(
                &mut commands,
                &mut meshes,
                &mut materials,
                pos3,
                0.0,
            );
            progress.npcs_spawned += 1;
        }
        if start.elapsed().as_millis() > BOOTSTRAP_BUDGET_MS { break; }
    }
}

fn spawn_vehicles_over_time(
    mut progress: ResMut<BootstrapProgress>,
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground: Res<GroundDetectionService>,
    config: Res<GameConfig>,
) {
    let start = std::time::Instant::now();
    let entity_factory = UnifiedEntityFactory::with_config(config.clone());
    
    const PREFERRED: [Vec3; 3] = [
        Vec3::new(25.0, 0.0, 10.0),
        Vec3::new(-30.0, 0.0, 15.0),
        Vec3::new(10.0, 0.0, -25.0),
    ];

    while progress.vehicles_spawned < VEHICLE_TARGET {
        let idx = progress.vehicles_spawned % PREFERRED.len();
        let base = PREFERRED[idx];
        let ground_h = ground.get_ground_height_simple(Vec2::new(base.x, base.z));
        let candidate = Vec3::new(base.x, ground_h + 0.5, base.z);

        let Ok(validated) = entity_factory.validate_position(candidate) else {
            progress.vehicles_spawned += 1;
            if start.elapsed().as_millis() > BOOTSTRAP_BUDGET_MS { break; }
            continue;
        };

        let vehicle = commands
            .spawn((
                Transform::from_translation(validated),
                Visibility::default(),
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ))
            .id();

        let _ = SpawnValidator::spawn_entity_safely(
            &mut spawn_registry,
            validated,
            SpawnableType::Vehicle,
            vehicle,
        );

        progress.vehicles_spawned += 1;
        if start.elapsed().as_millis() > BOOTSTRAP_BUDGET_MS { break; }
    }
}

fn palm_positions(idx: usize) -> Option<(f32, f32)> {
    const POS: &[(f32,f32)] = &[
        (10.0, 15.0), (15.0, 8.0), (-12.0, 18.0), (-8.0, -14.0),
        (22.0, -16.0), (-18.0, 12.0), (25.0, 25.0), (-25.0, -25.0),
        (45.0, 35.0), (38.0, -42.0), (-35.0, 48.0), (-45.0, -38.0),
    ];
    POS.get(idx).copied()
}

fn spawn_palm_trees_over_time(
    mut progress: ResMut<BootstrapProgress>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let start = std::time::Instant::now();
    while progress.trees_spawned < TREE_TARGET {
        if let Some((x,z)) = palm_positions(progress.trees_spawned) {
            let palm = commands
                .spawn((
                    Transform::from_xyz(x, 0.0, z),
                    Visibility::Visible,
                    InheritedVisibility::VISIBLE,
                    ViewVisibility::default(),
                ))
                .id();

            let trunk = commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.4, 0.25, 0.15),
                    ..default()
                })),
                Transform::from_xyz(0.0, 4.0, 0.0),
            )).id();
            commands.entity(palm).add_child(trunk);

            for i in 0..2 {
                let angle = (i as f32) * std::f32::consts::PI;
                let leaf = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(2.5, 0.1, 0.8))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.6, 0.25),
                    ..default()
                })),
                    Transform::from_xyz(angle.cos() * 1.2, 7.5, angle.sin() * 1.2)
                        .with_rotation(Quat::from_rotation_y(angle) * Quat::from_rotation_z(-0.2)),
                )).id();
                commands.entity(palm).add_child(leaf);
            }
        }
        progress.trees_spawned += 1;
        if start.elapsed().as_millis() > BOOTSTRAP_BUDGET_MS { break; }
    }
}

