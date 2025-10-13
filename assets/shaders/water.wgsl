#import bevy_pbr::{
    mesh_functions,
    mesh_bindings::mesh,
    forward_io::VertexOutput,
    view_transformations::position_world_to_clip,
}

struct WaveMaterial {
    base_color: vec4<f32>,
    amplitude: f32,
    frequency: f32,
    speed: f32,
    time: f32,
}

@group(2) @binding(0) var<uniform> material: WaveMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply wave displacement to vertex position
    var displaced = vertex.position;
    
    // Simple sine wave displacement using position and time
    let phase = material.frequency * (vertex.position.x + vertex.position.z) 
                + material.speed * material.time;
    displaced.y += material.amplitude * sin(phase);
    
    // Add second wave for more complex motion
    let phase2 = material.frequency * 0.7 * (vertex.position.x - vertex.position.z * 0.5) 
                 + material.speed * 1.3 * material.time;
    displaced.y += material.amplitude * 0.5 * sin(phase2);
    
    // Get transformation matrix for this instance
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    
    // Transform to world space
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(displaced, 1.0)
    );
    
    // Transform to clip space
    out.position = position_world_to_clip(world_position.xyz);
    out.world_position = world_position;
    
    // Pass through UVs
    out.uv = vertex.uv;
    
    // Transform normal (approximate - good enough for simple waves)
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return material.base_color;
}
