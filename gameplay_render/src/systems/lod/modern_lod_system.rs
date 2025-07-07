//! ───────────────────────────────────────────────
//! System:   Modern Lod System
//! Purpose:  Manages camera positioning and following
//! Schedule: Update
//! Reads:    ActiveEntity, VegetationBatchable, LodLevel, Car, Camera
//! Writes:   VegetationLOD, LodLevel
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Physics values are validated and finite
//!   * Only active entities can be controlled
//! Owner:    @render-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;

/// Modern LOD system that uses component-based approach
pub fn modern_lod_system(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &mut LodLevel, &GlobalTransform), (With<ActiveEntity>, Or<(With<Car>, With<SuperCar>, With<Helicopter>, With<F16>)>)>,
    mut npc_query: Query<(Entity, &mut LodLevel, &GlobalTransform), (With<ActiveEntity>, With<NPC>)>,
    mut vegetation_query: Query<(Entity, &mut VegetationLOD, &GlobalTransform), (With<ActiveEntity>, With<VegetationBatchable>)>,
    camera_query: Query<&GlobalTransform, (With<Camera>, Without<ActiveEntity>)>,
    config: Res<GameConfig>,
    mut performance_counters: ResMut<game_core::config::performance_config::PerformanceCounters>,
) {
    let Some(camera_transform) = camera_query.iter().next() else {
        return;
    };
    
    let camera_pos = camera_transform.translation();
    // Update vehicle LOD levels
    for (entity, mut lod_level, transform) in vehicle_query.iter_mut() {
        let distance = camera_pos.distance(transform.translation());
        let new_level = calculate_vehicle_lod(distance, &config);
        
        if *lod_level != new_level {
            *lod_level = new_level;
            performance_counters.lod_updates += 1;
            
            // Update component sets based on LOD level
            update_vehicle_components(&mut commands, entity, new_level);
        }
    }
    // Update NPC LOD levels
    for (entity, mut lod_level, transform) in npc_query.iter_mut() {
        let new_level = calculate_npc_lod(distance, &config);
            update_npc_components(&mut commands, entity, new_level);
    // Update vegetation LOD levels
    for (entity, mut veg_lod, transform) in vegetation_query.iter_mut() {
        let new_level = calculate_vegetation_lod(distance);
        if veg_lod.detail_level != new_level {
            veg_lod.detail_level = new_level;
            veg_lod.distance_to_player = distance;
            update_vegetation_components(&mut commands, entity, new_level);
}
fn calculate_vehicle_lod(distance: f32, config: &GameConfig) -> LodLevel {
    let lod_distances = config.world.lod_distances;
    match distance {
        d if d < lod_distances[0] => LodLevel::High,
        d if d < lod_distances[1] => LodLevel::Medium,
        _ => LodLevel::Sleep,
fn calculate_npc_lod(distance: f32, config: &GameConfig) -> LodLevel {
    let intervals = &config.npc.update_intervals;
        d if d < intervals.close_distance => LodLevel::High,
        d if d < intervals.far_distance => LodLevel::Medium,
fn calculate_vegetation_lod(distance: f32) -> VegetationDetailLevel {
        d if d < 50.0 => VegetationDetailLevel::Full,
        d if d < 150.0 => VegetationDetailLevel::Medium,
        d if d < 300.0 => VegetationDetailLevel::Billboard,
        _ => VegetationDetailLevel::Culled,
fn update_vehicle_components(commands: &mut Commands, entity: Entity, lod_level: LodLevel) {
    let mut entity_commands = commands.entity(entity);
    match lod_level {
        LodLevel::High => {
            // Full detail - all components active
            entity_commands.insert(HighDetailVehicle);
            entity_commands.remove::<SleepingEntity>();
        LodLevel::Medium => {
            // Medium detail - reduced physics
            entity_commands.remove::<HighDetailVehicle>();
        LodLevel::Sleep => {
            // Sleep mode - minimal components
            entity_commands.insert(SleepingEntity);
fn update_npc_components(commands: &mut Commands, entity: Entity, lod_level: LodLevel) {
            // Full detail - all AI systems active
            entity_commands.insert(HighDetailNPC);
            // Medium detail - reduced AI
            entity_commands.remove::<HighDetailNPC>();
            // Sleep mode - minimal AI
fn update_vegetation_components(commands: &mut Commands, entity: Entity, detail_level: VegetationDetailLevel) {
    match detail_level {
        VegetationDetailLevel::Full => {
            entity_commands.insert(FullDetailVegetation);
            entity_commands.remove::<BillboardVegetation>();
            entity_commands.remove::<CulledVegetation>();
        VegetationDetailLevel::Medium => {
            entity_commands.remove::<FullDetailVegetation>();
        VegetationDetailLevel::Billboard => {
            entity_commands.insert(BillboardVegetation);
        VegetationDetailLevel::Culled => {
            entity_commands.insert(CulledVegetation);
/// Resource-based performance monitoring system
pub fn lod_performance_monitoring_system(
    time: Res<Time>,
    vehicles: Query<&LodLevel, With<ActiveEntity>>,
    vegetation: Query<&VegetationLOD>,
    performance_counters.update_frame(time.delta_secs());
    // Count entities by LOD level
    let mut high_detail_count = 0;
    let mut medium_detail_count = 0;
    let mut sleep_count = 0;
    for lod in vehicles.iter() {
        match lod {
            LodLevel::High => high_detail_count += 1,
            LodLevel::Medium => medium_detail_count += 1,
            LodLevel::Sleep => sleep_count += 1,
    // Update entity counts
    performance_counters.entity_count = high_detail_count + medium_detail_count + sleep_count;
    performance_counters.culled_entities = sleep_count;
    // Reset per-frame counters at the end of the frame
    performance_counters.reset_per_frame_counters();
/// Marker components for LOD levels
#[derive(Component)]
pub struct HighDetailVehicle;
pub struct HighDetailNPC;
pub struct SleepingEntity;
pub struct FullDetailVegetation;
pub struct BillboardVegetation;
pub struct CulledVegetation;
/// Modern LOD plugin that replaces manual systems
pub struct ModernLODPlugin;
impl Plugin for ModernLODPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<game_core::config::performance_config::PerformanceCounters>()
            .add_systems(
                Update,
                (
                    modern_lod_system,
                    lod_performance_monitoring_system,
                ).in_set(LodSystemSet)
            );
