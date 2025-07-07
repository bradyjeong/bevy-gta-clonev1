// Emotional state components for testing (mirrors gameplay_sim implementation)
use bevy::prelude::*;

#[derive(Component, Clone, PartialEq)]
pub struct HumanEmotions {
    pub stress_level: f32,
    pub energy_level: f32,
    pub mood: Mood,
    pub last_mood_change: f32,
}
#[derive(Clone, PartialEq, Debug)]
pub enum Mood {
    Calm,
    Excited,
    Tired,
    Anxious,
    Confident,
impl Default for HumanEmotions {
    fn default() -> Self {
        Self {
            stress_level: 0.0,
            energy_level: 100.0,
            mood: Mood::Calm,
            last_mood_change: 0.0,
        }
    }
