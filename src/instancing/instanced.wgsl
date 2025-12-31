#import bevy_pbr::mesh_view_bindings::{view, lights, globals, clusterable_objects}
#import bevy_pbr::shadows::fetch_directional_shadow
#import bevy_pbr::shadows::fetch_point_shadow
#import bevy_pbr::mesh_view_types::POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT
#import bevy_pbr::clustered_forward::{fragment_cluster_index, unpack_clusterable_object_index_ranges, get_clusterable_object_id}

#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::utils::rand_f
#import bevy_pbr::mesh_bindings::mesh


#import bevy_feronia::wind::Wind
#import bevy_feronia::instancing::bindings::material_uniforms
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::displace::displace_vertex_and_calc_normal
#import bevy_feronia::noise::sample_noise

#import bevy_eidolon::render::utils
#import bevy_eidolon::render::bindings::instance_uniforms
#import bevy_eidolon::render::io_types::Vertex

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

#ifdef VISIBILITY_RANGE_DITHER
    @location(0) @interpolate(flat) visibility_range_dither: i32,
#endif

    @location(1) world_position: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) world_tangent: vec4<f32>,
    @location(5) local_pos: vec3<f32>,

    @location(6) curve_factor: f32,
#ifdef AMBIENT_OCCLUSION
    @location(7) ao: f32,
#endif
};


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var scale = vertex.i_pos_scale.w;
    var translation = vertex.i_pos_scale.xyz;

    var world_from_local_matrix: mat4x4<f32>;

#ifdef BILLBOARDING
    world_from_local_matrix = mat4x4<f32>(
        vec4<f32>(scale, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, scale, 0.0),
        vec4<f32>(translation, 1.0)
    );
   var final_matrix = instance_uniforms.world_from_local * world_from_local_matrix;
#else
   let final_matrix = utils::calc_instance_world_matrix(
        vertex.i_pos_scale,
        vertex.i_rotation,
        instance_uniforms.world_from_local
    );
#endif


    var instance: InstanceInfo;
    instance.world_from_local = final_matrix;
    instance.instance_position = final_matrix[3];
    instance.wrapped_time = globals.time;
    instance.instance_index = vertex.i_index;

// TODO
#ifdef STATIC_BEND
    const STATIC_BEND_DIR = vec2<f32>(0.309017, -0.951056);

    let static_bend = STATIC_BEND_DIR * material_uniforms.static_bend_strength * 0.0;
#endif

    let wind = material_uniforms.wind;
    let noise = sample_noise(instance, wind, vertex.position);

    // --- DISPLACEMENT ---
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
    out.world_normal = displaced.world_normal;
    out.local_pos = vertex.position;
    out.world_tangent = displaced.world_tangent;
    out.clip_position = position_world_to_clip(out.world_position.xyz);

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif

#ifdef AMBIENT_OCCLUSION
    let height_range = wind.aabb_max.y - wind.aabb_min.y;
    let normalized_height = saturate((vertex.position.y - wind.aabb_min.y) / max(height_range, 0.0001));

    out.ao = normalized_height;
#endif

    out.visibility_range_dither = utils::get_visibility_range_dither_level(
        instance_uniforms.visibility_range,
        final_matrix[3]
    );

#ifdef CURVE_NORMALS
    out.curve_factor = material_uniforms.curve_factor;
#endif

    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {

#ifdef VISIBILITY_RANGE_DITHER
    #ifndef SHADOW_PASS
        bevy_pbr::pbr_functions::visibility_range_dither(in.clip_position, in.visibility_range_dither);
    #endif
#endif

    // UV Requirements:
    // - uv.y = 0.0 corresponds to the tip (Top).
    // - uv.y = 1.0 corresponds to the root (Bottom).
    let height_factor = saturate(1.0 - in.uv.y);
    let gradient_mix = smoothstep(
        material_uniforms.gradient_start,
        material_uniforms.gradient_end,
        height_factor
    );
    
    let raw_gradient_color = mix(
        material_uniforms.bottom_color.rgb,
        material_uniforms.top_color.rgb,
        gradient_mix
    );

    let gradient_lum = dot(raw_gradient_color, vec3<f32>(0.299, 0.587, 0.114));
    let instance_shaded_color = instance_uniforms.color.rgb * gradient_lum;

    // TODO: allow texture usage here
    var albedo = mix(
        instance_shaded_color,
        raw_gradient_color,
        material_uniforms.tint_factor
    );;

#ifdef AMBIENT_OCCLUSION
    // fake ao TODO
    let ambient_occlusion = mix(0.4, 1.0, in.ao);
    albedo = albedo * ambient_occlusion;
#endif

    var pbr_color = vec4<f32>(albedo, 1.0);
    var normal = in.world_normal;
    if !is_front {
        normal = -normal;
    }

#ifndef DIRECTIONAL_LIGHTS
#ifndef POINT_LIGHTS
    return pbr_color;
#endif
#endif


// TODO move out / re-use
#ifdef CURVE_NORMALS

    let signed_norm_x = in.uv.x * 2.0 - 1.0;

    let curve_angle = in.curve_factor * abs(signed_norm_x);
    let clamped_angle = clamp(curve_angle, 0.0, 1.4); // ~80 deg
    let offset_mag = sin(clamped_angle);

    let curve_offset_world = in.world_tangent.xyz * offset_mag * sign(signed_norm_x);

    normal = normalize(normal + curve_offset_world);
#endif

    // --- LIGHTING MODEL ---
    // Implements a simplified Blinn-Phong reflection model
    // for specular highlights, combined with a standard Lambertian diffuse term.
    //
    // 1. Blinn-Phong: Blinn, J. F. (1977). "Models of light reflection
    //    for computer synthesized pictures". SIGGRAPH '77.
    //
    // 2. Phong (used in bevy_procedural_grass):
    //    Phong, B. T. (1975). "Illumination for computer generated pictures".
    //    Communications of the ACM.
    // ---

    // TODO expose/unify as much as possible with extended shader before exposing fields in `MaterialOptions`

    // TODO tweak && expose
    const DIFFUSE_SCALING: f32 = 1.0;

    // Scale down light rgb
    const LIGHT_INTENSITY_SCALE: f32 = 0.00005;
    const AMBIENT_INTENSITY_SCALE: f32 = 0.001;

    var final_color_rgb = pbr_color.rgb * lights.ambient_color.rgb * AMBIENT_INTENSITY_SCALE;
    var final_specular = vec3<f32>(0.);

    let V = normalize(view.world_position.xyz - in.world_position.xyz);

    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ),  in.world_position);

