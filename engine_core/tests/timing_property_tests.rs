use proptest::prelude::*;
use engine_core::prelude::*;
use std::collections::HashMap;

const EPSILON: f32 = 1e-6;
const MAX_DELTA: f32 = 1.0 / 30.0; // 30 FPS minimum
const MIN_DELTA: f32 = 1e-6;

fn delta_time_strategy() -> impl Strategy<Value = f32> {
    (MIN_DELTA..MAX_DELTA).prop_filter("valid delta", |&d| d.is_finite() && d > 0.0)
}

fn delta_sequence_strategy() -> impl Strategy<Value = Vec<f32>> {
    prop::collection::vec(delta_time_strategy(), 1..1000)
}

fn interval_strategy() -> impl Strategy<Value = f32> {
    (0.01f32..10.0f32).prop_filter("valid interval", |&i| i.is_finite() && i > 0.0)
}

fn entity_id_strategy() -> impl Strategy<Value = EntityId> {
    any::<u64>().prop_filter("valid entity id", |&id| id > 0)
}

proptest! {
    #[test]
    fn test_delta_time_accumulation_no_drift(deltas in delta_sequence_strategy()) {
        let mut timing_service = TimingService::new();
        let mut expected_time = 0.0f64; // Use double precision for reference
        
        for delta in &deltas {
            timing_service.update_time(*delta);
            expected_time += *delta as f64;
        }
        
        // Check that accumulated time is within reasonable bounds
        let accumulated_time = timing_service.current_time as f64;
        let drift = (accumulated_time - expected_time).abs();
        
        // Allow for some floating point error, but not significant drift
        let max_allowed_drift = deltas.len() as f64 * f32::EPSILON as f64;
        prop_assert!(drift <= max_allowed_drift, 
            "Drift too large: {} > {}, accumulated: {}, expected: {}", 
            drift, max_allowed_drift, accumulated_time, expected_time);
    }
    
    #[test]
    fn test_timing_service_monotonic_time(deltas in delta_sequence_strategy()) {
        let mut timing_service = TimingService::new();
        let mut previous_time = 0.0;
        
        for delta in &deltas {
            timing_service.update_time(*delta);
            
            // Time should always increase
            prop_assert!(timing_service.current_time > previous_time);
            prop_assert_eq!(timing_service.delta_time, *delta);
            
            previous_time = timing_service.current_time;
        }
    }
    
    #[test]
    fn test_system_throttling_accuracy(
        deltas in delta_sequence_strategy(),
        interval in interval_strategy()
    ) {
        let mut timing_service = TimingService::new();
        timing_service.vehicle_lod_interval = interval;
        
        let mut last_run_time = 0.0;
        let mut run_count = 0;
        
        for delta in &deltas {
            timing_service.update_time(*delta);
            
            if timing_service.should_run_system(SystemType::VehicleLOD) {
                let time_since_last = timing_service.current_time - last_run_time;
                
                // Should only run after interval has passed
                if run_count > 0 {
                    prop_assert!(time_since_last >= interval - EPSILON, 
                        "System ran too early: {} < {}", time_since_last, interval);
                }
                
                last_run_time = timing_service.current_time;
                run_count += 1;
            }
        }
    }
    
    #[test]
    fn test_entity_timer_precision(
        entity_id in entity_id_strategy(),
        interval in interval_strategy(),
        deltas in delta_sequence_strategy()
    ) {
        let mut timing_service = TimingService::new();
        timing_service.register_entity(entity_id, EntityTimerType::VehicleLOD, interval);
        
        let mut last_update_time = 0.0;
        let mut update_count = 0;
        
        for delta in &deltas {
            timing_service.update_time(*delta);
            
            if timing_service.should_update_entity(entity_id) {
                let time_since_last = timing_service.current_time - last_update_time;
                
                // Should only update after interval has passed
                if update_count > 0 {
                    prop_assert!(time_since_last >= interval - EPSILON,
                        "Entity updated too early: {} < {}", time_since_last, interval);
                }
                
                last_update_time = timing_service.current_time;
                update_count += 1;
            }
        }
    }
    
    #[test]
    fn test_multiple_entity_timers_independence(
        entity_ids in prop::collection::vec(entity_id_strategy(), 1..10),
        intervals in prop::collection::vec(interval_strategy(), 1..10),
        deltas in delta_sequence_strategy()
    ) {
        prop_assume!(entity_ids.len() == intervals.len());
        
        let mut timing_service = TimingService::new();
        let mut last_update_times = HashMap::new();
        
        // Register all entities
        for (entity_id, interval) in entity_ids.iter().zip(intervals.iter()) {
            timing_service.register_entity(*entity_id, EntityTimerType::VehicleLOD, *interval);
            last_update_times.insert(*entity_id, 0.0);
        }
        
        for delta in &deltas {
            timing_service.update_time(*delta);
            
            for (entity_id, interval) in entity_ids.iter().zip(intervals.iter()) {
                if timing_service.should_update_entity(*entity_id) {
                    let last_time = last_update_times[entity_id];
                    let time_since_last = timing_service.current_time - last_time;
                    
                    if last_time > 0.0 {
                        prop_assert!(time_since_last >= *interval - EPSILON,
                            "Entity {} updated too early: {} < {}", 
                            entity_id, time_since_last, *interval);
                    }
                    
                    last_update_times.insert(*entity_id, timing_service.current_time);
                }
            }
        }
    }
    
    #[test]
    fn test_timer_cleanup_correctness(
        entity_ids in prop::collection::vec(entity_id_strategy(), 1..50),
        max_age in (0.1f32..5.0f32),
        deltas in delta_sequence_strategy()
    ) {
        let mut timing_service = TimingService::new();
        
        // Register entities at different times
        for (i, entity_id) in entity_ids.iter().enumerate() {
            if i < deltas.len() {
                // Advance time to spread out registrations
                for delta in &deltas[0..i] {
                    timing_service.update_time(*delta);
                }
            }
            timing_service.register_entity(*entity_id, EntityTimerType::VehicleLOD, 0.1);
        }
        
        // Advance time significantly
        for delta in &deltas {
            timing_service.update_time(*delta);
        }
        
        let timers_before = timing_service.get_entity_timers().len();
        timing_service.cleanup_old_timers(max_age);
        let timers_after = timing_service.get_entity_timers().len();
        
        // Should have cleaned up some timers if time has passed
        if timing_service.current_time > max_age {
            prop_assert!(timers_after <= timers_before);
        }
        
        // Verify remaining timers are actually recent
        for (_, timer) in timing_service.get_entity_timers() {
            let age = timing_service.current_time - timer.last_update;
            prop_assert!(age < max_age, "Timer not cleaned up: age {} >= max_age {}", age, max_age);
        }
    }
    
    #[test]
    fn test_system_type_consistency(
        deltas in delta_sequence_strategy(),
        interval in interval_strategy()
    ) {
        let mut timing_service = TimingService::new();
        timing_service.vehicle_lod_interval = interval;
        timing_service.npc_lod_interval = interval;
        
        let mut vehicle_runs = 0;
        let mut npc_runs = 0;
        
        for delta in &deltas {
            timing_service.update_time(*delta);
            
            if timing_service.should_run_system(SystemType::VehicleLOD) {
                vehicle_runs += 1;
            }
            if timing_service.should_run_system(SystemType::NPCLOD) {
                npc_runs += 1;
            }
        }
        
        // Both systems should run approximately the same number of times
        // since they have the same interval
        let run_difference = (vehicle_runs as i32 - npc_runs as i32).abs();
        prop_assert!(run_difference <= 1, 
            "System run counts too different: vehicle={}, npc={}", 
            vehicle_runs, npc_runs);
    }
    
    #[test]
    fn test_timing_precision_under_stress(
        deltas in prop::collection::vec(delta_time_strategy(), 10000..50000)
    ) {
        let mut timing_service = TimingService::new();
        let mut theoretical_time = 0.0f64;
        
        // Stress test with many small updates
        for delta in &deltas {
            timing_service.update_time(*delta);
            theoretical_time += *delta as f64;
        }
        
        let actual_time = timing_service.current_time as f64;
        let relative_error = ((actual_time - theoretical_time) / theoretical_time).abs();
        
        // Should maintain precision even under stress
        prop_assert!(relative_error < 1e-6, 
            "Timing precision degraded under stress: relative error {}", 
            relative_error);
    }
    
    #[test]
    fn test_zero_delta_handling(
        normal_deltas in prop::collection::vec(delta_time_strategy(), 1..100),
        zero_count in 1usize..10
    ) {
        let mut timing_service = TimingService::new();
        let mut time_before_zeros = 0.0;
        
        // Run some normal updates
        for delta in &normal_deltas {
            timing_service.update_time(*delta);
        }
        time_before_zeros = timing_service.current_time;
        
        // Apply zero deltas
        for _ in 0..zero_count {
            timing_service.update_time(0.0);
        }
        
        // Time should not change with zero deltas
        prop_assert_eq!(timing_service.current_time, time_before_zeros);
        prop_assert_eq!(timing_service.delta_time, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_timing_operations() {
        let mut timing_service = TimingService::new();
        
        // Basic time update
        timing_service.update_time(0.016);
        assert_eq!(timing_service.current_time, 0.016);
        assert_eq!(timing_service.delta_time, 0.016);
        
        // System throttling
        timing_service.vehicle_lod_interval = 0.1;
        let should_run_first = timing_service.should_run_system(SystemType::VehicleLOD);
        let should_run_second = timing_service.should_run_system(SystemType::VehicleLOD);
        
        assert!(should_run_first);  // First call should run
        assert!(!should_run_second); // Second call should not run (too soon)
        
        // Entity timer
        timing_service.register_entity(123, EntityTimerType::VehicleLOD, 0.1);
        let should_update = timing_service.should_update_entity(123);
        assert!(should_update); // First update should work
        
        // Cleanup
        timing_service.cleanup_old_timers(0.0);
        assert!(timing_service.get_entity_timers().is_empty());
    }
    
    #[test]
    fn test_timing_service_defaults() {
        let timing_service = TimingService::new();
        
        assert_eq!(timing_service.current_time, 0.0);
        assert_eq!(timing_service.delta_time, 0.0);
        assert_eq!(timing_service.vehicle_lod_interval, 0.1);
        assert_eq!(timing_service.npc_lod_interval, 0.1);
        assert_eq!(timing_service.audio_cleanup_interval, 1.0);
        assert_eq!(timing_service.effect_update_interval, 0.05);
        assert!(timing_service.get_entity_timers().is_empty());
    }
    
    #[test]
    fn test_entity_timer_types() {
        let timer1 = EntityTimer {
            last_update: 0.0,
            interval: 0.1,
            timer_type: EntityTimerType::VehicleLOD,
        };
        
        let timer2 = EntityTimer {
            last_update: 0.0,
            interval: 0.1,
            timer_type: EntityTimerType::Custom("test".to_string()),
        };
        
        assert_eq!(timer1.timer_type, EntityTimerType::VehicleLOD);
        assert_eq!(timer2.timer_type, EntityTimerType::Custom("test".to_string()));
    }
    
    #[test]
    fn test_system_types() {
        let systems = vec![
            SystemType::VehicleLOD,
            SystemType::NPCLOD,
            SystemType::AudioUpdate,
            SystemType::EffectUpdate,
            SystemType::Custom("test".to_string()),
        ];
        
        for system in systems {
            match system {
                SystemType::Custom(name) => assert_eq!(name, "test"),
                _ => {} // Other variants are valid
            }
        }
    }
}
