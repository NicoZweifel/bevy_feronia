//
#import bevy_pbr::mesh_view_bindings::{view, lights, globals, clusterable_objects}
#import bevy_pbr::shadows::fetch_directional_shadow
#import bevy_pbr::shadows::fetch_point_shadow
#import bevy_pbr::mesh_view_types::POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT
#import bevy_pbr::clustered_forward::{fragment_cluster_index, unpack_clusterable_object_index_ranges, get_clusterable_object_id}
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions
#import bevy_pbr::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT
#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::utils::rand_f
#import bevy_pbr::pbr_types::PbrInput,

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

    @location(1) @interpolate(linear, centroid) world_position: vec4<f32>,
    @location(2) @interpolate(linear, centroid) world_normal: vec3<f32>,
    @location(3) @interpolate(linear, centroid) uv: vec2<f32>,
    @location(4) @interpolate(linear, centroid) world_tangent: vec4<f32>,
    @location(5) @interpolate(linear, centroid) local_pos: vec3<f32>,

    @location(6) @interpolate(linear, centroid) curve_factor: f32,
#ifdef AMBIENT_OCCLUSION
    @location(7) @interpolate(linear, centroid) ao: f32,
#endif

    @location(8) i_batch_id: u32,

#ifdef SUBSURFACE_SCATTERING
    @location(9) thinness_factor: f32,
#endif
};


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var scale = vertex.i_pos_scale.w;
    var translation = vertex.i_pos_scale.xyz;
    var world_from_local_matrix: mat4x4<f32>;

    let batch = instance_uniforms[vertex.i_batch_id];

#ifdef BILLBOARDING
    world_from_local_matrix = mat4x4<f32>(
        vec4<f32>(scale, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, scale, 0.0),
        vec4<f32>(translation, 1.0)
    );
   var final_matrix = batch.world_from_local * world_from_local_matrix;
#else
   let final_matrix = utils::calc_instance_world_matrix(
        vertex.i_pos_scale,
        vertex.i_rotation,
        batch.world_from_local
    );
#endif

    var instance: InstanceInfo;
    instance.world_from_local = final_matrix;
    instance.instance_position = final_matrix[3];
    instance.wrapped_time = globals.time;
    instance.instance_index = vertex.i_index;
    instance.edge_correction_factor = material_uniforms.edge_correction_factor;
    instance.seed = vertex.i_seed;

// TODO
#ifdef STATIC_BEND
    let static_bend = material_uniforms.static_bend_direction
                    * material_uniforms.static_bend_strength;
#endif

    let wind = material_uniforms.current;
    let noise = sample_noise(instance, wind, vertex.position);

    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
#ifdef STATIC_BEND
        static_bend,
        material_uniforms.static_bend_control_point,
        material_uniforms.static_bend_min_max,
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

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = utils::get_visibility_range_dither_level(
        batch.visibility_range,
        final_matrix[3]
    );
#endif

#ifdef SUBSURFACE_SCATTERING
    let local_pos = vertex.position;

    let height = wind.aabb_max.y - wind.aabb_min.y;

    let inverse_height = 1.0 / max(height, 0.00001);

    // TODO support thickness texture instead of `thinness_factor`
    out.thinness_factor = saturate((local_pos.y - wind.aabb_min.y) * inverse_height);
 #endif

#ifdef CURVE_NORMALS
    out.curve_factor = material_uniforms.curve_factor;
#endif

    out.i_batch_id = vertex.i_batch_id;

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

    let batch = instance_uniforms[in.i_batch_id];

#ifdef MATERIAL_DEBUG
    return batch.color;
