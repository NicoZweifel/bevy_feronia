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
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::displace::displace_vertex_and_calc_normal
#import bevy_feronia::noise::sample_noise
#import bevy_eidolon::render::bindings::instance_uniforms

struct InstancedMaterialUniforms {
    wind: Wind,
    top_color: vec4<f32>,
    bottom_color: vec4<f32>,
    static_bend_strength: f32,
    curve_factor: f32,
    translucency: f32,
    specular_strength: f32,
    specular_power: f32,
};

@group(3) @binding(50) var<uniform> material: InstancedMaterialUniforms;
@group(3) @binding(51) var noise_texture: texture_2d<f32>;
@group(3) @binding(52) var noise_texture_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,

#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
#endif

    @location(8) i_pos_scale: vec4<f32>,
    @location(9) i_rotation: f32,
    @location(10) i_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ao: f32,

#ifdef VISIBILITY_RANGE_DITHER
    @location(1) @interpolate(flat) visibility_range_dither: i32,
#endif

    @location(2) world_position: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
    @location(4) uv: vec2<f32>,
    @location(5) local_pos: vec3<f32>,
    @location(6) world_tangent: vec4<f32>,
    @location(7) curve_factor: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var scale = vertex.i_pos_scale.w;
    var translation = vertex.i_pos_scale.xyz;

    var world_from_local_matrix: mat4x4<f32>;

    var rand_state = vertex.i_index;
#ifdef BILLBOARDING
    world_from_local_matrix = mat4x4<f32>(
        vec4<f32>(scale, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, scale, 0.0),
        vec4<f32>(translation, 1.0)
    );
#else
    // TODO pre-calculate (use i_rotation)

    let angle = rand_f(&rand_state) * 6.2831853;

    let c = cos(angle);
    let s = sin(angle);

    let rot_y_matrix = mat3x3<f32>(
        vec3<f32>(c, 0.0, s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-s, 0.0, c)
    );

    let rot_scale_matrix = rot_y_matrix * scale;

    world_from_local_matrix = mat4x4<f32>(
        vec4<f32>(rot_scale_matrix[0], 0.0),
        vec4<f32>(rot_scale_matrix[1], 0.0),
        vec4<f32>(rot_scale_matrix[2], 0.0),
        vec4<f32>(translation, 1.0)
    );

#endif

   var instance_world_matrix = instance_uniforms.world_from_local * world_from_local_matrix;

    var instance: InstanceInfo;
    instance.world_from_local = instance_world_matrix;
    instance.instance_position = instance_world_matrix[3];
    instance.wrapped_time = globals.time;
    instance.instance_index = vertex.i_index;

#ifdef STATIC_BEND
    let raw_rand = rand_f(&rand_state);
    let biased_rand = material.static_bend_strength + (raw_rand * (1. - material.static_bend_strength));

    let static_bend_angle = rand_f(&rand_state) * 6.28318;

    let static_bend_strength = biased_rand * material.static_bend_strength;

    let static_bend = vec2<f32>(cos(static_bend_angle), sin(static_bend_angle)) * static_bend_strength;
#endif

    let wind = material.wind;
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

    out.world_position = displaced.world_position.xyz;
    out.world_normal = displaced.world_normal;
    out.local_pos = vertex.position;
    out.world_tangent = displaced.world_tangent;
    out.clip_position = position_world_to_clip(out.world_position);

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif

    // Calculate the AO here, but leave the gradient/texture for the fragment shader.
    let height_range = wind.aabb_max.y - wind.aabb_min.y;
    let normalized_height = saturate((vertex.position.y - wind.aabb_min.y) / max(height_range, 0.0001));

    // 1.0 = full brightness, 0.0 = dark root area (global)
    out.ao = normalized_height;

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = get_visibility_range_dither_level(
        instance_uniforms.visibility_range, instance.world_from_local[3]);
#endif

#ifdef CURVE_NORMALS
    out.curve_factor = instance_uniforms.curve_factor;
#endif

    return out;
}