#ifdef DIRECTIONAL_LIGHTS

    // --- Directional Lights (Sun) ---
    for (var i = 0u; i < lights.n_directional_lights; i = i + 1u) {
        let sun = lights.directional_lights[i];
        let L = sun.direction_to_light;

        let scaled_light_color = sun.color.rgb * LIGHT_INTENSITY_SCALE;

        // Translucency
        let NdotL_raw = dot(normal, L);
        let NdotL_front = saturate(NdotL_raw);
        let NdotL_back = saturate(-NdotL_raw) * material_uniforms.translucency;
        let NdotL = NdotL_front + NdotL_back;

        // Specular Term
        let H = normalize(V + L);
        let NdotH = saturate(dot(normal, H));
        let specular_factor = pow(NdotH, material_uniforms.specular_power);

        let shadow = fetch_directional_shadow(
            i,
            in.world_position,
            normal,
            view_z
        );
        let final_shadow = clamp(shadow, 0.1, 1.);

        // Accumulate Diffuse
        final_color_rgb += pbr_color.rgb * scaled_light_color * NdotL * final_shadow * DIFFUSE_SCALING;

        //  Accumulate Specular
        if NdotL_raw > 0. {
            final_specular += scaled_light_color * specular_factor * material_uniforms.specular_strength * shadow;
        }
    }
#endif

#ifdef POINT_LIGHTS
    let is_orthographic = view.clip_from_view[3].w == 1.;
    let cluster_index = fragment_cluster_index(
        in.clip_position.xy,
        view_z,
        is_orthographic
    );
    let ranges = unpack_clusterable_object_index_ranges(cluster_index);

    for (var i = ranges.first_point_light_index_offset; i < ranges.first_spot_light_index_offset; i = i + 1u) {
        let light_id = get_clusterable_object_id(i);
        let light = clusterable_objects.data[light_id];

        let light_position = light.position_radius.xyz;
        let scaled_light_color = light.color_inverse_square_range.rgb * LIGHT_INTENSITY_SCALE;
        let inverse_square_range = light.color_inverse_square_range.w;

        if (inverse_square_range <= 0.) { continue; }

        // Skip out of range lights
        let range_sq = 1. / inverse_square_range;
        let light_vector = light_position - in.world_position.xyz;
        let distance_sq = dot(light_vector, light_vector);

        if (distance_sq > range_sq) { continue; }

        let L = normalize(light_vector);

        let range_factor = distance_sq * inverse_square_range;
        let smooth_falloff = saturate(1.0 - range_factor);
        let attenuation = smooth_falloff * smooth_falloff;

        // Translucency
        let NdotL_raw = dot(normal, L);
        let NdotL_front = saturate(NdotL_raw);
        let NdotL_back = saturate(-NdotL_raw) * material_uniforms.translucency;
        let NdotL = NdotL_front + NdotL_back;

        // Specular Term
        let H = normalize(V + L);
        let NdotH = saturate(dot(normal, H));
        let specular_factor = pow(NdotH, material_uniforms.specular_power);

        var shadow = 1.;
        if ((light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = fetch_point_shadow(light_id, in.world_position, normal);
        }
        let final_shadow = clamp(shadow, 0.1, 1.);

        // Accumulate Diffuse
        final_color_rgb += pbr_color.rgb * scaled_light_color * attenuation * NdotL * final_shadow * DIFFUSE_SCALING;

        //  Accumulate Specular
        if NdotL_raw > 0. {
            final_specular += scaled_light_color * attenuation * specular_factor * material_uniforms.specular_strength * shadow;
        }
    }
#endif // POINT_LIGHTS

    final_color_rgb += final_specular;

    var final_color = vec4<f32>(final_color_rgb, pbr_color.w);

#ifdef MATERIAL_DEBUG
    final_color = wind.debug_color;
#endif

    return final_color;
}