#endif

    let h = saturate(1.0 - in.uv.y);
    var albedo = batch.color.rgb;
    let bottom_height = material_uniforms.gradient_start;
    let bottom_strength = saturate(1.0 - (h / max(bottom_height, 0.0001)));
    albedo = mix(
        albedo,
        material_uniforms.bottom_color.rgb,
        bottom_strength * material_uniforms.tint_factor
    );

    let top_start_height = material_uniforms.gradient_end;
    let top_range = max(1.0 - top_start_height, 0.0001);
    let top_strength = saturate((h - top_start_height) / top_range);
    albedo = mix(
        albedo,
        material_uniforms.top_color.rgb,
        top_strength * material_uniforms.tint_factor
    );

#ifdef AMBIENT_OCCLUSION
    let ambient_occlusion = mix(0.01, 1.0, in.ao);
    albedo = albedo * ambient_occlusion;
#endif

#ifdef STANDARD_PBR
    var pbr_input: pbr_types::PbrInput = pbr_types::pbr_input_new();

    pbr_input.material.base_color = vec4<f32>(albedo, 1.0);
    pbr_input.material.perceptual_roughness = material_uniforms.roughness;
    pbr_input.material.metallic = material_uniforms.metallic;
    pbr_input.material.reflectance = vec3<f32>(material_uniforms.reflectance);
    pbr_input.frag_coord = in.clip_position;
    pbr_input.world_position = in.world_position;
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.flags = MESH_FLAGS_SHADOW_RECEIVER_BIT;

    var N = normalize(in.world_normal);
    var T = in.world_tangent;

    if !is_front {
        N = -N;
        T = -T;
    }


#ifdef CURVE_NORMALS
    let signed_norm_x = in.uv.x * 2.0 - 1.0;
    let curve_angle = in.curve_factor /2. * abs(signed_norm_x);
    let clamped_angle = clamp(curve_angle, 0.0, 1.4);
    let offset_mag = sin(clamped_angle);

    let curve_offset_world = in.world_tangent.xyz * offset_mag * sign(signed_norm_x);

    N = normalize(N + curve_offset_world);
#endif

    pbr_input.world_normal = N;
    pbr_input.N = N;
    pbr_input.V = pbr_functions::calculate_view(
        in.world_position,
        pbr_input.is_orthographic
    );

 #ifdef SUBSURFACE_SCATTERING
    let sss_glow = calc_sss_lighting(
        material_uniforms.subsurface_scattering_scale,
        material_uniforms.subsurface_scattering_intensity,
        pbr_input,
        in.thinness_factor
    );

    #ifdef DEBUG_SSS
        return vec4<f32>(sss_glow, 1.0);
    #else
        pbr_input.material.emissive = vec4<f32>(
            pbr_input.material.emissive.rgb + (sss_glow * pbr_input.material.base_color.rgb),
            pbr_input.material.emissive.a
        );
    #endif
#endif

    var output_color = pbr_functions::apply_pbr_lighting(pbr_input);
    output_color = pbr_functions::main_pass_post_lighting_processing(
        pbr_input,
        output_color
    );
    return output_color;

