// The use of layered macro/micro noise, secondary motions (S-curve, bop)
// and the methods for normal curving and edge correction are all heavily inspired by:

// "Ghost of Tsushima" and Eric Wohllaib
// of Sucker Punch Productions and the GDC 2021 talk:
// "Advanced Graphics Summit: Procedural Grass in 'Ghost of Tsushima'".

#define_import_path bevy_feronia::displace
#import bevy_pbr::mesh_functions::{mesh_normal_local_to_world, mesh_tangent_local_to_world}
#import bevy_pbr::mesh_view_bindings::view

#import bevy_feronia::types::{SampledNoise, DisplacedVertex, InstanceInfo}
#import bevy_feronia::wind::{Wind, BindlessWindIndices}
#import bevy_feronia::noise::sample_noise

struct DisplacementCache {
    normalized_height: f32,
    height_attenuation_factor: f32,
    bend_curve_derivative: f32,
    macro_wind_factor: f32,
    twist_angle: f32,
    bent_twisted_local_pos: vec3<f32>,
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
    tangent: vec4<f32>
#endif
) -> DisplacedVertex {
    var out: DisplacedVertex;

    let cache = displacement_cache(
        vertex_pos,
        wind,
        noise,
    #ifdef STATIC_BEND
        static_bend,
    #endif
    );

    let final_pos_xyz = finish_vertex_displacement(
        cache,
        vertex_pos,
        wind,
        noise,
        instance
    );

    out.world_position = vec4<f32>(final_pos_xyz, 1.0);

#ifdef VERTEX_NORMALS
    let mesh_normal = mesh_normal_local_to_world(normal, instance.instance_index);

    #ifdef VERTEX_TANGENTS
      let mesh_tangent = mesh_tangent_local_to_world(instance.world_from_local, tangent, instance.instance_index);
    #endif

    var normal_data: DisplacedVertex;
    var normal_fallback = false;

    #ifdef FAST_NORMALS
        normal_fallback = true;
    #else
        #ifdef ANALYTICAL_NORMALS
            normal_fallback = false;
        #else
            #ifdef WIND_AFFECTED
                normal_fallback = false;
            #endif
            #ifdef STATIC_BEND
                normal_fallback = false;
            #endif
        #endif // ANALYTICAL_NORMALS
    #endif // FAST_NORMALS

    if normal_fallback {
        normal_data = calculate_fallback_normal(
            mesh_normal,
        #ifdef VERTEX_TANGENTS
            mesh_tangent,
        #endif
            vertex_pos,
            wind
        );
    } else {
        #ifdef ANALYTICAL_NORMALS
            normal_data = calculate_analytical_normal(
                cache,
                wind,
                instance,
                vertex_pos,
                normal,
            #ifdef STATIC_BEND
                static_bend,
            #endif
            #ifdef VERTEX_TANGENTS
                tangent,
            #endif
            );
        #else
            normal_data = calculate_numerical_normal(
                final_pos_xyz,
                vertex_pos,
                wind,
                instance,
            #ifdef STATIC_BEND
                static_bend,
            #endif
            #ifdef VERTEX_TANGENTS
                tangent
            #endif
            );
        #endif // ANALYTICAL_NORMALS
    }

    out.world_normal = normal_data.world_normal;
#ifdef VERTEX_TANGENTS
    out.world_tangent = normal_data.world_tangent;
#endif

#else // NOT VERTEX_NORMALS
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
#endif // VERTEX_NORMALS

    return out;
}


