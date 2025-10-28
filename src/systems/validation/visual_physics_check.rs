use crate::components::VisualOnly;
use bevy::log::tracing;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Visual-Physics Separation Validation System
///
/// Ensures proper separation of visual and physics rigs following GTA-lite pattern:
/// - Parent entities have physics (RigidBody, Collider, Velocity)
/// - Visual children marked with VisualOnly must NOT have physics components
///
/// This system runs once at startup in debug mode to catch configuration errors.
#[allow(clippy::too_many_arguments)]
pub fn validate_visual_physics_separation(
    visual_query: Query<(Entity, Option<&Name>), With<VisualOnly>>,
    rigidbody_query: Query<&RigidBody>,
    collider_query: Query<&Collider>,
    velocity_query: Query<&Velocity>,
    external_force_query: Query<&ExternalForce>,
    damping_query: Query<&Damping>,
    ccd_query: Query<&Ccd>,
    sleeping_query: Query<&Sleeping>,
    locked_axes_query: Query<&LockedAxes>,
    collision_groups_query: Query<&CollisionGroups>,
) {
    let mut violations = 0;

    for (entity, name) in visual_query.iter() {
        let entity_name = name.map(|n| n.as_str()).unwrap_or("Unnamed");

        if rigidbody_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + RigidBody (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if collider_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + Collider (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if velocity_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + Velocity (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if external_force_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + ExternalForce (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if damping_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + Damping (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if ccd_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + Ccd (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if sleeping_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + Sleeping (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if locked_axes_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + LockedAxes (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }

        if collision_groups_query.contains(entity) {
            tracing::error!(
                target: "validation::visual_physics",
                "VIOLATION: Entity {:?} '{}' has VisualOnly + CollisionGroups (physics on visual-only entity)",
                entity,
                entity_name
            );
            violations += 1;
        }
    }

    if violations > 0 {
        tracing::warn!(
            target: "validation::visual_physics",
            "Visual-Physics Separation: Found {} violations. Visual-only entities should not have physics components.",
            violations
        );

        #[cfg(any(test, debug_assertions))]
        debug_assert!(
            violations == 0,
            "Visual-Physics Separation VIOLATED: Found {violations} entities marked VisualOnly with physics components.\n\
             This breaks the visual-physics rig separation pattern (see VISUAL_PHYSICS_SEPARATION.md).\n\
             Common causes:\n\
             1. Added RigidBody/Collider to visual child entity (wheels, rotors, mesh children)\n\
             2. Forgot to mark entity with VisualOnly component\n\
             3. Physics components should ONLY be on parent vehicle entity\n\
             Fix: Move physics components to parent entity, ensure visual children use VisibleChildBundle.\n\
             Run with --features debug-ui and press F3 to inspect entity hierarchy."
        );
    } else {
        tracing::info!(
            target: "validation::visual_physics",
            "Visual-Physics Separation: ✓ All visual-only entities properly separated from physics"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::{App, Startup};
    use bevy_rapier3d::plugin::RapierPhysicsPlugin;

    #[test]
    fn test_visual_only_no_physics() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .register_type::<VisualOnly>()
            .add_systems(Startup, validate_visual_physics_separation);

        let visual_entity = app
            .world_mut()
            .spawn((VisualOnly, Transform::default(), Name::new("TestVisual")))
            .id();

        app.update();

        assert!(!app.world().entity(visual_entity).contains::<RigidBody>());
        assert!(!app.world().entity(visual_entity).contains::<Collider>());
    }

    #[test]
    #[should_panic(expected = "VIOLATION")]
    fn test_visual_only_with_rigidbody_fails() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RapierPhysicsPlugin::<()>::default())
            .add_systems(Startup, validate_visual_physics_separation);

        app.world_mut().spawn((
            VisualOnly,
            RigidBody::Dynamic,
            Transform::default(),
            Name::new("InvalidVisual"),
        ));

        app.update();
    }

    #[test]
    #[should_panic(expected = "VIOLATION")]
    fn test_visual_only_with_collider_fails() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RapierPhysicsPlugin::<()>::default())
            .add_systems(Startup, validate_visual_physics_separation);

        app.world_mut().spawn((
            VisualOnly,
            Collider::cuboid(1.0, 1.0, 1.0),
            Transform::default(),
            Name::new("InvalidVisualCollider"),
        ));

        app.update();
    }

    #[test]
    #[should_panic(expected = "VIOLATION")]
    fn test_visual_only_with_velocity_fails() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RapierPhysicsPlugin::<()>::default())
            .add_systems(Startup, validate_visual_physics_separation);

        app.world_mut().spawn((
            VisualOnly,
            Velocity::default(),
            Transform::default(),
            Name::new("InvalidVisualVelocity"),
        ));

        app.update();
    }
}
