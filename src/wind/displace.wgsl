// The use of layered macro/micro noise, secondary motions (S-curve, bop)
// and the methods for normal curving and edge correction are all heavily inspired by:

// "Ghost of Tsushima" and Eric Wohllaib
// of Sucker Punch Productions and the GDC 2021 talk:
// "Advanced Graphics Summit: Procedural Grass in 'Ghost of Tsushima'".

#define_import_path bevy_feronia::displace

#import bevy_pbr::mesh_functions::{mesh_normal_local_to_world, mesh_tangent_local_to_world}
#import bevy_pbr::mesh_view_bindings::view
#import bevy_render::view::{position_world_to_view, position_view_to_world}

#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::wind::Wind
#import bevy_feronia::noise::sample_noise

struct CurveResult {
    local_pos: vec3<f32>,
    tangent: vec3<f32>,
    twist: f32,
    height_factor: f32,
    local_wind_dir: vec3<f32>
}

fn displace_vertex_and_calc_normal(
    wind: Wind,
    noise: SampledNoise,
    vertex_pos: vec3<f32>,
    instance: InstanceInfo,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
#ifdef VERTEX_NORMALS
    normal: vec3<f32>,
#endif
#ifdef VERTEX_TANGENTS
    tangent: vec4<f32>,
#endif
#ifdef VERTEX_UVS_A
    uv: vec2<f32>
#endif
) -> DisplacedVertex {
    var result: DisplacedVertex;

    // Macro (wind bend, static bend)
    // Macro, i.e., wind bend, static bend
    let curve_data = calc_macro_curve(
        vertex_pos,
        wind,
        noise,
        instance,
    #ifdef STATIC_BEND
        static_bend
    #endif
    );

    // Micro, s-curve, bop
    // Micro, i.e., s-curve, bop
    let final_local_pos = apply_micro_details(
        curve_data.local_pos,
        curve_data,
        wind,
        instance,
        noise
    );

    result.world_position = instance.world_from_local * vec4<f32>(final_local_pos, 1.0);

    #ifdef BILLBOARDING
    result.world_position = vec4<f32>(
        billboarding(
            wind,
            instance,
            final_local_pos,
            final_local_pos - vertex_pos
        ),
        1.0
    );
    #endif

#ifdef FAST_NORMALS
    // Uses the original, un-displaced world-space normals.
    //
    // Will have incorrect lighting as the normals will not match the displaced vertex positions.
    //
    // The mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
    // and its "face" pointing along Z-Up (`+Z`).
    //
    // Should be used for performance reasons and/or on static or barely wind affected objects.

    #ifdef VERTEX_NORMALS
        result.world_normal = mesh_normal_local_to_world(normal, instance.instance_index);
    #else
        result.world_normal = normalize(instance.world_from_local[2].xyz);
    #endif

    #ifdef VERTEX_TANGENTS
        result.world_tangent = mesh_tangent_local_to_world(instance.world_from_local, tangent, instance.instance_index);
    #else
        let world_tangent_xyz = normalize(instance.world_from_local[0].xyz);
        result.world_tangent = vec4<f32>(world_tangent_xyz, 1.0);
    #endif

#else
#ifdef ANALYTICAL_NORMALS
    // Calculates normals using a mathematical approximation of the
    // displacement.
    //
    // Should be faster than numerical sampling but less
    // accurate, as it only accounts for static_bend, twist,
    // and macro_wind, ignoring high-frequency displacements.
    //
    // The mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
    // and its "face" pointing along Z-Up (`+Z`).
    //
    // Typically used for billboarded foliage or flat meshes like grass.

    let local_spine_direction = curve_data.tangent;

    let cos_twist = cos(curve_data.twist);
    let sin_twist = sin(curve_data.twist);

    let local_width_direction = vec3<f32>(cos_twist, 0.0, sin_twist);
    let model_rotation_matrix = mat3x3<f32>(
        instance.world_from_local[0].xyz,
        instance.world_from_local[1].xyz,
        instance.world_from_local[2].xyz
    );

    let world_spine = normalize(model_rotation_matrix * local_spine_direction);
    let world_width = normalize(model_rotation_matrix * local_width_direction);

    let raw_normal = cross(world_width, world_spine);
    let len_sq = dot(raw_normal, raw_normal);
    let safe_normal = select(vec3<f32>(0.0, 1.0, 0.0), raw_normal, len_sq > 1.0e-6);

    result.world_normal = normalize(safe_normal);
    result.world_tangent = vec4<f32>(world_width, 1.0);

#else
    // Calculates normals numerically by sampling neighboring positions.
    //
    // Should be the most accurate, but most expensive path, as it runs the full
    // displacement logic on the neighbors to find the surface direction.
    //
    // Typically used for complex foliage like non-billboarded bushes, trees.

    #ifdef VERTEX_NORMALS
        let local_normal = normal;
    #else
        let local_normal = vec3<f32>(0.0, 0.0, 1.0);
    #endif

    #ifdef VERTEX_TANGENTS
        let local_tangent = tangent.xyz;
        let tangent_sign = tangent.w;
    #else
        let local_tangent = vec3<f32>(1.0, 0.0, 0.0);
        let tangent_sign = 1.0;
    #endif

    let local_bitangent = normalize(cross(local_normal, local_tangent) * tangent_sign);

    // Too small, e.g. 0.01, causes flicker on simple geometry
    let sample_offset = 0.05;
    let base_displaced_pos = final_local_pos;

    // Sample Neighbor along Tangent
    let neighbor_tangent_origin = vertex_pos + local_tangent * sample_offset;
    let noise_tangent = sample_noise(instance, wind, neighbor_tangent_origin);
    let curve_tangent = calc_macro_curve(neighbor_tangent_origin, wind, noise_tangent, instance,
        #ifdef STATIC_BEND
        static_bend
        #endif
    );
    let neighbor_tangent_displaced = apply_micro_details(curve_tangent.local_pos, curve_tangent, wind, instance, noise_tangent);

    // Sample Neighbor along Bitangent
    let neighbor_bitangent_origin = vertex_pos + local_bitangent * sample_offset;
    let noise_bitangent = sample_noise(instance, wind, neighbor_bitangent_origin);
    let curve_bitangent = calc_macro_curve(neighbor_bitangent_origin, wind, noise_bitangent, instance,
        #ifdef STATIC_BEND
        static_bend
        #endif
    );
    let neighbor_bitangent_displaced = apply_micro_details(curve_bitangent.local_pos, curve_bitangent, wind, instance, noise_bitangent);

    // Deltas
    let delta_tangent = neighbor_tangent_displaced - base_displaced_pos;
    let delta_bitangent = neighbor_bitangent_displaced - base_displaced_pos;

    let computed_local_normal = normalize(cross(delta_tangent, delta_bitangent) * tangent_sign);

    // Prevent back-facing normals if displacement is extreme
    let dot_alignment = dot(computed_local_normal, local_normal);
    let safe_local_normal = select(computed_local_normal, local_normal, dot_alignment < 0.1);

    let model_rotation_matrix = mat3x3<f32>(
        instance.world_from_local[0].xyz,
        instance.world_from_local[1].xyz,
        instance.world_from_local[2].xyz
    );

    result.world_normal = normalize(model_rotation_matrix * safe_local_normal);

    let world_tangent_vec = normalize(model_rotation_matrix * delta_tangent);

    result.world_tangent = vec4<f32>(
        normalize(world_tangent_vec - dot(world_tangent_vec, result.world_normal) * result.world_normal),
        tangent_sign
    );
#endif
#endif

#ifdef EDGE_CORRECTION
#ifdef VERTEX_UVS_A
    let edge_correction_offset = calc_edge_correction(
        result.world_position.xyz,
        result.world_normal,
        uv.x,
        instance.edge_correction_factor
    );

    result.world_position += vec4<f32>(edge_correction_offset, 0.);
#endif
#endif

    return result;
}