// Adjusts the world normal to simulate a "curved" blade surface by
// pushing the normal outwards along the tangent based on the
// vertex's distance from the local X-axis center.
fn curve_normal(
    base_normal_world: vec3<f32>,
    tangent_world: vec3<f32>,
    local_pos: vec3<f32>,
    wind: Wind
) -> vec3<f32> {
    if (wind.curve_factor <= 0.0) {
        return base_normal_world;
    }

    let center_x = (wind.aabb_max.x + wind.aabb_min.x) * 0.5;
    let half_width = (wind.aabb_max.x - wind.aabb_min.x) * 0.5;

    let signed_norm_x = (local_pos.x - center_x) / max(half_width, 0.0001);

    let curve_angle =  wind.curve_factor * abs(signed_norm_x);

    let clamped_angle = clamp(curve_angle, 0.0, 1.4); // ~80 deg
    let offset_mag = sin(clamped_angle);

    let curve_offset_world = tangent_world * offset_mag * sign(signed_norm_x);

    return normalize(base_normal_world + curve_offset_world);
}

fn displacement_cache(
    local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
) -> DisplacementCache {
    var cache: DisplacementCache;
    var pos = local_pos;

    let height_range = wind.aabb_max.y - wind.aabb_min.y;
    cache.normalized_height = (local_pos.y - wind.aabb_min.y) / max(height_range, 0.0001);
    cache.height_attenuation_factor = pow(cache.normalized_height, wind.bend_exponent);

    cache.bend_curve_derivative = 0.0;
    if (cache.normalized_height > 0.0001 && wind.bend_exponent > 0.0) {
        cache.bend_curve_derivative = wind.bend_exponent * pow(cache.normalized_height, wind.bend_exponent - 1.0) * (1.0 / max(height_range, 0.0001));
    }

#ifdef STATIC_BEND
    let bend_offset = static_bend * cache.height_attenuation_factor;
    pos.x += bend_offset.x;
    pos.z += bend_offset.y;
#endif

    cache.macro_wind_factor = 0.0;
    cache.twist_angle = 0.0;

#ifdef WIND_AFFECTED
    let clamped_macro_noise = clamp(noise.macro_noise, 0.001, 1.0 - 0.001);
    cache.macro_wind_factor = clamped_macro_noise * 2.0 - 1.0;

    #ifndef WIND_BILLBOARDING
        let twist = cache.macro_wind_factor * wind.twist_strength;
        cache.twist_angle = twist * cache.height_attenuation_factor;

        let cos_a = cos(cache.twist_angle);
        let sin_a = sin(cache.twist_angle);

        let rotated_x = pos.x * cos_a - pos.z * sin_a;
        let rotated_z = pos.x * sin_a + pos.z * cos_a;
        pos = vec3<f32>(rotated_x, pos.y, rotated_z);
    #endif
#endif

    cache.bent_twisted_local_pos = pos;

    return cache;
}

fn finish_vertex_displacement(
    cache: DisplacementCache,
    original_local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
    instance: InstanceInfo,
) -> vec3<f32> {
    var total_world_offset = vec3<f32>(0.0);
    let pos = cache.bent_twisted_local_pos;

#ifdef WIND_AFFECTED
    let horizontal_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);
    let macro_displacement = cache.macro_wind_factor * wind.strength * cache.height_attenuation_factor;
    total_world_offset = horizontal_dir * macro_displacement;

    #ifndef WIND_LOW_QUALITY
        let clamped_micro_noise = clamp(noise.micro_noise, 0.001, 1.0 - 0.001);
        let micro_wind_factor = clamped_micro_noise * 2.0 - 1.0;
        let micro_displacement = micro_wind_factor * wind.micro_strength * cache.height_attenuation_factor;
        let micro_wind = horizontal_dir * micro_displacement;

        let s_curve = calculate_s_curve_displacement(wind, cache.height_attenuation_factor, cache.normalized_height, instance.wrapped_time, noise.phase_noise.x);
        let bop = calculate_bop_displacement(wind, cache.height_attenuation_factor, instance.wrapped_time, noise.phase_noise.y);

        total_world_offset += micro_wind + s_curve + bop;
    #endif // WIND_LOW_QUALITY
#endif

    var final_world_pos = (instance.world_from_local * vec4<f32>(pos, 1.0)).xyz;
    final_world_pos += total_world_offset;

