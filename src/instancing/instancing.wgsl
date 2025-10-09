#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::utils::rand_f

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
    @location(9) i_color: vec4<f32>,
    @location(10) i_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
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

    let dark_color = vec4<f32>(vertex.i_color.rgb * 0.01, vertex.i_color.a);

    out.color = mix(dark_color, vertex.i_color, gradient_factor);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}