#else // NOT STANDARD_PBR
    var pbr_color = vec4<f32>(albedo, 1.0);
    var N = normalize(in.world_normal);
    var T = in.world_tangent;

    if !is_front {
        N = -N;
        T = -T;
    }

    #ifndef DIRECTIONAL_LIGHTS
    #ifndef POINT_LIGHTS
        return pbr_color;
    #endif
    #endif

    #ifdef CURVE_NORMALS
    #ifdef CURVE_NORMALS
        let signed_norm_x = in.uv.x * 2.0 - 1.0;
        let curve_angle = in.curve_factor * abs(signed_norm_x);
        let clamped_angle = clamp(curve_angle, 0.0, 1.4); // ~80 deg
        let offset_mag = sin(clamped_angle);

        let curve_offset_world = in.world_tangent.xyz * offset_mag * sign(signed_norm_x);

        N = normalize(N + curve_offset_world);
    #endif

    #endif
        // TODO move to module/file
        // Implements a simplified Blinn-Phong reflection model
        // for specular highlights, combined with a standard Lambertian diffuse term.
        //
        // 1. Blinn-Phong: Blinn, J. F. (1977). "Models of light reflection
        //    for computer synthesized pictures". SIGGRAPH '77.
        //
        // 2. Phong (used in bevy_procedural_grass):
        //    Phong, B. T. (1975). "Illumination for computer generated pictures".
        //    Communications of the ACM.

        var final_color_rgb = pbr_color.rgb * lights.ambient_color.rgb * material_uniforms.ambient_light_intensity;
        var final_specular = vec3<f32>(0.);

        let V = normalize(view.world_position.xyz - in.world_position.xyz);

        let view_z = dot(vec4<f32>(
            view.view_from_world[0].z,
            view.view_from_world[1].z,
            view.view_from_world[2].z,
            view.view_from_world[3].z
        ),  in.world_position);

    #ifdef DIRECTIONAL_LIGHTS
        for (var i = 0u; i < lights.n_directional_lights; i = i + 1u) {
            let sun = lights.directional_lights[i];
            let L = sun.direction_to_light;

            let scaled_light_color = sun.color.rgb * material_uniforms.light_intensity;

            // Translucency
            let NdotL_raw = dot(N, L);
            let NdotL_front = saturate(NdotL_raw);
            let NdotL_back = saturate(-NdotL_raw) * material_uniforms.translucency;
            let NdotL = NdotL_front + NdotL_back;

            // Specular Term
            let H = normalize(V + L);
            let NdotH = saturate(dot(N, H));
            let specular_factor = pow(NdotH, material_uniforms.specular_power);

            let shadow = fetch_directional_shadow(
                i,
                in.world_position,
                N,
                view_z
            );
            let final_shadow = clamp(shadow, 0.1, 1.);

            // Accumulate Diffuse
            final_color_rgb += pbr_color.rgb * scaled_light_color * NdotL * final_shadow * material_uniforms.diffuse_scaling;

            //  Accumulate Specular
            if NdotL_raw > 0. {
                final_specular += scaled_light_color * specular_factor * material_uniforms.specular_strength * shadow;
            }
        }
    #endif // DIRECTIONAL_LIGHTS

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
            let scaled_light_color = light.color_inverse_square_range.rgb * material_uniforms.light_intensity;
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
            let NdotL_raw = dot(N, L);
            let NdotL_front = saturate(NdotL_raw);
            let NdotL_back = saturate(-NdotL_raw) * material_uniforms.translucency;
            let NdotL = NdotL_front + NdotL_back;

            // Specular Term
            let H = normalize(V + L);
            let NdotH = saturate(dot(N, H));
            let specular_factor = pow(NdotH, material_uniforms.specular_power);

            var shadow = 1.;
            if ((light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
                shadow = fetch_point_shadow(light_id, in.world_position, N);
            }
            let final_shadow = clamp(shadow, 0.1, 1.);

            // Accumulate Diffuse
            final_color_rgb += pbr_color.rgb * scaled_light_color * attenuation * NdotL * final_shadow * material_uniforms.diffuse_scaling;

            //  Accumulate Specular
            if NdotL_raw > 0. {
                final_specular += scaled_light_color * attenuation * specular_factor * material_uniforms.specular_strength * shadow;
            }
        }
    #endif // POINT_LIGHTS

        final_color_rgb += final_specular;

        var final_color = vec4<f32>(final_color_rgb, pbr_color.w);

        return final_color;
    #endif // NOT STANDARD_PBR
}

// TODO make reusable/expose/tweakable, see /extension/fragment.wgsl
#ifdef SUBSURFACE_SCATTERING
// GDC 2011 "Approximating Translucency" Implementation
// Source: Barré-Brisebois, C., & Bouchard, M. (2011). Approximating Translucency for a
// Fast, Cheap and Convincing Subsurface Scattering Look. Game Developers Conference.


