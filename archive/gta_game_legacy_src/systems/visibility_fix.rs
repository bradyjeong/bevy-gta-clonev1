//! ───────────────────────────────────────────────
//! System:   Visibility Fix
//! Purpose:  Manages entity visibility fixes
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;

/// System that automatically adds InheritedVisibility to child entities that are missing it
/// This runs once during startup to fix any entities that need visibility inheritance
pub fn fix_missing_inherited_visibility(
    mut commands: Commands,
    // Find entities that have a Parent but no InheritedVisibility
    entities_missing_visibility: Query<Entity, (With<Parent>, Without<InheritedVisibility>)>,
) {
    for entity in entities_missing_visibility.iter() {
        commands.entity(entity).insert(InheritedVisibility::VISIBLE);
        info!("Added InheritedVisibility to entity {:?}", entity);
    }
}

/// System that ensures parent entities have proper visibility components
pub fn fix_parent_visibility(
    mut commands: Commands,
    // Find entities that have children but incomplete visibility
    parents_missing_visibility: Query<Entity, (With<Children>, Or<(Without<Visibility>, Without<InheritedVisibility>, Without<ViewVisibility>)>)>,
) {
    for entity in parents_missing_visibility.iter() {
        commands.entity(entity).insert((
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ));
        info!("Added complete visibility bundle to parent entity {:?}", entity);
    }
}