#ifdef WIND_BILLBOARDING
    final_world_pos = billboarding(wind, instance, pos, total_world_offset);
#endif

#ifdef WIND_EDGE_CORRECTION
    final_world_pos = calculate_edge_correction(
        final_world_pos,
        original_local_pos,
        wind,
        instance
    );
#endif

    return final_world_pos;
}


fn calculate_vertex_displacement(
    local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
    instance: InstanceInfo,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
) -> vec3<f32> {
    let cache = displacement_cache(
        local_pos,
        wind,
        noise,
    #ifdef STATIC_BEND
        static_bend,
    #endif
    );

    return finish_vertex_displacement(
        cache,
        local_pos,
        wind,
        noise,
        instance
    );
}

fn billboarding(
    wind: Wind,
    instance: InstanceInfo,
    pos: vec3<f32>,
    total_world_offset: vec3<f32>,
) -> vec3<f32> {
    let billboard_anchor = instance.instance_position + vec4<f32>(total_world_offset.x, 0.0, total_world_offset.z, 0.0);

    let billboard_matrix = calculate_billboard_matrix(
        billboard_anchor,
        view.world_position.xyz,
        instance.world_from_local
    );

    let billboard_base = billboard_anchor.xyz + (billboard_matrix * pos);

    return billboard_base + vec3(0.0, total_world_offset.y, 0.0);
}


fn calculate_analytical_normal(
    cache: DisplacementCache,
    wind: Wind,
    instance: InstanceInfo,
    vertex_pos: vec3<f32>,
    normal: vec3<f32>,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    tangent: vec4<f32>,
#endif
) -> DisplacedVertex {
    var out: DisplacedVertex;
    let local_normal_dir = normal;
#ifdef VERTEX_TANGENTS
    let local_tangent_dir = tangent.xyz;
#else
    let local_tangent_dir = vec3<f32>(1.0, 0.0, 0.0);
#endif

    // Static Bend
#ifdef STATIC_BEND
    let bent_local_normal = normalize(vec3<f32>(
        local_normal_dir.x,
        local_normal_dir.y - local_normal_dir.x * static_bend.x * cache.bend_curve_derivative - local_normal_dir.z * static_bend.y * cache.bend_curve_derivative,
        local_normal_dir.z
    ));
    let bent_local_tangent = normalize(vec3<f32>(
        local_tangent_dir.x + local_tangent_dir.y * static_bend.x * cache.bend_curve_derivative,
        local_tangent_dir.y,
        local_tangent_dir.z + local_tangent_dir.y * static_bend.y * cache.bend_curve_derivative
    ));
#else
    let bent_local_normal = local_normal_dir;
    let bent_local_tangent = local_tangent_dir;
#endif

    // Twist
    let cos_a = cos(cache.twist_angle);
    let sin_a = sin(cache.twist_angle);

    let twisted_local_normal = normalize(vec3<f32>(
        bent_local_normal.x * cos_a - bent_local_normal.z * sin_a,
        bent_local_normal.y,
        bent_local_normal.x * sin_a + bent_local_normal.z * cos_a
    ));
    let twisted_local_tangent = normalize(vec3<f32>(
        bent_local_tangent.x * cos_a - bent_local_tangent.z * sin_a,
        bent_local_tangent.y,
        bent_local_tangent.x * sin_a + bent_local_tangent.z * cos_a
    ));

    var model_3x3: mat3x3<f32>;
    let horizontal_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);

    #ifdef WIND_BILLBOARDING
        let macro_displacement = cache.macro_wind_factor * wind.strength * cache.height_attenuation_factor;
        let macro_world_offset = horizontal_dir * macro_displacement;

        let billboard_anchor = instance.instance_position + vec4<f32>(macro_world_offset.x, 0.0, macro_world_offset.z, 0.0);

        model_3x3 = calculate_billboard_matrix(
            billboard_anchor,
            view.world_position.xyz,
            instance.world_from_local
        );
    #else
        model_3x3 = mat3x3<f32>(
            instance.world_from_local[0].xyz,
            instance.world_from_local[1].xyz,
            instance.world_from_local[2].xyz
        );
    #endif

    let world_normal_dir = normalize(model_3x3 * twisted_local_normal);
    let world_tangent_dir = normalize(model_3x3 * twisted_local_tangent);

    // Macro Wind
    let macro_wind_derivative = horizontal_dir * wind.strength * cache.macro_wind_factor * cache.bend_curve_derivative;

    let leaned_world_normal = normalize(vec3<f32>(
        world_normal_dir.x - world_normal_dir.y * macro_wind_derivative.x,
        world_normal_dir.y,
        world_normal_dir.z - world_normal_dir.y * macro_wind_derivative.z
    ));
    let leaned_world_tangent = normalize(vec3<f32>(
        world_tangent_dir.x + world_tangent_dir.y * macro_wind_derivative.x,
        world_tangent_dir.y,
        world_tangent_dir.z + world_tangent_dir.y * macro_wind_derivative.z
    ));

    out.world_normal = curve_normal(leaned_world_normal, leaned_world_tangent, vertex_pos, wind);