// Controls how much the surface normal influences the scattered light direction.
// A value of 0.0 makes light appear to pass straight through.
// A value of 1.0 makes light appear to be heavily scattered by the surface.
const SSS_DISTORTION: f32 = 0.1;

const SSS_WRAP: f32 = 1.5;
const SSS_WRAP_INV: f32 = 1.0 / (1.0 + SSS_WRAP);


fn calc_sss_lighting(scale:f32, intensity:f32, pbr_input: PbrInput, thinness_factor: f32) -> vec3<f32> {
    var sss_light = vec3<f32>(0.0);

    let view_pos = view.view_from_world * pbr_input.world_position;
    let view_z = view_pos.z;
    let cluster_index = fragment_cluster_index(pbr_input.frag_coord.xy, view_z, pbr_input.is_orthographic);
    let ranges = unpack_clusterable_object_index_ranges(cluster_index);

    // Point lights
    for (var i = ranges.first_point_light_index_offset; i < ranges.first_spot_light_index_offset; i = i + 1u) {
        let light_id = get_clusterable_object_id(i);
        let light = clusterable_objects.data[light_id];

        // Skip if covered by shadow
        var shadow = 1.0;
        if (light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT)!= 0u {
            shadow = fetch_point_shadow(light_id, pbr_input.world_position, pbr_input.world_normal);
        }

        if (shadow <= 0.0) { continue; }

        // Skip invalid range lights
        let light_position = light.position_radius.xyz;
        let scaled_light_color = light.color_inverse_square_range.rgb * material_uniforms.light_intensity;
        let inverse_square_range = light.color_inverse_square_range.w;

        if (inverse_square_range <= 0.0) { continue; }

        // Skip out of range lights
        let range_sq = 1.0 / inverse_square_range;
        let light_vector = light_position - pbr_input.world_position.xyz;
        let distance_sq = dot(light_vector, light_vector);

        if (distance_sq > range_sq) { continue; }

        let L = normalize(light_vector);

        let att_factor = saturate(1.0 - distance_sq / range_sq);
        let attenuation = att_factor * att_factor;
        let light_contribution = scaled_light_color * attenuation;

        // --- GDC 2011 SSS MODEL ---
        let H = normalize(L + pbr_input.world_normal * SSS_DISTORTION);

        let back_scatter_dot = saturate(dot(pbr_input.V, -H));

        // pow(dot, 8) * SSS_SCALE
        let t = back_scatter_dot * back_scatter_dot;
        let t2 = t * t;
        let back_scatter = t2 * t2 * scale;

        let front_scatter = saturate((dot(pbr_input.world_normal, L) + SSS_WRAP) * SSS_WRAP_INV);

        let sss_factor = back_scatter + front_scatter;

        sss_light += sss_factor * light_contribution * shadow;
    }

    // Directional lights
    for (var i = 0u; i < lights.n_directional_lights; i = i + 1u) {
        // Skip if covered by shadow
        let shadow = fetch_directional_shadow(i, pbr_input.world_position, pbr_input.world_normal, view_z);
        if (shadow <= 0.0) { continue; }

        let sun = lights.directional_lights[i];
        let L = sun.direction_to_light;

        // GDC 2011 SSS model
        let H = normalize(L + pbr_input.world_normal * SSS_DISTORTION);

        let back_scatter_dot = saturate(dot(pbr_input.V, -H));

        // pow(dot, 8) * SSS_SCALE
        let t = back_scatter_dot * back_scatter_dot;
        let t2 = t * t;
        let back_scatter = t2 * t2 * scale;

        let front_scatter = saturate((dot(pbr_input.world_normal, L) + SSS_WRAP) * SSS_WRAP_INV);

        let sss_factor = back_scatter + front_scatter;
        let light_contribution = sun.color.rgb * material_uniforms.light_intensity;

        sss_light += sss_factor * light_contribution * shadow;
    }

    return sss_light * intensity * thinness_factor;
}
#endif // SUBSURFACE_SCATTERING
