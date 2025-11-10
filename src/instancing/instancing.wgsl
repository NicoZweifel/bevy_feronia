#import bevy_pbr::mesh_view_bindings::{view, lights, globals, clusterable_objects}
#import bevy_pbr::shadows::fetch_directional_shadow
#import bevy_pbr::shadows::fetch_point_shadow
#import bevy_pbr::mesh_view_types::POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT
#import bevy_pbr::clustered_forward::{fragment_cluster_index, unpack_clusterable_object_index_ranges, get_clusterable_object_id}

#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::utils::rand_f
#import bevy_pbr::mesh_bindings::mesh

#import bevy_feronia::wind::{Wind, BindlessWindIndices}
#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::bindings::wind
#import bevy_feronia::displace::displace_vertex_and_calc_normal
#import bevy_feronia::noise::sample_noise

struct InstanceUniforms {
    color: vec4<f32>,
    visibility_range: vec4<f32>,
    static_bend_strength: f32,
    curve_factor: f32,
};

@group(4) @binding(0)
var<uniform> instance_uniforms: InstanceUniforms;

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
    @location(9) i_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,

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

    // --- INSTANCE ---
    var instance: InstanceInfo;

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
    // TODO pre-calculate / expose

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

    instance.world_from_local = world_from_local_matrix;
    instance.instance_position = instance.world_from_local[3];
    instance.wrapped_time = globals.time % 1000.0;
    instance.instance_index = vertex.i_index;


// TODO change to use quadratic cubic bezier
// https://github.com/NicoZweifel/bevy_feronia/issues/38
#ifdef STATIC_BEND
    let static_bend_angle = rand_f(&rand_state) * 6.28318;
    let static_bend_strength = rand_f(&rand_state) * instance_uniforms.static_bend_strength;

    let static_bend = vec2<f32>(cos(static_bend_angle), sin(static_bend_angle)) * static_bend_strength;
#endif

    let noise = sample_noise(instance, vertex.position);

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
    out.clip_position = view.clip_from_world * vec4<f32>(out.world_position, 1.0);

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif

    let height_range = wind.aabb_max.y - wind.aabb_min.y;
    let normalized_height = saturate((vertex.position.y - wind.aabb_min.y) / max(height_range, 0.0001));

    // Fake ambient occlusion
    let dark_color = vec4<f32>(instance_uniforms.color.rgb * 0.1, instance_uniforms.color.a);
    out.color = mix(dark_color, instance_uniforms.color, normalized_height);

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

    var normal = in.world_normal;

    if !is_front {
        normal = -normal;
    }


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
    const SPECULAR_POWER: f32 = 32.;

    // TODO tweak && expose
    const SPECULAR_STRENGTH: f32 = 0.2;
    const DIFFUSE_SCALING: f32 = 1.0;

    // Scale down light rgb
    const LIGHT_INTENSITY_SCALE: f32 = 0.00005;
    const AMBIENT_INTENSITY_SCALE: f32 = 0.002;

    const TRANSLUCENCY: f32 = 0.2;

    let V = normalize(view.world_position.xyz - in.world_position);

    var final_color_rgb = in.color.rgb * lights.ambient_color.rgb * AMBIENT_INTENSITY_SCALE;
    var final_specular = vec3<f32>(0.);

    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ), vec4<f32>(in.world_position, 1.));

    // --- Directional Lights (Sun) ---
    for (var i = 0u; i < lights.n_directional_lights; i = i + 1u) {
        let sun = lights.directional_lights[i];
        let L = sun.direction_to_light;
        let scaled_light_color = sun.color.rgb * LIGHT_INTENSITY_SCALE;

        // Translucency
        let NdotL_raw = dot(normal, L);
        let NdotL_front = saturate(NdotL_raw);
        let NdotL_back = saturate(-NdotL_raw) * TRANSLUCENCY;
        let NdotL = NdotL_front + NdotL_back;

        // Specular Term
        let H = normalize(V + L);
        let NdotH = saturate(dot(normal, H));
        let specular_factor = pow(NdotH, SPECULAR_POWER);

        let shadow = fetch_directional_shadow(
            i,
            vec4<f32>(in.world_position, 1.),
            normal,
            view_z
        );
        let final_shadow = clamp(shadow, 0.1, 1.);

        // Accumulate Diffuse
        final_color_rgb += in.color.rgb * scaled_light_color * NdotL * final_shadow * DIFFUSE_SCALING;

        //  Accumulate Specular
        if NdotL_raw > 0. {
            final_specular += scaled_light_color * specular_factor * SPECULAR_STRENGTH * shadow;
        }
    }

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

        // Translucency
        let NdotL_raw = dot(normal, L);
        let NdotL_front = saturate(NdotL_raw);
        let NdotL_back = saturate(-NdotL_raw) * TRANSLUCENCY;
        let NdotL = NdotL_front + NdotL_back;

        // Specular Term
        let H = normalize(V + L);
        let NdotH = saturate(dot(normal, H));
        let specular_factor = pow(NdotH, SPECULAR_POWER);

        var shadow = 1.;
        if ((light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
            shadow = fetch_point_shadow(light_id, vec4<f32>(in.world_position, 1.), normal);
        }
        let final_shadow = clamp(shadow, 0.1, 1.);

        // Accumulate Diffuse
        final_color_rgb += in.color.rgb * scaled_light_color * NdotL * final_shadow * DIFFUSE_SCALING;

        //  Accumulate Specular
        if NdotL_raw > 0. {
            final_specular += scaled_light_color * specular_factor * SPECULAR_STRENGTH * shadow;
        }
    }
#endif // POINT_LIGHTS

    final_color_rgb += final_specular;

    var final_color = vec4<f32>(final_color_rgb, in.color.a);

#ifdef MATERIAL_DEBUG
    final_color = wind.debug_color;
#endif

    return final_color;
}