#ifdef VERTEX_TANGENTS
    out.world_tangent = vec4<f32>(leaned_world_tangent, tangent.w);
#endif

    return out;
}

fn calculate_numerical_normal(
    final_pos_xyz: vec3<f32>,
    vertex_pos: vec3<f32>,
    wind: Wind,
    instance: InstanceInfo,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    tangent: vec4<f32>,
#endif
) -> DisplacedVertex {
    var out: DisplacedVertex;
    let small_offset = 0.01;

    let local_pos_x = vertex_pos + vec3<f32>(small_offset, 0.0, 0.0);
    let noise_x = sample_noise(instance, local_pos_x);
    let neighbor_pos_x = calculate_vertex_displacement(
        vertex_pos + vec3<f32>(small_offset, 0.0, 0.0),
        wind,
        noise_x,
        instance,
    #ifdef STATIC_BEND
        static_bend,
    #endif
    );

    let local_pos_z = vertex_pos + vec3<f32>(0.0, 0.0, small_offset);
    let noise_z = sample_noise(instance, local_pos_z);
    let neighbor_pos_z = calculate_vertex_displacement(
        vertex_pos + vec3<f32>(0.0, 0.0, small_offset),
        wind,
        noise_z,
        instance,
    #ifdef STATIC_BEND
        static_bend,
    #endif
    );

    let surface_delta_x = neighbor_pos_x - final_pos_xyz;
    let surface_delta_z = neighbor_pos_z - final_pos_xyz;

    let approximated_world_normal = normalize(cross(surface_delta_z, surface_delta_x));

#ifdef VERTEX_TANGENTS
    let approximated_world_tangent = normalize(tangent.x * surface_delta_x + tangent.z * surface_delta_z);
    let orthogonalized_tangent = normalize(approximated_world_tangent - dot(approximated_world_tangent, approximated_world_normal) * approximated_world_normal);

    out.world_normal = curve_normal(approximated_world_normal, orthogonalized_tangent, vertex_pos, wind);
    out.world_tangent = vec4<f32>(orthogonalized_tangent, tangent.w);
#else
    out.world_normal = approximated_world_normal;
#endif

    return out;
}

fn calculate_fallback_normal(
    mesh_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    mesh_tangent: vec4<f32>,
#endif
    vertex_pos: vec3<f32>,
    wind: Wind
) -> DisplacedVertex {
    var out: DisplacedVertex;

#ifdef VERTEX_TANGENTS
    let world_tangent_dir = mesh_tangent.xyz;
    out.world_normal = curve_normal(mesh_normal, world_tangent_dir, vertex_pos, wind);
    out.world_tangent = mesh_tangent;
#else
    out.world_normal = mesh_normal;
#endif

    return out;
}

