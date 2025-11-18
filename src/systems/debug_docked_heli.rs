use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::components::{DockedOnYacht, Helicopter};

#[derive(Resource, Default)]
pub struct DockedHeliAudit {
    last_transforms: std::collections::HashMap<Entity, (Vec3, Quat)>,
}

#[allow(clippy::type_complexity)]
pub fn audit_docked_helicopter_movement(
    mut audit: Local<DockedHeliAudit>,
    docked_query: Query<
        (Entity, &Transform, Option<&Velocity>),
        (With<Helicopter>, With<DockedOnYacht>),
    >,
) {
    for (entity, transform, velocity) in docked_query.iter() {
        let current_pos = transform.translation;
        let current_rot = transform.rotation;

        if let Some(velocity) = velocity {
            error!(
                "AUDIT: Docked helicopter {:?} still has Velocity component! linvel={:?}, angvel={:?}",
                entity, velocity.linvel, velocity.angvel
            );
        }

        if let Some((last_pos, last_rot)) = audit.last_transforms.get(&entity) {
            let pos_delta = current_pos.distance(*last_pos);
            let rot_delta = current_rot.angle_between(*last_rot);

            if pos_delta > 0.001 || rot_delta > 0.001 {
                warn!(
                    "AUDIT: Docked helicopter {:?} Transform changed! pos_delta={:.4}, rot_delta={:.4}, new_pos={:?}",
                    entity, pos_delta, rot_delta, current_pos
                );
            }
        }

        audit
            .last_transforms
            .insert(entity, (current_pos, current_rot));
    }
}
