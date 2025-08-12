use bevy::prelude::*;

// ===== Movement Plugin Events =====

/// Request movement input processing (Player → Movement)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestMovementInput {
    pub entity: Entity,
    pub forward: f32,
    pub right: f32,
    pub run: bool,
    pub jump: bool,
}

/// Movement state update (Movement → Player)
#[derive(Event, Debug, Clone, Copy)]
pub struct MovementStateUpdate {
    pub entity: Entity,
    pub velocity: Vec3,
    pub is_moving: bool,
    pub is_running: bool,
    pub stamina: f32,
}

/// Request vehicle movement (Vehicle → Movement)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestVehicleMovement {
    pub entity: Entity,
    pub vehicle_type: VehicleType,
    pub throttle: f32,
    pub steering: f32,
    pub brake: f32,
    pub special: bool, // turbo/afterburner
}

#[derive(Debug, Clone, Copy)]
pub enum VehicleType {
    Car,
    Helicopter,
    F16,
    Yacht,
}

// ===== Camera Plugin Events =====

/// Request camera follow (Player → Camera)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestCameraFollow {
    pub target: Entity,
    pub offset: Vec3,
    pub smooth_factor: f32,
}

// ===== Audio Plugin Events =====

/// Request footstep sound (Player → Audio)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestFootstepSound {
    pub entity: Entity,
    pub position: Vec3,
    pub is_running: bool,
    pub surface_type: SurfaceType,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceType {
    Concrete,
    Grass,
    Metal,
    Wood,
}

/// Request audio cleanup (Player → Audio)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestAudioCleanup {
    pub entity: Entity,
}

// ===== Interaction Plugin Events =====

/// Request interaction check (Player → Interaction)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestInteractionCheck {
    pub entity: Entity,
    pub position: Vec3,
    pub interaction_range: f32,
}

/// Interaction available (Interaction → Player/UI)
#[derive(Event, Debug, Clone, Copy)]
pub struct InteractionAvailable {
    pub entity: Entity,
    pub target: Entity,
    pub interaction_type: InteractionType,
}

#[derive(Debug, Clone, Copy)]
pub enum InteractionType {
    Vehicle,
    NPC,
    Item,
    Door,
}

// ===== Effects Plugin Events =====

/// Request exhaust effect (Vehicle → Effects)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestExhaustEffect {
    pub entity: Entity,
    pub intensity: f32,
    pub position: Vec3,
    pub direction: Vec3,
}

/// Request jet flame update (Vehicle → Effects)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestJetFlameUpdate {
    pub entity: Entity,
    pub throttle: f32,
    pub afterburner: bool,
}

/// Request waypoint update (UI → Effects)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestWaypointUpdate {
    pub entity: Entity,
    pub position: Vec3,
    pub visible: bool,
}

/// Request beacon visibility (Debug → Effects)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestBeaconVisibility {
    pub entity: Entity,
    pub visible: bool,
}

// ===== Human Behavior Events =====

/// Request emotion update (Player → HumanBehavior)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestEmotionUpdate {
    pub entity: Entity,
    pub emotion_delta: f32,
    pub trigger: EmotionTrigger,
}

#[derive(Debug, Clone, Copy)]
pub enum EmotionTrigger {
    Idle,
    Movement,
    Interaction,
    Danger,
}

/// Request fidget animation (Player → HumanBehavior)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestFidgetAnimation {
    pub entity: Entity,
    pub fidget_type: FidgetType,
}

#[derive(Debug, Clone, Copy)]
pub enum FidgetType {
    LookAround,
    Stretch,
    Yawn,
    CheckPhone,
}
