#!/bin/bash

# Fix remaining unused variables in gameplay_sim
cd gameplay_sim

# Fix layered_generation.rs
sed -i '' 's/meshes: &mut ResMut<Assets<Mesh>>,/_meshes: \&mut ResMut<Assets<Mesh>>,/g' src/systems/world/layered_generation.rs
sed -i '' 's/materials: &mut ResMut<Assets<StandardMaterial>>,/_materials: \&mut ResMut<Assets<StandardMaterial>>,/g' src/systems/world/layered_generation.rs
sed -i '' 's/coord: ChunkCoord) -> bool {/_coord: ChunkCoord) -> bool {/g' src/systems/world/layered_generation.rs
sed -i '' 's/meshes: ResMut<Assets<Mesh>>,/_meshes: ResMut<Assets<Mesh>>,/g' src/systems/world/layered_generation.rs
sed -i '' 's/materials: ResMut<Assets<StandardMaterial>>,/_materials: ResMut<Assets<StandardMaterial>>,/g' src/systems/world/layered_generation.rs

# Fix unified_lod.rs
sed -i '' 's/commands: Commands,/_commands: Commands,/g' src/systems/world/unified_lod.rs
sed -i '' 's/time: Res<Time>,/_time: Res<Time>,/g' src/systems/world/unified_lod.rs

# Fix npc_lod.rs
sed -i '' 's/for (entity, mut npc_state, transform, mut visibility)/for (_entity, mut npc_state, transform, mut visibility)/g' src/systems/world/npc_lod.rs
sed -i '' 's/meshes: &mut ResMut<Assets<Mesh>>,/_meshes: \&mut ResMut<Assets<Mesh>>,/g' src/systems/world/npc_lod.rs
sed -i '' 's/materials: &mut ResMut<Assets<StandardMaterial>>,/_materials: \&mut ResMut<Assets<StandardMaterial>>,/g' src/systems/world/npc_lod.rs

# Fix npc_spawn.rs
sed -i '' 's/game_config: Res<GameConfig>,/_game_config: Res<GameConfig>,/g' src/systems/world/npc_spawn.rs

# Fix vegetation_lod.rs
sed -i '' 's/for (entity, mut veg_lod, transform, mut visibility)/for (_entity, mut veg_lod, transform, mut visibility)/g' src/systems/world/vegetation_lod.rs
sed -i '' 's/meshes: ResMut<Assets<Mesh>>,/_meshes: ResMut<Assets<Mesh>>,/g' src/systems/world/vegetation_lod.rs
sed -i '' 's/materials: ResMut<Assets<StandardMaterial>>,/_materials: ResMut<Assets<StandardMaterial>>,/g' src/systems/world/vegetation_lod.rs

# Fix unified_distance_culling.rs
sed -i '' 's/time: Res<Time>,/_time: Res<Time>,/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/for (entity, cullable, vehicle_state)/for (_entity, cullable, _vehicle_state)/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/let target_lod = if cullable.last_distance < 100.0 {/let _target_lod = if cullable.last_distance < 100.0 {/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/for (entity, cullable) in building_query.iter()/for (_entity, cullable) in building_query.iter()/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/let target_lod = calculate_building_lod(cullable.last_distance);/let _target_lod = calculate_building_lod(cullable.last_distance);/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/for (entity, cullable) in npc_query.iter()/for (_entity, cullable) in npc_query.iter()/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/let target_lod = if cullable.last_distance < 50.0 {/let _target_lod = if cullable.last_distance < 50.0 {/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/let active_pos = active_transform.translation;/let _active_pos = active_transform.translation;/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/let cleanup_distance = 2000.0;/let _cleanup_distance = 2000.0;/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/for entity in cullable_query.iter()/for _entity in cullable_query.iter()/g' src/systems/world/unified_distance_culling.rs
sed -i '' 's/commands: Commands,/_commands: Commands,/g' src/systems/world/unified_distance_culling.rs

# Fix generic_bundle.rs
sed -i '' 's/let safe_position = self.validate_position(position)?;/let _safe_position = self.validate_position(position)?;/g' src/factories/generic_bundle.rs

echo "Fixed remaining unused variables"
