#import bevy_pbr::{
    mesh_functions,
    forward_io::VertexOutput,
    view_transformations::position_world_to_clip,
}

const MAX_WAVES: u32 = 4u;
const PI: f32 = 3.14159265359;
const GRAVITY: f32 = 9.81;

struct WaveMaterial {
    base_color: vec4<f32>,
    shallow_color: vec3<f32>,
    deep_color: vec3<f32>,
    foam_color: vec3<f32>,
    roughness: f32,
    fresnel_bias: f32,
    fresnel_power: f32,
    time: f32,
    wave_count: u32,
    _pad: vec2<f32>,
    // Wave data arrays
    wave_data0: array<vec4<f32>, 4>,  // (dir.x, dir.y, amplitude, wavelength)
    wave_data1: array<vec4<f32>, 4>,  // (speed, steepness, _pad, _pad)
}

@group(2) @binding(0) var<uniform> material: WaveMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct GerstnerResult {
    displacement: vec3<f32>,
    tangent_x: vec3<f32>,
    tangent_z: vec3<f32>,
}

// Gerstner wave calculation
fn gerstner_wave(xz: vec2<f32>, t: f32, wave_index: u32) -> GerstnerResult {
    var result: GerstnerResult;
    
    let wd0 = material.wave_data0[wave_index];
    let wd1 = material.wave_data1[wave_index];
    
    // Wave parameters
    let dir = normalize(vec2<f32>(wd0.x, wd0.y));
    let amplitude = wd0.z;
    let wavelength = max(wd0.w, 1.0);
    let steepness = clamp(wd1.y, 0.0, 1.0);
    
    // Wave number k = 2π/λ
    let k = 2.0 * PI / wavelength;
    
    // Angular frequency ω = sqrt(gk) for deep water
    let w = select(wd1.x, sqrt(GRAVITY * k), wd1.x == 0.0);
    
    // Steepness factor Q (controls wave sharpness)
    let Q = steepness / (amplitude * k + 0.0001);
    
    // Phase
    let phase = k * dot(dir, xz) - w * t;
    let s = sin(phase);
    let c = cos(phase);
    
    // Position displacement
    result.displacement = vec3<f32>(
        Q * amplitude * dir.x * c,
        amplitude * s,
        Q * amplitude * dir.y * c
    );
    
    // Tangent vectors for normal calculation
    let kQA = k * Q * amplitude;
    let kA = k * amplitude;
    
    // ∂P/∂x
    result.tangent_x = vec3<f32>(
        1.0 - kQA * dir.x * dir.x * s,
        kA * dir.x * c,
        -kQA * dir.y * dir.x * s
    );
    
    // ∂P/∂z
    result.tangent_z = vec3<f32>(
        -kQA * dir.x * dir.y * s,
        kA * dir.y * c,
        1.0 - kQA * dir.y * dir.y * s
    );
    
    return result;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    var displaced_pos = vertex.position;
    var tangent_x = vec3<f32>(1.0, 0.0, 0.0);
    var tangent_z = vec3<f32>(0.0, 0.0, 1.0);
    
    // Accumulate Gerstner waves
    for (var i: u32 = 0u; i < material.wave_count; i = i + 1u) {
        let wave = gerstner_wave(vertex.position.xz, material.time, i);
        displaced_pos += wave.displacement;
        tangent_x += wave.tangent_x - vec3<f32>(1.0, 0.0, 0.0);
        tangent_z += wave.tangent_z - vec3<f32>(0.0, 0.0, 1.0);
    }
    
    // Calculate normal from tangents
    let normal = normalize(cross(tangent_z, tangent_x));
    
    // Transform to world space
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(displaced_pos, 1.0)
    );
    
    // Transform to clip space
    out.position = position_world_to_clip(world_position.xyz);
    out.world_position = world_position;
    out.uv = vertex.uv;
    
    // Transform normal to world space
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        normal,
        vertex.instance_index
    );
    
    return out;
}

// Schlick's Fresnel approximation
fn fresnel_schlick(cos_theta: f32, bias: f32, power: f32) -> f32 {
    return clamp(bias + (1.0 - bias) * pow(1.0 - cos_theta, power), 0.0, 1.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize normal
    let N = normalize(in.world_normal);
    
    // View direction (camera to fragment)
    let V = normalize(in.world_position.xyz);
    let ndotv = max(dot(N, -V), 0.0);
    
    // Fresnel effect - water is more reflective at grazing angles
    let fresnel = fresnel_schlick(ndotv, material.fresnel_bias, material.fresnel_power);
    
    // Base water color
    let water_color = material.base_color.rgb;
    
    // Simple sky/environment reflection color (will improve with IBL)
    let sky_color = vec3<f32>(0.35, 0.45, 0.65);
    
    // Mix water color with sky reflection based on Fresnel
    let final_color = mix(water_color, sky_color, fresnel);
    
    return vec4<f32>(final_color, material.base_color.a);
}
