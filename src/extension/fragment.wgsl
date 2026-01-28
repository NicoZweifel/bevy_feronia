#import bevy_pbr::{
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decal_base_color,
    mesh_view_bindings::{view, lights, clusterable_objects},
    mesh_bindings::mesh,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_UNLIT_BIT},
    forward_io::{FragmentOutput},
    pbr_functions::{
        alpha_discard,
        apply_pbr_lighting,
        main_pass_post_lighting_processing,
        visibility_range_dither
    },
    shadows::{fetch_directional_shadow, fetch_point_shadow},
    mesh_view_types::POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    clustered_forward::{
        fragment_cluster_index,
        unpack_clusterable_object_index_ranges,
        get_clusterable_object_id
    }
}

#import bevy_feronia::forward_sss_io::VertexOutput

#ifdef BINDLESS
#import bevy_feronia::extension::bindings::material_uniforms_array
#else
#import bevy_feronia::extension::bindings::material_uniforms
#endif

#ifdef BINDLESS
#import bevy_feronia::wind::bindings::wind_affected_material_indices
#else
#import bevy_feronia::bindings::{noise_texture, noise_texture_sampler}
#endif


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var standard_in: bevy_pbr::forward_io::VertexOutput;

    standard_in.position = in.position;
    standard_in.world_position = in.world_position;
#ifdef VERTEX_NORMALS
    standard_in.world_normal = in.world_normal;

    if !is_front {
        standard_in.world_normal = -standard_in.world_normal;
    }

#endif

#ifdef VERTEX_UVS_A
    standard_in.uv = in.uv;
#endif

#ifdef VERTEX_TANGENTS
    standard_in.world_tangent = in.world_tangent;
#endif
    standard_in.instance_index = in.instance_index;

#ifdef VISIBILITY_RANGE_DITHER
    standard_in.visibility_range_dither = in.visibility_range_dither;
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

    var pbr_input = pbr_input_from_standard_material(standard_in, is_front);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;

#ifdef BINDLESS
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let material_uniforms =  material_uniforms_array[wind_affected_material_indices[slot].material];
#endif

    let wind = material_uniforms.current;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        #ifdef SUBSURFACE_SCATTERING
            let sss_glow = calc_sss_lighting(material_uniforms.sss_scale, material_uniforms.sss_intensity, pbr_input, in.thinness_factor);

            #ifdef DEBUG_SSS
                out.color = vec4<f32>(sss_glow, 1.0);
                return out;
            #else
                pbr_input.material.emissive = vec4<f32>(
                    pbr_input.material.emissive.rgb + (sss_glow * pbr_input.material.base_color.rgb),
                    pbr_input.material.emissive.a
                );
            #endif
        #endif

        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }

#ifdef MATERIAL_DEBUG
    out.color = pbr_input.material.base_color;
#else
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

   return out;
}

// TODO make reusable/expose/tweakable, see instanced.wgsl
#ifdef SUBSURFACE_SCATTERING
// GDC 2011 "Approximating Translucency" Implementation
// Source: Barré-Brisebois, C., & Bouchard, M. (2011). Approximating Translucency for a
// Fast, Cheap and Convincing Subsurface Scattering Look. Game Developers Conference.


// Controls how much the surface normal influences the scattered light direction.
// A value of 0.0 makes light appear to pass straight through.
// A value of 1.0 makes light appear to be heavily scattered by the surface.
const SSS_DISTORTION: f32 = 0.5;

const SSS_WRAP: f32 = 0.2;
const SSS_WRAP_INV: f32 = 1.0 / (1.0 + SSS_WRAP);

// Scale down light rgb TODO
const LIGHT_INTENSITY_SCALE = 0.00005;

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
        let scaled_light_color = light.color_inverse_square_range.rgb * LIGHT_INTENSITY_SCALE;
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

        // GDC 2011 SSS model
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
        let light_contribution = sun.color.rgb * LIGHT_INTENSITY_SCALE;

        sss_light += sss_factor * light_contribution * shadow;
    }

    return sss_light * intensity * thinness_factor;
}
#endif SUBSURFACE_SCATTERING
