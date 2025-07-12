#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::{get_model_matrix, get_world_from_local}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::prepass_io::{Vertex, VertexOutput}
#import bevy_pbr::prepass_bindings::globals

#import "shaders/wind.wgsl"::{Wind, BindlessWindIndices}
#import "shaders/wind_displace.wgsl"::{DisplacedVertex, SampledNoise, InstanceInfo, displace_vertex_and_calc_normal}
#import bevy_render::globals::Globals
#import bevy_pbr::mesh_bindings::mesh

#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#import bevy_pbr::pbr_bindings::{material_array, material_indices}
#else
#import bevy_pbr::pbr_bindings::material
#endif

@group(0) @binding(1) var<uniform> globals: Globals;

#ifdef BINDLESS
@group(2) @binding(100) var<storage> wind_indices:
    array<BindlessWindIndices>;
@group(2) @binding(101) var<storage> wind_material:
    array<Wind>;

#else

@group(2) @binding(50) var<uniform> wind: Wind;
@group(2) @binding(51) var noise_texture: texture_2d<f32>;
@group(2) @binding(52) var noise_texture_sampler: sampler;

#endif
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef BINDLESS
    let slot = mesh[vertex.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let wind =  wind_material[wind_indices[slot].material];
    let noise_texture =   bindless_textures_2d[wind_indices[slot].noise_texture];
    let noise_texture_sampler =  bindless_samplers_filtering[wind_indices[slot].noise_texture_sampler];
#endif

    // --- INSTANCE ---
    var instance: InstanceInfo;
    let camera_world_pos = view.world_position.xyz;
    instance.world_from_local = get_world_from_local(vertex.instance_index);
    instance.instance_position = instance.world_from_local[3];
    instance.wrapped_time = globals.time % 1000.0;
    instance.instance_index = vertex.instance_index;

    let dist_to_camera = distance(instance.instance_position.xyz, view.world_position.xyz);
    let lod_fade = smoothstep(wind.lod_threshold * 2.0, wind.lod_threshold, dist_to_camera);

    // --- TEXTURE SAMPLING ---
    var noise: SampledNoise;
    noise.micro_noise = 0.0;
    noise.phase_noise = vec2<f32>(0.0);

    let macro_coord = instance.instance_position.xz * wind.noise_scale + instance.wrapped_time * wind.scroll_speed * wind.direction;
    noise.macro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, macro_coord, 0.0).r;

    if (lod_fade > 0.0) {
        let micro_coord = instance.instance_position.xz * wind.micro_noise_scale + instance.wrapped_time * wind.micro_scroll_speed;
        noise.micro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, micro_coord, 0.0).r;

        let texture_dimension = 512.0;
        let phase_coord_x = f32(instance.instance_index % u32(texture_dimension)) / texture_dimension;
        let phase_coord_y = f32(instance.instance_index / u32(texture_dimension)) / texture_dimension;
        let phase_coord = vec2<f32>(phase_coord_x, phase_coord_y);
        let phase_sample = textureSampleLevel(noise_texture, noise_texture_sampler, phase_coord, 0.0);
        noise.phase_noise = vec2(phase_sample.g, phase_sample.b);
    }

    // --- DISPLACEMENT ---
    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
        dist_to_camera,
    #ifdef VERTEX_NORMALS
        vertex.normal,
        vertex.uv
    #endif
    );

    out.position = position_world_to_clip(displaced.world_position.xyz);
    out.world_position = displaced.world_position;

    #ifdef VERTEX_NORMALS
        out.world_normal = displaced.world_normal;
    #endif

    return out;
}


