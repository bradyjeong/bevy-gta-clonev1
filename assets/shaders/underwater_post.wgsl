#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

struct UnderwaterSettings {
    sea_level: f32,
    fog_density: f32,
    absorption: vec3<f32>,
    scatter_color: vec3<f32>,
    enabled: u32,
}

@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0) var src_color: texture_2d<f32>;
@group(1) @binding(1) var src_sampler: sampler;

@group(2) @binding(0) var depth_tex: texture_depth_2d;

@group(3) @binding(0) var<uniform> params: UnderwaterSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(src_color, src_sampler, in.uv);
    
    if (params.enabled == 0u) {
        return color;
    }
    
    let pixel = vec2<i32>(in.position.xy);
    let depth_sample = textureLoad(depth_tex, pixel, 0);
    
    // If no geometry (sky/far plane), apply fog if camera is underwater
    if (depth_sample >= 1.0) {
        let cam_world_y = view.world_from_view[3].y;
        let cam_thickness = max(0.0, params.sea_level - cam_world_y);
        if (cam_thickness > 0.0) {
            let T = exp(-(params.absorption * cam_thickness) * params.fog_density);
            let fogged = color.rgb * T + params.scatter_color * (vec3<f32>(1.0) - T);
            return vec4<f32>(fogged, color.a);
        }
        return color;
    }
    
    let ndc = vec3<f32>(in.uv * 2.0 - 1.0, depth_sample);
    let clip = vec4<f32>(ndc.x, -ndc.y, ndc.z, 1.0);
    let world_h = view.world_from_clip * clip;
    let world_pos = world_h.xyz / world_h.w;
    
    let thickness = max(0.0, params.sea_level - world_pos.y);
    
    let T = exp(-(params.absorption * thickness) * params.fog_density);
    let fogged = color.rgb * T + params.scatter_color * (vec3<f32>(1.0) - T);
    
    return vec4<f32>(fogged, color.a);
}
