use bevy::prelude::*;
use rand::Rng;
use crate::components::{Player, ActiveEntity, HumanAnimation, HumanBehavior, HumanMovement};

#[derive(Component)]
pub struct HumanEmotions {
    pub stress_level: f32,
    pub energy_level: f32,
    pub mood: Mood,
    pub last_mood_change: f32,
}

#[derive(Clone, PartialEq)]
pub enum Mood {
    Calm,
    Excited,
    Tired,
    Anxious,
    Confident,
}

impl Default for HumanEmotions {
    fn default() -> Self {
        Self {
            stress_level: 0.0,
            energy_level: 100.0,
            mood: Mood::Calm,
            last_mood_change: 0.0,
        }
    }
}

// System to update human emotional state based on actions and environment
pub fn human_emotional_state_system(
    time: Res<Time>,
    mut player_query: Query<
        (&mut HumanEmotions, &mut HumanBehavior, &mut HumanMovement, &HumanAnimation),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((mut emotions, mut behavior, mut movement, animation)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let mut rng = rand::thread_rng();

    // Update energy based on activity
    if animation.is_running {
        emotions.energy_level -= 20.0 * dt;
        emotions.stress_level += 10.0 * dt;
    } else if animation.is_walking {
        emotions.energy_level -= 5.0 * dt;
    } else {
        emotions.energy_level += 15.0 * dt; // Rest recovery
        emotions.stress_level -= 5.0 * dt;
    }

    // Clamp values
    emotions.energy_level = emotions.energy_level.clamp(0.0, 100.0);
    emotions.stress_level = emotions.stress_level.clamp(0.0, 100.0);

    // Determine mood based on energy and stress
    let new_mood = if emotions.energy_level < 20.0 {
        Mood::Tired
    } else if emotions.stress_level > 70.0 {
        Mood::Anxious
    } else if emotions.energy_level > 80.0 && emotions.stress_level < 30.0 {
        Mood::Confident
    } else if emotions.stress_level > 40.0 {
        Mood::Excited
    } else {
        Mood::Calm
    };

    // Update mood with some persistence
    if new_mood != emotions.mood && time.elapsed_secs() - emotions.last_mood_change > 5.0 {
        emotions.mood = new_mood;
        emotions.last_mood_change = time.elapsed_secs();
    }

    // Adjust behavior based on emotional state
    match emotions.mood {
        Mood::Tired => {
            behavior.personality_speed_modifier = 0.7;
            behavior.reaction_time = rng.gen_range(0.15..0.25);
            behavior.confidence_level = 0.6;
            movement.tired_speed_modifier = 0.5;
        }
        Mood::Anxious => {
            behavior.personality_speed_modifier = 1.3;
            behavior.reaction_time = rng.gen_range(0.03..0.08);
            behavior.confidence_level = 0.4;
            behavior.movement_variation = rng.gen_range(0.7..1.4);
        }
        Mood::Confident => {
            behavior.personality_speed_modifier = 1.1;
            behavior.reaction_time = rng.gen_range(0.05..0.1);
            behavior.confidence_level = 1.0;
            behavior.movement_variation = rng.gen_range(0.95..1.05);
        }
        Mood::Excited => {
            behavior.personality_speed_modifier = 1.2;
            behavior.reaction_time = rng.gen_range(0.04..0.09);
            behavior.confidence_level = 0.9;
            behavior.movement_variation = rng.gen_range(0.8..1.3);
        }
        Mood::Calm => {
            behavior.personality_speed_modifier = 1.0;
            behavior.reaction_time = rng.gen_range(0.08..0.12);
            behavior.confidence_level = 0.8;
            behavior.movement_variation = rng.gen_range(0.9..1.1);
        }
    }
}

// System to add subtle random behaviors like looking around, fidgeting
pub fn human_fidget_system(
    time: Res<Time>,
    mut player_query: Query<
        (&mut HumanAnimation, &HumanBehavior),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((mut animation, behavior)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    animation.idle_fidget_timer += dt;

    // Trigger fidgeting when idle and timer expires
    if !animation.is_walking && animation.idle_fidget_timer >= animation.next_fidget_time {
        // Reset timer and set next fidget time
        animation.idle_fidget_timer = 0.0;
        let mut rng = rand::thread_rng();
        
        // Vary fidget frequency based on personality
        let base_fidget_time = match behavior.confidence_level {
            level if level > 0.8 => rng.gen_range(5.0..10.0),
            level if level > 0.6 => rng.gen_range(3.0..7.0),
            _ => rng.gen_range(2.0..5.0), // Anxious people fidget more
        };
        
        animation.next_fidget_time = base_fidget_time;
        
        // In a full implementation, this would trigger head turns, 
        // shoulder adjustments, weight shifting, etc.
    }
}
