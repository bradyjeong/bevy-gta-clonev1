use bevy::prelude::*;
use std::cell::RefCell;
use game_core::prelude::*;
// Removed bevy16_compat - using direct Bevy methods
use rand::Rng;

thread_local! {
    static BEHAVIOR_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

#[derive(Component)]
pub struct HumanEmotions {
    pub stress_level: f32,
    pub energy_level: f32,
    pub mood: Mood,
    pub last_mood_change: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
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

/// System to update human emotional state based on actions and environment
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
    
    // Update behavior based on emotional state
    behavior.confidence_level = match emotions.mood {
        Mood::Confident => (emotions.energy_level / 100.0).clamp(0.7, 1.0),
        Mood::Anxious => (emotions.energy_level / 200.0).clamp(0.1, 0.4),
        Mood::Tired => (emotions.energy_level / 150.0).clamp(0.2, 0.6),
        Mood::Excited => (emotions.energy_level / 120.0).clamp(0.6, 0.9),
        Mood::Calm => (emotions.energy_level / 100.0).clamp(0.4, 0.8),
    };
    
    // Update movement patterns based on mood
    movement.max_speed = match emotions.mood {
        Mood::Confident => 1.4,
        Mood::Excited => 1.6,
        Mood::Anxious => 0.8,
        Mood::Tired => 0.6,
        Mood::Calm => 1.0,
    };
}

/// System to handle fidgeting and micro-animations
pub fn human_fidget_system(
    time: Res<Time>,
    mut behavior_query: Query<(&mut HumanBehavior, &mut HumanAnimation), With<HumanEmotions>>,
) {
    for (behavior, mut animation) in &mut behavior_query {
        // Update fidget timer
        animation.next_fidget_time -= time.delta_secs();
        
        if animation.next_fidget_time <= 0.0 {
            // Trigger a fidget animation
            // Fidgeting is reflected in higher step frequency
            animation.step_frequency *= 1.2;
            
            // Set next fidget time based on confidence level
            let base_fidget_time = match behavior.confidence_level {
                level if level > 0.8 => BEHAVIOR_RNG.with(|rng| rng.borrow_mut().gen_range(5.0..10.0)),
                level if level > 0.6 => BEHAVIOR_RNG.with(|rng| rng.borrow_mut().gen_range(3.0..7.0)),
                _ => BEHAVIOR_RNG.with(|rng| rng.borrow_mut().gen_range(2.0..5.0)), // Anxious people fidget more
            };
            animation.next_fidget_time = base_fidget_time;
            // In a full implementation, this would trigger head turns, 
            // shoulder adjustments, weight shifting, etc.
        }
    }
}
