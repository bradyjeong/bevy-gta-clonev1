use bevy::prelude::*;
use crate::events::cross_plugin_events::*;

/// Handle camera follow requests from other plugins
pub fn handle_camera_follow_request_system(
    mut events: EventReader<RequestCameraFollow>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<FollowTarget>)>,
    target_query: Query<&Transform, (With<FollowTarget>, Without<Camera3d>)>,
    mut follow_state: ResMut<CameraFollowState>,
) {
    for event in events.read() {
        // Update follow state
        follow_state.target = Some(event.target);
        follow_state.offset = event.offset;
        follow_state.smooth_factor = event.smooth_factor;
        
        // Apply immediate follow if target exists
        if let Ok(target_transform) = target_query.get(event.target) {
            for mut camera_transform in camera_query.iter_mut() {
                let desired_pos = target_transform.translation + event.offset;
                camera_transform.translation = camera_transform.translation.lerp(
                    desired_pos,
                    event.smooth_factor
                );
                camera_transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}

/// Continuous camera follow system using the stored state
pub fn camera_follow_update_system(
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<FollowTarget>)>,
    target_query: Query<&Transform, With<FollowTarget>>,
    follow_state: Res<CameraFollowState>,
    time: Res<Time>,
) {
    if let Some(target_entity) = follow_state.target {
        if let Ok(target_transform) = target_query.get(target_entity) {
            for mut camera_transform in camera_query.iter_mut() {
                let desired_pos = target_transform.translation + follow_state.offset;
                let smooth_factor = follow_state.smooth_factor * time.delta_secs() * 60.0; // Frame-rate independent
                
                camera_transform.translation = camera_transform.translation.lerp(
                    desired_pos,
                    smooth_factor.min(1.0)
                );
                camera_transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}

/// Resource to store camera follow state
#[derive(Resource, Default)]
pub struct CameraFollowState {
    pub target: Option<Entity>,
    pub offset: Vec3,
    pub smooth_factor: f32,
}

/// Marker component for entities that can be followed by the camera
#[derive(Component)]
pub struct FollowTarget;