fn calc_macro_curve(
    local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
    instance: InstanceInfo,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
) -> CurveResult {
    var result: CurveResult;

    let height_range = max(wind.aabb_max.y - wind.aabb_min.y, 0.001);
    let height_progress = clamp((local_pos.y - wind.aabb_min.y) / height_range, 0.0, 1.0);
    result.height_factor = height_progress;
    result.twist = 0.0;

    #ifdef WIND_AFFECTED
        // Get scale from the instance matrix to apply wind correctly in local space
        let scale_x = length(instance.world_from_local[0].xyz);
        let scale_y = length(instance.world_from_local[1].xyz);
        let scale_z = length(instance.world_from_local[2].xyz);

        let rotation_matrix = mat3x3<f32>(
            instance.world_from_local[0].xyz / max(scale_x, 0.001),
            instance.world_from_local[1].xyz / max(scale_y, 0.001),
            instance.world_from_local[2].xyz / max(scale_z, 0.001)
        );

        let world_wind_direction = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);
        let local_wind_unscaled = transpose(rotation_matrix) * world_wind_direction;

        let safe_wind_vec = local_wind_unscaled + vec3<f32>(1.0e-5, 0.0, 0.0);
        result.local_wind_dir = normalize(safe_wind_vec);
    #else
        result.local_wind_dir = vec3<f32>(1.0, 0.0, 0.0);
    #endif

    var target_bend_vector = vec2<f32>(0.0);

    #ifdef STATIC_BEND
        target_bend_vector += static_bend;
    #endif

    let total_bend_amount = length(target_bend_vector);

    // Limit bending to prevent the mesh from curling into itself
    let max_allowed_bend = height_range * 0.95;
    let safe_bend_amount = min(total_bend_amount, max_allowed_bend);
    let bend_factor = clamp(total_bend_amount / height_range, 0.0, 1.0);
    let bend_stiffness = 0.33;

    // Estimate vertical height
    var tip_height = sqrt(max(height_range * height_range - safe_bend_amount * safe_bend_amount, 0.0));

    // The resulting Bezier curve arc is longer than the estimated straight-line distance.
    // Compensate by shortening the grass to prevent it from appearing to "grow" or stretch as it bends outward.
    let stretch_compensation = 1.0 - (bend_factor * 0.1);
    tip_height *= stretch_compensation;

    // Quadratic Bezier
    // P0: Start
    // P1: Control Point
    // P2: End Point
    let point_end = vec3<f32>(target_bend_vector.x, tip_height, target_bend_vector.y);
    let point_start = vec3<f32>(0.0, 0.0, 0.0);

    // Adjust control point height based on how much we are bending, i.e.,
    // pushing the curve up and making the tip bend more than the base.
    let control_point_y_factor = mix(0.5, 0.6, bend_factor);
    let control_point_y = height_range * control_point_y_factor;

    let point_control = vec3<f32>(
        target_bend_vector.x * bend_stiffness,
        control_point_y,
        target_bend_vector.y * bend_stiffness
    );

    // B(t) = (1-t)^2 * P0 + 2(1-t)t * P1 + t^2 * P2
    let inverse_progress = 1.0 - height_progress;
    let bezier_position = (inverse_progress * inverse_progress) * point_start
              + (2.0 * inverse_progress * height_progress) * point_control
              + (height_progress * height_progress) * point_end;

    // B'(t) = 2(1-t)(P1-P0) + 2t(P2-P1)
    let bezier_tangent = 2.0 * inverse_progress * (point_control - point_start)
                       + 2.0 * height_progress * (point_end - point_control);

    let effective_spine_y = height_progress * height_range;
    let spine_delta = bezier_position - vec3<f32>(0.0, effective_spine_y, 0.0);

    result.local_pos = vec3<f32>(
        local_pos.x + spine_delta.x,
        local_pos.y + spine_delta.y,
        local_pos.z + spine_delta.z
    );

    #ifdef WIND_AFFECTED
        let forward_dir = result.local_wind_dir;

        let macro_noise = clamp(noise.macro_noise, 0.001, 0.999) * 2.0 - 1.0;
        let macro_wind_offset = forward_dir * (macro_noise * wind.strength * 0.5 * result.height_factor);

        result.local_pos += macro_wind_offset;

        #ifndef BILLBOARDING
            result.twist = macro_noise * wind.twist_strength * height_progress;
        #endif
    #endif

    result.tangent = normalize(bezier_tangent + vec3<f32>(0.0, 1.0e-5, 0.0));

    return result;
}

