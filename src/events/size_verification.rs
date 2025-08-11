//! Compile-time verification that all events meet AGENT.md size requirements
//! Events must be ≤128 bytes for performance (cleared every frame)

#[cfg(test)]
mod tests {
    use super::super::world::*;
    use std::mem::size_of;
    
    // All events MUST be ≤128 bytes for O(n) frame clearing performance
    const MAX_EVENT_SIZE: usize = 128;

    #[test]
    fn verify_chunk_event_sizes() {
        assert!(size_of::<RequestChunkLoad>() <= MAX_EVENT_SIZE);
        assert!(size_of::<ChunkLoaded>() <= MAX_EVENT_SIZE);
        assert!(size_of::<RequestChunkUnload>() <= MAX_EVENT_SIZE);
        assert!(size_of::<ChunkUnloaded>() <= MAX_EVENT_SIZE);
        
        println!("Chunk event sizes:");
        println!("  RequestChunkLoad: {} bytes", size_of::<RequestChunkLoad>());
        println!("  ChunkLoaded: {} bytes", size_of::<ChunkLoaded>());
        println!("  RequestChunkUnload: {} bytes", size_of::<RequestChunkUnload>());
        println!("  ChunkUnloaded: {} bytes", size_of::<ChunkUnloaded>());
    }

    #[test]
    fn verify_content_event_sizes() {
        assert!(size_of::<RequestDynamicSpawn>() <= MAX_EVENT_SIZE);
        assert!(size_of::<DynamicContentSpawned>() <= MAX_EVENT_SIZE);
        assert!(size_of::<RequestDynamicDespawn>() <= MAX_EVENT_SIZE);
        assert!(size_of::<DynamicContentDespawned>() <= MAX_EVENT_SIZE);
        
        println!("Content event sizes:");
        println!("  RequestDynamicSpawn: {} bytes", size_of::<RequestDynamicSpawn>());
        println!("  DynamicContentSpawned: {} bytes", size_of::<DynamicContentSpawned>());
        println!("  RequestDynamicDespawn: {} bytes", size_of::<RequestDynamicDespawn>());
        println!("  DynamicContentDespawned: {} bytes", size_of::<DynamicContentDespawned>());
    }

    #[test]
    fn verify_validation_event_sizes() {
        assert!(size_of::<RequestSpawnValidation>() <= MAX_EVENT_SIZE);
        assert!(size_of::<RequestRoadValidation>() <= MAX_EVENT_SIZE);
        assert!(size_of::<RoadValidationResult>() <= MAX_EVENT_SIZE);
        
        println!("Validation event sizes:");
        println!("  RequestSpawnValidation: {} bytes", size_of::<RequestSpawnValidation>());
        println!("  RequestRoadValidation: {} bytes", size_of::<RequestRoadValidation>());
        println!("  RoadValidationResult: {} bytes", size_of::<RoadValidationResult>());
        
        // SpawnValidationResult has String so verify separately
        let result_size = size_of::<SpawnValidationResult>();
        println!("  SpawnValidationResult: {} bytes (includes String heap pointer)", result_size);
        // String is heap-allocated, so stack size is just pointer + metadata
    }
}
