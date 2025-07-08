use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


use crate::game_state::GameState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // Quaternion as [x, y, z, w]
    pub scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
        }
    }
}

impl Into<Transform> for SerializableTransform {
    fn into(self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_xyzw(
                self.rotation[0],
                self.rotation[1],
                self.rotation[2],
                self.rotation[3],
            ),
            scale: Vec3::from_array(self.scale),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveGameState {
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub game_state: GameState,
    pub active_entity_id: Option<u32>,
    pub world_seed: Option<u64>,
    pub play_time: f64,
}

const SAVE_VERSION: u32 = 1;

impl SaveGameState {
    pub fn validate(&self) -> Result<(), String> {
        // Version compatibility check
        if self.version > SAVE_VERSION {
            return Err(format!("Save version {} is too new (current: {})", self.version, SAVE_VERSION));
        }

        Ok(())
    }
}