fn apply_micro_details(
    pos: vec3<f32>,
    curve_data: CurveResult,
    wind: Wind,
    instance: InstanceInfo,
    noise: SampledNoise
) -> vec3<f32> {
    var final_pos = pos;

#ifdef WIND_AFFECTED
#ifndef WIND_LOW_QUALITY

    let forward_dir = curve_data.local_wind_dir;

    let up_dir = vec3<f32>(0.0, 1.0, 0.0);
    let right_dir = normalize(cross(forward_dir, up_dir));

    // Micro
    let micro_noise = (clamp(noise.micro_noise, 0.001, 0.999) * 2.0 - 1.0);
    let micro = forward_dir * (micro_noise * wind.micro_strength * 0.5 * curve_data.height_factor);

    // S-Curve
    let s_curve_seed = noise.phase_noise.x * 6.28;
    let s_curve_lag = curve_data.height_factor * wind.s_curve_frequency;
    let s_curve_input = instance.wrapped_time * wind.s_curve_speed + s_curve_seed + s_curve_lag;

    let s_primary_oscillation = sin(s_curve_input);
    let s_secondary_oscillation = cos(s_curve_input * 0.7) * 0.5;

    let s_curve = (forward_dir * s_primary_oscillation + right_dir * s_secondary_oscillation)
                 * wind.s_curve_strength
                 * curve_data.height_factor;

    // Bop
    let bop_seed = noise.phase_noise.y * 6.28;
    let bop_input = instance.wrapped_time * wind.bop_speed + bop_seed + s_curve_lag;
    let bop_val = sin(bop_input);

    let bop = vec3<f32>(0.0, bop_val * wind.bop_strength * curve_data.height_factor, 0.0);

    final_pos += micro + s_curve + bop;
#endif
#endif

    return final_pos;
}

