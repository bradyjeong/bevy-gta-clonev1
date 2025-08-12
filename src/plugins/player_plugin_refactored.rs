use bevy::prelude::*;
use crate::events::cross_plugin_events::*;
use crate::game_state::GameState;
use crate::components::player::*;
use crate::components::control_state::ControlState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all cross-plugin events
            .add_event::<RequestMovementInput>()
            .add_event::<MovementStateUpdate>()
            .add_event::<RequestCameraFollow>()
            .add_event::<RequestFootstepSound>()
            .add_event::<RequestAudioCleanup>()
            .add_event::<RequestInteractionCheck>()
            .add_event::<InteractionAvailable>()
            .add_event::<RequestEmotionUpdate>()
            .add_event::<RequestFidgetAnimation>()
            
            // Player-specific systems that don't cross plugin boundaries
            .add_systems(Update, (
                // Send events to other plugins instead of calling systems directly
                send_movement_request_system.run_if(in_state(GameState::Walking)),
                handle_movement_updates_system.run_if(in_state(GameState::Walking)),
                send_camera_follow_request_system,
                send_audio_requests_system.run_if(in_state(GameState::Walking)),
                send_interaction_check_system,
                handle_interaction_available_system,
                send_behavior_requests_system.run_if(in_state(GameState::Walking)),
            ));
    }
}

/// Send movement input requests to the movement plugin
fn send_movement_request_system(
    mut events: EventWriter<RequestMovementInput>,
    query: Query<(Entity, &ControlState), With<HumanPlayer>>,
) {
    for (entity, control) in query.iter() {
        events.send(RequestMovementInput {
            entity,
            forward: control.move_forward,
            right: control.move_right,
            run: control.run,
            jump: control.jump,
        });
    }
}

/// Handle movement state updates from the movement plugin
fn handle_movement_updates_system(
    mut events: EventReader<MovementStateUpdate>,
    mut query: Query<(&mut Stamina, &mut AnimationFlags), With<HumanPlayer>>,
) {
    for event in events.read() {
        if let Ok((mut stamina, mut anim_flags)) = query.get_mut(event.entity) {
            stamina.current = event.stamina;
            anim_flags.is_moving = event.is_moving;
            anim_flags.is_running = event.is_running;
        }
    }
}

/// Send camera follow requests to the camera plugin
fn send_camera_follow_request_system(
    mut events: EventWriter<RequestCameraFollow>,
    query: Query<Entity, With<HumanPlayer>>,
) {
    for entity in query.iter() {
        events.send(RequestCameraFollow {
            target: entity,
            offset: Vec3::new(0.0, 2.5, -5.0),
            smooth_factor: 0.1,
        });
    }
}

/// Send audio requests based on player movement
fn send_audio_requests_system(
    mut footstep_events: EventWriter<RequestFootstepSound>,
    mut cleanup_events: EventWriter<RequestAudioCleanup>,
    query: Query<(Entity, &Transform, &AnimationFlags), With<HumanPlayer>>,
    time: Res<Time>,
    mut last_footstep: Local<f32>,
) {
    *last_footstep += time.delta_secs();
    
    for (entity, transform, anim_flags) in query.iter() {
        if anim_flags.is_moving && *last_footstep > 0.4 {
            footstep_events.send(RequestFootstepSound {
                entity,
                position: transform.translation,
                is_running: anim_flags.is_running,
                surface_type: SurfaceType::Concrete, // Could determine from ground check
            });
            *last_footstep = 0.0;
        }
        
        // Cleanup sounds when stopping
        if !anim_flags.is_moving {
            cleanup_events.send(RequestAudioCleanup { entity });
        }
    }
}

/// Send interaction check requests
fn send_interaction_check_system(
    mut events: EventWriter<RequestInteractionCheck>,
    query: Query<(Entity, &Transform), With<HumanPlayer>>,
) {
    for (entity, transform) in query.iter() {
        events.send(RequestInteractionCheck {
            entity,
            position: transform.translation,
            interaction_range: 3.0,
        });
    }
}

/// Handle interaction availability responses
fn handle_interaction_available_system(
    mut events: EventReader<InteractionAvailable>,
    mut query: Query<&mut InteractionState, With<HumanPlayer>>,
) {
    for event in events.read() {
        if let Ok(mut interaction_state) = query.get_mut(event.entity) {
            interaction_state.available_target = Some(event.target);
            interaction_state.interaction_type = Some(event.interaction_type);
        }
    }
}

/// Send behavior update requests
fn send_behavior_requests_system(
    mut emotion_events: EventWriter<RequestEmotionUpdate>,
    mut fidget_events: EventWriter<RequestFidgetAnimation>,
    query: Query<(Entity, &AnimationFlags), With<HumanPlayer>>,
    time: Res<Time>,
    mut fidget_timer: Local<f32>,
) {
    *fidget_timer += time.delta_secs();
    
    for (entity, anim_flags) in query.iter() {
        // Update emotions based on activity
        let trigger = if anim_flags.is_moving {
            EmotionTrigger::Movement
        } else {
            EmotionTrigger::Idle
        };
        
        emotion_events.send(RequestEmotionUpdate {
            entity,
            emotion_delta: time.delta_secs(),
            trigger,
        });
        
        // Request fidget animations when idle
        if !anim_flags.is_moving && *fidget_timer > 10.0 {
            fidget_events.send(RequestFidgetAnimation {
                entity,
                fidget_type: FidgetType::LookAround,
            });
            *fidget_timer = 0.0;
        }
    }
}

// Helper components for interaction state (if not already defined)
#[derive(Component, Default)]
pub struct InteractionState {
    pub available_target: Option<Entity>,
    pub interaction_type: Option<InteractionType>,
}

#[derive(Component, Default)]
pub struct AnimationFlags {
    pub is_moving: bool,
    pub is_running: bool,
    pub is_jumping: bool,
}

#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}
