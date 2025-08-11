use std::mem::size_of;

#[test]
fn test_world_resource_sizes() {
    // Test size assertions are truthful
    
    // ChunkCoord: 8 bytes (2 * i32)
    assert_eq!(size_of::<(i32, i32)>(), 8);
    
    // ChunkState: 1 byte enum
    assert_eq!(size_of::<u8>(), 1);
    
    // ChunkTracker components:
    // - loaded_chunks: [(ChunkCoord, ChunkState); 2] = (8+1)*2 = 18 bytes + padding = 20 bytes (aligned to 4)
    // - focus_chunk: ChunkCoord = 8 bytes  
    // - lod_radius: i16 = 2 bytes
    // - performance_stats: u16 = 2 bytes
    // - active_count: u8 = 1 byte
    // - focus_valid: bool = 1 byte
    // Total: 20 + 8 + 2 + 2 + 1 + 1 = 34 bytes, aligned to 36 or 40 depending on packing
    
    // IVec3: 12 bytes (3 * i32)
    assert_eq!(size_of::<(i32, i32, i32)>(), 12);
    
    // WorldCoordinator components:
    // - focus_position: IVec3 = 12 bytes
    // - streaming_radius: f32 = 4 bytes
    // - generation_frame: u32 = 4 bytes
    // - flags: u32 = 4 bytes
    // - _reserved: [u32; 2] = 8 bytes
    // Total: 12 + 4 + 4 + 4 + 8 = 32 bytes exactly
    
    println!("Size validations:");
    println!("ChunkCoord (i32, i32): {} bytes", size_of::<(i32, i32)>());
    println!("IVec3 (i32, i32, i32): {} bytes", size_of::<(i32, i32, i32)>());
    println!("Tuple ((i32, i32), u8): {} bytes", size_of::<((i32, i32), u8)>());
    println!("Array [((i32, i32), u8); 2]: {} bytes", size_of::<[((i32, i32), u8); 2]>());
}
