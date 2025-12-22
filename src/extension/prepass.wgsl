#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::{
    get_world_from_local,
    get_visibility_range_dither_level,
    get_model_matrix,
    mesh_tangent_local_to_world,
    get_previous_world_from_local,
    mesh_normal_local_to_world
}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material
#import bevy_pbr::pbr_fragment::pbr_material_from_standard_material
#import bevy_pbr::pbr_functions::alpha_discard
#import bevy_pbr::pbr_functions::apply_pbr_lighting
#import bevy_pbr::pbr_functions::main_pass_post_lighting_processing
#import bevy_pbr::prepass_io::Vertex
#import bevy_pbr::prepass_io::VertexOutput
#import bevy_render::{
    globals::Globals,
}
#import bevy_pbr::mesh_bindings::mesh

#import bevy_feronia::wind::Wind
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}

#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#endif

#ifdef BINDLESS
#import bevy_feronia::bindings::{wind_indices, wind_material}
#else
#import bevy_feronia::bindings::{wind, noise_texture, noise_texture_sampler}
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

    // --- CURRENT FRAME ---
    {
        let world_from_local = get_world_from_local(vertex.instance_index);

        // --- INSTANCE ---
        var instance: InstanceInfo;
        let camera_world_pos = view.world_position.xyz;
        instance.world_from_local = world_from_local;
        instance.instance_position = instance.world_from_local[3];
        instance.wrapped_time = globals.time;
        instance.instance_index = vertex.instance_index;

        let noise = sample_noise(instance, wind, vertex.position);

        var static_shadows = false;

        #ifdef STATIC_SHADOW
        #ifndef MOTION_VECTOR_PREPASS
        static_shadows = true;
        #endif
        #endif

        // --- DISPLACEMENT ---
       var displaced: DisplacedVertex;
        if static_shadows {
            displaced.world_position = (instance.world_from_local * vec4<f32>(vertex.position, 1.0));

            #ifdef VERTEX_NORMALS
                displaced.world_normal = mesh_normal_local_to_world(vertex.normal, instance.instance_index);
            #else
                displaced.world_normal = vec3<f32>(0.0, 0.0, 1.0);
            #endif

            #ifdef VERTEX_TANGENTS
                displaced.world_tangent = mesh_tangent_local_to_world(instance.world_from_local, vertex.tangent, instance.instance_index);
            #else
                const local_fallback_tangent: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                let world_tangent_xyz = (instance.world_from_local * vec4<f32>(local_fallback_tangent.xyz, 0.0)).xyz;

                displaced.world_tangent = vec4<f32>(normalize(world_tangent_xyz), local_fallback_tangent.w);
            #endif
        } else {
            displaced = displace_vertex_and_calc_normal(
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
        }

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

#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
        out.unclipped_depth = view.projection[2][2] * out.world_position.z + view.projection[3][2];
#endif
}

#ifdef MOTION_VECTOR_PREPASS
    // --- PREVIOUS FRAME ---
    {
        // --- INSTANCE ---
        var instance_prev: InstanceInfo;


        instance_prev.world_from_local = get_previous_world_from_local(vertex.instance_index);
        instance_prev.instance_position = instance_prev.world_from_local[3];
        instance_prev.wrapped_time = (globals.time - globals.delta_time);
        instance_prev.instance_index = vertex.instance_index;

        // TODO prev wind https://github.com/NicoZweifel/bevy_feronia/issues/34
        let noise_prev = sample_noise(instance_prev, wind, vertex.position);

        /// --- DISPLACEMENT ---
        let displaced_prev = displace_vertex_and_calc_normal(
            wind,
            noise_prev,
            vertex.position,
            instance_prev,
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
    }
#endif // MOTION_VECTOR_PREPASS

    return out;
}