fn billboarding(
    wind: Wind,
    instance: InstanceInfo,
    local_displaced_pos: vec3<f32>,
    total_offset: vec3<f32>
) -> vec3<f32> {
    let billboard_anchor_point = instance.instance_position;

    let billboard_matrix = calc_billboard_matrix(
        billboard_anchor_point,
        view.world_position.xyz,
        instance.world_from_local
    );

    let billboard_base_pos = billboard_anchor_point.xyz + (billboard_matrix * local_displaced_pos);

    return billboard_base_pos + vec3(0.0, total_offset.y, 0.0);
}

fn calc_billboard_matrix(
    instance_position: vec4<f32>,
    camera_world_pos: vec3<f32>,
    world_from_local: mat4x4<f32>
) -> mat3x3<f32> {
    let scale = vec3<f32>(
        length(world_from_local[0].xyz),
        length(world_from_local[1].xyz),
        length(world_from_local[2].xyz)
    );

    let to_camera_vector = camera_world_pos - instance_position.xyz;
    let billboard_z = normalize(vec3<f32>(to_camera_vector.x, 0.0, to_camera_vector.z));
    let billboard_y = vec3<f32>(0.0, 1.0, 0.0);
    let billboard_x = normalize(cross(billboard_y, billboard_z));

    return mat3x3<f32>(billboard_x * scale.x, billboard_y * scale.y, billboard_z * scale.z);
}

// TODO requires previous camera/view
fn calc_edge_correction(
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    uv_x: f32,
    edge_correction_factor: f32
) -> vec3<f32> {
    let signed_edge_factor = uv_x * 2.0 - 1.0;

    let to_camera_dir = normalize(view.world_position.xyz - world_pos);

    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let view_side_dir = normalize(cross(to_camera_dir, world_up));

    // Normal orthogonal to camera/view vec
    let normal_dot_view = dot(world_normal, to_camera_dir);
    let grazing_angle_factor = pow(1.0 - abs(normal_dot_view), 2.0);

    // Fade out correction when looking top-down
    let top_down_factor = abs(dot(to_camera_dir, world_up));
    let top_down_fade = pow(1.0 - top_down_factor, 0.5);

    // TODO remove * 0.
    let strength = grazing_angle_factor * edge_correction_factor * 0. * top_down_fade;

    let correction_shift = view_side_dir * -signed_edge_factor;

    return correction_shift * strength;
}
