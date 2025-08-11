use std::mem::size_of;

#[test]
fn validate_struct_sizes() {
    // ChunkCoord should be 8 bytes
    println!("ChunkCoord size: {}", size_of::<(i32, i32)>());
    
    // Option<ChunkCoord> adds discriminant + alignment
    println!("Option<ChunkCoord> size: {}", size_of::<Option<(i32, i32)>>());
    
    // ChunkState enum
    println!("ChunkState (enum) size: {}", size_of::<u8>());
    
    // ChunkTracker components
    println!("Array [Option<ChunkCoord>; 2] size: {}", size_of::<[Option<(i32, i32)>; 2]>());
    println!("Array [u8; 2] size: {}", size_of::<[u8; 2]>());
    println!("Option<ChunkCoord> size: {}", size_of::<Option<(i32, i32)>>());
    
    // Total estimated ChunkTracker size
    let chunk_tracker_size = size_of::<[Option<(i32, i32)>; 2]>() + 
                             size_of::<[u8; 2]>() + 
                             size_of::<Option<(i32, i32)>>() + 
                             size_of::<i16>() + 
                             size_of::<u16>() + 
                             size_of::<u8>() + 
                             size_of::<u8>();
    println!("Estimated ChunkTracker size: {}", chunk_tracker_size);
    
    // Vec3 size
    println!("Vec3 equivalent (3*f32) size: {}", size_of::<(f32, f32, f32)>());
    
    // WorldCoordinator components
    let world_coordinator_size = size_of::<(f32, f32, f32)>() + 
                                 size_of::<f32>() + 
                                 size_of::<u32>() + 
                                 size_of::<u32>() + 
                                 size_of::<[u32; 2]>();
    println!("Estimated WorldCoordinator size: {}", world_coordinator_size);
}
