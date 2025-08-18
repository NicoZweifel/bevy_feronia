#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::mesh_functions::get_model_matrix
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::prepass_io::Vertex
#import bevy_pbr::prepass_io::VertexOutput
#import bevy_pbr::prepass_bindings::globals
#import bevy_render::globals::Globals
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

@group(0) @binding(1) var<uniform> globals: Globals;

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
#ifdef VERTEX_NORMALS
        vertex.normal,
        vertex.uv,
#endif
    );

    out.position = position_world_to_clip(displaced.world_position.xyz);
    out.world_position = displaced.world_position;

#ifdef VERTEX_NORMALS
    out.world_normal = displaced.world_normal;
#endif

    out.uv = vertex.uv;

    return out;
}