#ifdef VISIBILITY_RANGE_DITHER
// taken/adapted from https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh_functions.wgsl
fn get_visibility_range_dither_level(lod_range: vec4<f32>, world_position: vec4<f32>) -> i32 {
    let camera_distance = length(view.world_position.xyz - world_position.xyz);

    // This encodes the following mapping:
    //
    //     `lod_range.`          x        y        z        w           camera distance
    //                   ←───────┼────────┼────────┼────────┼────────→
    //     Dither Level  -16    -16       0        0        16      16  Dither Level

    let offset = select(-16, 0, camera_distance >= lod_range.z);
    let bounds = select(lod_range.xy, lod_range.zw, camera_distance >= lod_range.z);
    let level = i32(round((camera_distance - bounds.x) / (bounds.y - bounds.x) * 16.));
    return offset + clamp(level, 0, 16);
}
#endif




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

    var top_color = material.top_color.rgb;
    var bottom_color = material.bottom_color.rgb;

    // Blender exports UVs with Y=0 at bottom.
    let corrected_uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    let gradient_mix = pow(corrected_uv.y, 0.8);
    let blade_color_rgb = mix(bottom_color, top_color, gradient_mix);

    // TODO: allow texture usage here
    var albedo = blade_color_rgb;

    // in.color.r holds the normalized world height
    let global_height_factor = in.ao;

    // fake ao
    let ambient_occlusion = mix(0.4, 1.0, global_height_factor);

    albedo = albedo * ambient_occlusion;

    var color_for_lighting = vec4<f32>(albedo, 1.0);

    var normal = in.world_normal;

    if !is_front {
        normal = -normal;
    }

#ifndef DIRECTIONAL_LIGHTS
#ifndef POINT_LIGHTS
    return vec4<f32>(blade_color_rgb, in.ao);
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

    var final_color_rgb = color_for_lighting.rgb * lights.ambient_color.rgb * AMBIENT_INTENSITY_SCALE;
    var final_specular = vec3<f32>(0.);
    
    let V = normalize(view.world_position.xyz - in.world_position);

    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ), vec4<f32>(in.world_position, 1.));

#ifdef DIRECTIONAL_LIGHTS

    // --- Directional Lights (Sun) ---
    for (var i = 0u; i < lights.n_directional_lights; i = i + 1u) {
        let sun = lights.directional_lights[i];
        let L = sun.direction_to_light;

        let scaled_light_color = sun.color.rgb * LIGHT_INTENSITY_SCALE;

        // Translucency
        let NdotL_raw = dot(normal, L);
        let NdotL_front = saturate(NdotL_raw);
        let NdotL_back = saturate(-NdotL_raw) * material.translucency;
        let NdotL = NdotL_front + NdotL_back;

        // Specular Term
        let H = normalize(V + L);
        let NdotH = saturate(dot(normal, H));
        let specular_factor = pow(NdotH, material.specular_power);

        let shadow = fetch_directional_shadow(
            i,
            vec4<f32>(in.world_position, 1.),
            normal,
            view_z
        );
        let final_shadow = clamp(shadow, 0.1, 1.);

        // Accumulate Diffuse
        final_color_rgb += color_for_lighting.rgb * scaled_light_color * NdotL * final_shadow * DIFFUSE_SCALING;

        //  Accumulate Specular
        if NdotL_raw > 0. {
            final_specular += scaled_light_color * specular_factor * material.specular_strength * shadow;
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
        let NdotL_back = saturate(-NdotL_raw) * material.translucency;
        let NdotL = NdotL_front + NdotL_back;

        // Specular Term
        let H = normalize(V + L);
        let NdotH = saturate(dot(normal, H));
        let specular_factor = pow(NdotH, material.specular_power);

        var shadow = 1.;
        if ((light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = fetch_point_shadow(light_id, vec4<f32>(in.world_position, 1.), normal);
        }
        let final_shadow = clamp(shadow, 0.1, 1.);

        // Accumulate Diffuse
        final_color_rgb += color_for_lighting.rgb * scaled_light_color * attenuation * NdotL * final_shadow * DIFFUSE_SCALING;

        //  Accumulate Specular
        if NdotL_raw > 0. {
            final_specular += scaled_light_color * attenuation * specular_factor * material.specular_strength * shadow;
        }
    }
#endif // POINT_LIGHTS

    final_color_rgb += final_specular;

    var final_color = vec4<f32>(final_color_rgb, in.ao);

#ifdef MATERIAL_DEBUG
    final_color = wind.debug_color;
#endif

    return final_color;
}