// Calculates a view-dependent offset for vegetation (like grass)
// to make it appear fuller when viewed from sharp angles.
fn calculate_edge_correction(
    world_pos: vec3<f32>,
    local_pos: vec3<f32>,
    wind: Wind,
    instance: InstanceInfo,
) -> vec3<f32> {
    // Normal orthogonal to view vector
    let camera_to_pos = world_pos - view.world_position.xyz;
    let view_vector = normalize(camera_to_pos);

    let instance_right_world = normalize(instance.world_from_local[0].xyz);
    let instance_up_world = normalize(instance.world_from_local[1].xyz);
    let instance_forward_world = normalize(instance.world_from_local[2].xyz);

    let ortho_factor_for_edge = 1.0 - abs(dot(view_vector, instance_forward_world));
    let smooth_factor_edge = pow(ortho_factor_for_edge, 2.0);

    // Fade out the effect when looking straight down or up
    let dot_view_up = abs(dot(view_vector, instance_up_world));
    let top_down_fade = pow(1.0 - dot_view_up, 0.5);

    let smooth_factor = smooth_factor_edge * top_down_fade;

    // Shift
    let center_x = (wind.aabb_max.x + wind.aabb_min.x) * 0.5;
    let signed_local_x_dist = local_pos.x - center_x;

    let max_distance_x = (wind.aabb_max.x - wind.aabb_min.x) * 0.5;
    let normalized_x_distance = abs(signed_local_x_dist) / max(max_distance_x, 0.0001);

    let final_offset_magnitude = normalized_x_distance * wind.edge_correction_factor * smooth_factor;

    let final_shift_direction = instance_right_world * sign(signed_local_x_dist);

    let final_offset = final_shift_direction * final_offset_magnitude;

    return world_pos + final_offset;
}

fn calculate_s_curve_displacement(
    wind: Wind,
    height_attenuation_factor: f32,
    normalized_height: f32,
    wrapped_time: f32,
    s_curve_seed: f32,
) -> vec3<f32> {
    let s_curve_phase_offset = s_curve_seed * 6.28318;
    let s_curve_anim = sin(wrapped_time * wind.s_curve_speed + s_curve_phase_offset);
    let s_curve_wiggles = sin(normalized_height * wind.s_curve_frequency);

    let final_s_curve_shape = height_attenuation_factor + (s_curve_wiggles * wind.s_curve_strength * height_attenuation_factor);
    let s_curve_amount = s_curve_anim * wind.s_curve_strength * final_s_curve_shape;
    let horizontal_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);

    return horizontal_dir * s_curve_amount;
}

fn calculate_bop_displacement(
    wind: Wind,
    height_attenuation_factor: f32,
    wrapped_time: f32,
    bop_seed: f32,
) -> vec3<f32> {
    let bop_phase_offset = bop_seed * 6.28318;
    let bop_value = sin(wrapped_time * wind.bop_speed + bop_phase_offset);
    let vertical_amount = bop_value * wind.bop_strength * height_attenuation_factor;

    return vec3<f32>(0.0, vertical_amount, 0.0);
}

fn calculate_billboard_matrix(
    instance_position: vec4<f32>,
    camera_world_pos: vec3<f32>,
    world_from_local: mat4x4<f32>
) -> mat3x3<f32> {
    let scale = vec3<f32>(
        length(world_from_local[0].xyz),
        length(world_from_local[1].xyz),
        length(world_from_local[2].xyz)
    );

    let to_camera = camera_world_pos - instance_position.xyz;
    let new_z = normalize(vec3<f32>(to_camera.x, 0.0, to_camera.z));
    let new_y = vec3<f32>(0.0, 1.0, 0.0);
    let new_x = normalize(cross(new_y, new_z));

    return mat3x3<f32>(new_x * scale.x, new_y * scale.y, new_z * scale.z);
}