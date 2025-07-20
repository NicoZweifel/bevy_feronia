#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::mesh_functions::get_model_matrix
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material
#import bevy_pbr::pbr_fragment::pbr_material_from_standard_material
#import bevy_pbr::pbr_functions::alpha_discard
#import bevy_pbr::pbr_functions::apply_pbr_lighting
#import bevy_pbr::pbr_functions::main_pass_post_lighting_processing
#import bevy_pbr::forward_io::Vertex
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::forward_io::FragmentOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_bindings::mesh

#import bevy_feronia::wind::{Wind, BindlessWindIndices}
#import bevy_feronia::displace::{displace_vertex_and_calc_normal, InstanceInfo, SampledNoise, DisplacedVertex}

#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#import bevy_pbr::pbr_bindings::{material_array, material_indices}
#else
#import bevy_pbr::pbr_bindings::material
#endif

#ifdef BINDLESS
@group(3) @binding(100) var<storage> wind_indices:
    array<BindlessWindIndices>;
@group(3) @binding(101) var<storage> wind_material:
    array<Wind>;

#else

@group(3) @binding(50) var<uniform> wind: Wind;
@group(3) @binding(51) var noise_texture: texture_2d<f32>;
@group(3) @binding(52) var noise_texture_sampler: sampler;

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

    // --- TEXTURE SAMPLING ---

    var noise: SampledNoise;
    noise.micro_noise = 0.0;
    noise.phase_noise = vec2<f32>(0.0);
    
    let macro_coord = instance.instance_position.xz * wind.noise_scale + instance.wrapped_time * wind.scroll_speed * wind.direction;
    noise.macro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, macro_coord, 0.0).r;
    
    #ifndef WIND_LOD
        let dist_to_camera = distance(instance.instance_position.xyz, view.world_position.xyz);
        let lod_fade = smoothstep(wind.lod_threshold * 2.0, wind.lod_threshold, dist_to_camera);
        let micro_coord = instance.instance_position.xz * wind.micro_noise_scale + instance.wrapped_time * wind.micro_scroll_speed;
        noise.micro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, micro_coord, 0.0).r;

        let texture_dimension = 512.0;
        let phase_coord_x = f32(instance.instance_index % u32(texture_dimension)) / texture_dimension;
        let phase_coord_y = f32(instance.instance_index / u32(texture_dimension)) / texture_dimension;
        let phase_coord = vec2<f32>(phase_coord_x, phase_coord_y);
        let phase_sample = textureSampleLevel(noise_texture, noise_texture_sampler, phase_coord, 0.0);
        noise.phase_noise = vec2(phase_sample.g, phase_sample.b);
    #endif

    // --- DISPLACEMENT ---
    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
        dist_to_camera,
        vertex.normal,
        vertex.uv
    );

    out.position = position_world_to_clip(displaced.world_position.xyz);
    out.world_position = displaced.world_position;
    out.world_normal = displaced.world_normal;

    out.uv = vertex.uv;
    out.instance_index = vertex.instance_index;

    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;

    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
