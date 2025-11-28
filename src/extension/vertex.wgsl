#import bevy_pbr::mesh_view_bindings::{view, globals}
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::mesh_functions::{get_world_from_local, get_visibility_range_dither_level}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::forward_io::Vertex


#import bevy_feronia::forward_sss_io::VertexOutput
#import bevy_feronia::wind::Wind
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::displace::{displace_vertex_and_calc_normal}
#import bevy_feronia::noise::sample_noise

#ifdef BINDLESS
    #import bevy_feronia::bindings::{wind_indices, wind_material}
#else
    #import bevy_feronia::bindings::{wind, noise_texture, noise_texture_sampler}
#endif


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef BINDLESS
    let slot = mesh[vertex.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let wind =  wind_material[wind_indices[slot].material];
#endif

    let world_from_local = get_world_from_local(vertex.instance_index);

    // --- INSTANCE ---
    var instance: InstanceInfo;
    let camera_world_pos = view.world_position.xyz;
    instance.world_from_local = world_from_local;
    instance.instance_position = instance.world_from_local[3];
    instance.wrapped_time = globals.time;
    instance.instance_index = vertex.instance_index;

    let noise = sample_noise(instance, vertex.position);

    // --- DISPLACEMENT ---
    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
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

#ifdef VERTEX_POSITIONS
    out.position = position_world_to_clip(displaced.world_position.xyz);
    out.world_position = displaced.world_position;
#endif

#ifdef VERTEX_NORMALS
    out.world_normal = displaced.world_normal;
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = displaced.world_tangent;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = get_visibility_range_dither_level(
        vertex.instance_index, world_from_local[3]);
#endif

#ifdef SUBSURFACE_SCATTERING
    let local_pos = vertex.position;

    let height = wind.aabb_max.y - wind.aabb_min.y;

    let inverse_height = 1.0 / max(height, 0.00001);

    // TODO support thickness texture instead of `thinness_factor`
    out.thinness_factor = saturate((local_pos.y - wind.aabb_min.y) * inverse_height);
 #endif

    return out;
}

