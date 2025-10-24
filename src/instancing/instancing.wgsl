#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::utils::rand_f
#import bevy_pbr::mesh_bindings::mesh

#import bevy_feronia::wind::{Wind, BindlessWindIndices}
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::bindings::wind
#import bevy_feronia::displace::displace_vertex_and_calc_normal
#import bevy_feronia::noise::sample_noise


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(8) i_pos_scale: vec4<f32>,
    @location(9) i_index: u32,
};

struct InstanceUniforms {
    color: vec4<f32>,
    visibility_range: vec4<f32>,
};

@group(4) @binding(0)
var<uniform> instance_uniforms: InstanceUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,

#ifdef VISIBILITY_RANGE_DITHER
    @location(1) @interpolate(flat) visibility_range_dither: i32,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // --- INSTANCE ---
    var instance: InstanceInfo;

    instance.world_from_local = mat4x4<f32>(
        vec4<f32>(vertex.i_pos_scale.w, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, vertex.i_pos_scale.w, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, vertex.i_pos_scale.w, 0.0),
        vec4<f32>(vertex.i_pos_scale.xyz, 1.0)
    );

    instance.instance_position = instance.world_from_local[3];
    instance.wrapped_time = globals.time % 1000.0;
    instance.instance_index = vertex.i_index;

    let noise = sample_noise(instance, vertex.position);

    // --- DISPLACEMENT ---
    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
        vertex.normal,
        vertex.uv
    );

    out.clip_position = view.clip_from_world * displaced.world_position;

    let min_height = 0.0;
    let max_height = 1.0;

    let gradient_factor = saturate((vertex.position.y - min_height) / (max_height - min_height));

    let dark_color = vec4<f32>(instance_uniforms.color.rgb * 0.01, instance_uniforms.color.a);

    out.color = mix(dark_color, instance_uniforms.color, gradient_factor);

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = get_visibility_range_dither_level(
        instance_uniforms.visibility_range, instance.world_from_local[3]);
#endif

    return out;
}


#ifdef VISIBILITY_RANGE_DITHER
// taken/adapted from https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh_functions.wgsl
fn get_visibility_range_dither_level(lod_range: vec4<f32>, world_position: vec4<f32>) -> i32 {
    let camera_distance = length(view.world_position.xyz - world_position.xyz);


    // This encodes the following mapping:
    //
    //     `lod_range.`          x        y        z        w           camera distance
    //                   ←───────┼────────┼────────┼────────┼────────→
    //        LOD level  -16    -16       0        0        16      16  LOD level
    let offset = select(-16, 0, camera_distance >= lod_range.z);
    let bounds = select(lod_range.xy, lod_range.zw, camera_distance >= lod_range.z);
    let level = i32(round((camera_distance - bounds.x) / (bounds.y - bounds.x) * 16.0));
    return offset + clamp(level, 0, 16);
}
#endif

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

#ifdef VISIBILITY_RANGE_DITHER
    bevy_pbr::pbr_functions::visibility_range_dither(in.clip_position, in.visibility_range_dither);
#endif

    return in.color;
}






