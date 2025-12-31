#import bevy_pbr::prepass_bindings

#import bevy_pbr::mesh_view_bindings::view
#import bevy_render::view::View
#import bevy_render::{
    globals::Globals,
}

#import bevy_eidolon::render::bindings::instance_uniforms
#import bevy_eidolon::render::utils
#import bevy_eidolon::render::io_types::Vertex

#import bevy_pbr::utils::rand_f

#import bevy_feronia::wind::Wind
#import bevy_feronia::instancing::bindings::material_uniforms
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::displace::displace_vertex_and_calc_normal
#import bevy_feronia::noise::sample_noise

@group(0) @binding(1) var<uniform> globals: Globals;

struct PrepassVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) previous_world_position: vec4<f32>,

#ifdef NORMAL_PREPASS
    @location(2) world_normal: vec3<f32>,
    #ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
    #endif
#endif

#ifdef VISIBILITY_RANGE_DITHER
    @location(4) @interpolate(flat) visibility_range_dither: i32,
#endif
};

// TODO specialize prepass pipeline correctly

#define STATIC_BEND
#define EDGE_CORRECTION
#define WIND_AFFECTED
#define VISIBILITY_RANGE_DITHER
#define STATIC_BEND
#ifdef NORMAL_PREPASS
#define VERTEX_NORMALS
#endif
#define ANALYTICAL_NORMALS


@vertex
fn vertex(vertex: Vertex) -> PrepassVertexOutput {
    var out: PrepassVertexOutput;
    let wind = material_uniforms.wind;

    let final_matrix = utils::calc_instance_world_matrix(
        vertex.i_pos_scale,
        vertex.i_rotation,
        instance_uniforms.world_from_local
    );

// TODO
#ifdef STATIC_BEND
    const STATIC_BEND_DIR = vec2<f32>(0.309017, -0.951056);

    let static_bend = STATIC_BEND_DIR * material_uniforms.static_bend_strength;
#endif

    var instance: InstanceInfo;
    instance.world_from_local = final_matrix;
    instance.instance_position = final_matrix[3];
    instance.wrapped_time = globals.time;
    instance.instance_index = vertex.i_index;

    let noise = sample_noise(instance, wind, vertex.position);

    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
#ifdef STATIC_BEND
        static_bend,
#endif
#ifdef VERTEX_NORMALS
        vertex.normal,
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent,
#endif
#ifdef VERTEX_UVS_A
        vertex.uv
#endif
    );

    out.world_position = displaced.world_position;
    out.clip_position = view.clip_from_world * out.world_position;

#ifdef NORMAL_PREPASS
    out.world_normal = displaced.world_normal;
    #ifdef VERTEX_TANGENTS
        out.world_tangent = displaced.world_tangent;
    #endif
#endif

    out.previous_world_position = displaced.world_position;

#ifdef MOTION_VECTOR_PREPASS
    let prev_final_matrix = utils::calc_instance_world_matrix(
        vertex.i_pos_scale,
        vertex.i_rotation,
        instance_uniforms.previous_world_from_local
    );

    var instance_prev: InstanceInfo;
    instance_prev.world_from_local = prev_final_matrix;
    instance_prev.instance_position = prev_final_matrix[3];
    instance_prev.wrapped_time = globals.time - globals.delta_time;
    instance_prev.instance_index = vertex.i_index;

    let noise_prev = sample_noise(instance_prev, wind, vertex.position);

    let displaced_prev = displace_vertex_and_calc_normal(
        wind,
        noise_prev,
        vertex.position,
        instance_prev,
#ifdef STATIC_BEND
        static_bend,
#endif
#ifdef VERTEX_NORMALS
        vertex.normal,
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent,
#endif
#ifdef VERTEX_UVS_A
        vertex.uv
#endif
    );

    out.previous_world_position = displaced_prev.world_position;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
        out.instance_index = vertex.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = utils::get_visibility_range_dither_level(
        instance_uniforms.visibility_range,
        final_matrix[3]
    );
#endif

    return out;
}

#ifdef PREPASS_FRAGMENT

#import bevy_pbr::prepass_io::FragmentOutput

@fragment
fn fragment(in: PrepassVertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

#ifdef VISIBILITY_RANGE_DITHER
    bevy_pbr::pbr_functions::visibility_range_dither(
        in.clip_position,
        in.visibility_range_dither
    );
#endif

#ifdef NORMAL_PREPASS
    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
#endif

#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif // UNCLIPPED_DEPTH_ORTHO_EMULATION

#ifdef MOTION_VECTOR_PREPASS
    let clip_position_t = view.unjittered_clip_from_world * in.world_position;
    let clip_position = clip_position_t.xy / clip_position_t.w;
    let previous_clip_position_t = prepass_bindings::previous_view_uniforms.clip_from_world * in.previous_world_position;
    let previous_clip_position = previous_clip_position_t.xy / previous_clip_position_t.w;

    out.motion_vector = (clip_position - previous_clip_position) * vec2(0.5, -0.5);
#endif // MOTION_VECTOR_PREPASS

    return out;
}
#endif // PREPASS_FRAGMENT