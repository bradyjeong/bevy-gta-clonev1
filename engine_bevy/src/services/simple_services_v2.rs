use bevy::prelude::*;

/// Simple configuration service - stores arbitrary config data
#[derive(Resource)]
pub struct ConfigService {
    data: std::collections::HashMap<String, String>,
}

impl ConfigService {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
    
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple physics validation service
#[derive(Resource)]
pub struct PhysicsService {
    pub max_velocity: f32,
    pub max_world_coord: f32,
    pub min_world_coord: f32,
    pub max_mass: f32,
    pub min_mass: f32,
    pub max_collider_size: f32,
    pub min_collider_size: f32,
}

impl PhysicsService {
    pub fn new() -> Self {
        Self {
            max_velocity: 500.0,
            max_world_coord: 10000.0,
            min_world_coord: -10000.0,
            max_mass: 50000.0,
            min_mass: 0.1,
            max_collider_size: 100.0,
            min_collider_size: 0.01,
        }
    }
    
    pub fn validate_position(&self, position: Vec3) -> Vec3 {
        Vec3::new(
            position.x.clamp(self.min_world_coord, self.max_world_coord),
            position.y.clamp(self.min_world_coord, self.max_world_coord),
            position.z.clamp(self.min_world_coord, self.max_world_coord),
        )
    }
    
    pub fn validate_velocity(&self, velocity: Vec3) -> Vec3 {
        let speed = velocity.length();
        if speed > self.max_velocity {
            velocity.normalize() * self.max_velocity
        } else {
            velocity
        }
    }
    
    pub fn validate_mass(&self, mass: f32) -> f32 {
        mass.clamp(self.min_mass, self.max_mass)
    }
    
    pub fn validate_collider_size(&self, size: Vec3) -> Vec3 {
        Vec3::new(
            size.x.clamp(self.min_collider_size, self.max_collider_size),
            size.y.clamp(self.min_collider_size, self.max_collider_size),
            size.z.clamp(self.min_collider_size, self.max_collider_size),
        )
    }
}

impl Default for PhysicsService {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timing service
#[derive(Resource)]
pub struct TimingService {
    pub current_time: f32,
    pub delta_time: f32,
    system_intervals: std::collections::HashMap<String, f32>,
    last_run_times: std::collections::HashMap<String, f32>,
}

impl TimingService {
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            delta_time: 0.0,
            system_intervals: std::collections::HashMap::new(),
            last_run_times: std::collections::HashMap::new(),
        }
    }
    
    pub fn update_time(&mut self, time: &Time) {
        self.current_time = time.elapsed_secs();
        self.delta_time = time.delta_secs();
    }
    
    pub fn set_system_interval(&mut self, system_name: String, interval: f32) {
        self.system_intervals.insert(system_name, interval);
    }
    
    pub fn should_run_system(&mut self, system_name: &str) -> bool {
        let interval = self.system_intervals.get(system_name).copied().unwrap_or(0.0);
        if interval <= 0.0 {
            return true; // Run every frame
        }
        
        let last_run = self.last_run_times.get(system_name).copied().unwrap_or(0.0);
        if self.current_time - last_run >= interval {
            self.last_run_times.insert(system_name.to_string(), self.current_time);
            true
        } else {
            false
        }
    }
}

impl Default for TimingService {
    fn default() -> Self {
        Self::new()
    }
}

/// System to initialize services
pub fn initialize_simple_services_v2(mut commands: Commands) {
    commands.insert_resource(ConfigService::new());
    commands.insert_resource(PhysicsService::new());
    commands.insert_resource(TimingService::new());
    
    info!("✅ SIMPLE SERVICES V2: Initialized config, physics, and timing services");
}

/// System to update timing service
pub fn update_timing_service_v2(
    mut timing_service: ResMut<TimingService>,
    time: Res<Time>,
) {
    timing_service.update_time(&time);
}
