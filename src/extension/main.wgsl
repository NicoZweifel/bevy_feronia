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
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}

#ifdef BINDLESS
#import bevy_feronia::bindings::{wind_indices, wind_material}
#else
#import bevy_feronia::bindings::wind
#endif

#import bevy_feronia::displace::{displace_vertex_and_calc_normal}
#import bevy_feronia::noise::sample_noise


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef BINDLESS
    let slot = mesh[vertex.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let wind =  wind_material[wind_indices[slot].material];
#endif

    // --- INSTANCE ---
    var instance: InstanceInfo;
    let camera_world_pos = view.world_position.xyz;
    instance.world_from_local = get_world_from_local(vertex.instance_index);
    instance.instance_position = instance.world_from_local[3];
    instance.wrapped_time = globals.time % 1000.0;
    instance.instance_index = vertex.instance_index;

    let noise = sample_noise(instance);

    // --- DISPLACEMENT ---
    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
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

#ifdef DEBUG
#ifdef BINDLESS
    // You'll need to get the `wind` uniform here, similar to the vertex shader
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let wind_uniform =  wind_material[wind_indices[slot].material];
    out.color = wind_uniform.debug_color;
#else
    // If not bindless, the `wind` uniform should be directly available
    out.color = wind.debug_color;
#endif
#endif

    return out;
}
