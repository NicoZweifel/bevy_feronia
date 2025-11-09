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
#import bevy_feronia::wind::{Wind, BindlessWindIndices}
#import bevy_feronia::noise::sample_noise

struct DisplacementCache {
    normalized_height: f32,
    height_attenuation_factor: f32,
    bend_curve: f32,
    macro_wind_factor: f32,
    twist_angle: f32,
    bent_twisted_local_pos: vec3<f32>,
    horizontal_wind_dir: vec3<f32>,
    micro_wind_factor: f32,
    macro_wind: vec3<f32>,
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
        instance,
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

#ifdef FAST_NORMALS
    // Uses the original, un-displaced world-space normals.
    //
    // Will have incorrect lighting as the normals will not match the displaced vertex positions.
    //
    // The mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
    // and its "face" pointing along Z-Up (`+Z`).
    //
    // Should be used for performance reasons and/or on static or barely wind affected objects.
    normal_data.world_normal = mesh_normal;

#ifdef VERTEX_TANGENTS
    normal_data.world_tangent = mesh_tangent;
#else
    let local_fallback_tangent = vec4<f32>(0.0, 1.0, 0.0, 1.0);

    let world_tangent_xyz = (instance.world_from_local * vec4<f32>(local_fallback_tangent.xyz, 0.0)).xyz;

    normal_data.world_tangent = vec4<f32>(normalize(world_tangent_xyz), local_fallback_tangent.w);
#endif

#else // NOT FAST_NORMALS
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
    normal_data = calculate_analytical_normal(
        cache,
        wind,
        instance,
        vertex_pos,
#ifdef STATIC_BEND
        static_bend,
#endif
#ifdef VERTEX_NORMALS
        normal,
#endif
#ifdef VERTEX_TANGENTS
        tangent
#endif
    );
#else // NOT ANALYTICAL_NORMALS
    // Calculates normals numerically by sampling neighboring positions.
    //
    // Should be the most accurate, but most expensive path, as it runs the full
    // displacement logic on the neighbors to find the surface direction.
    //
    // Typically used for complex foliage like non-billboarded bushes, trees.
    normal_data = calculate_numerical_normal(
        final_pos_xyz,
        vertex_pos,
        wind,
        instance,
#ifdef STATIC_BEND
        static_bend,
#endif
#ifdef VERTEX_NORMALS
        normal,
#endif
#ifdef VERTEX_TANGENTS
        tangent
#endif
    );
#endif // ANALYTICAL_NORMALS
#endif // FAST_NORMALS

    out.world_normal = normal_data.world_normal;
    out.world_tangent = normal_data.world_tangent;

#else // NOT VERTEX_NORMALS
    out.world_normal = vec3<f32>(0.0, 0.0, 1.0);
#endif // VERTEX_NORMALS

#ifdef EDGE_CORRECTION
    // TODO
#endif

    return out;
}




fn displacement_cache(
    local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
    instance: InstanceInfo,
#ifdef STATIC_BEND
    static_bend: vec2<f32>,
#endif
) -> DisplacementCache {
    var cache: DisplacementCache;
    var pos = local_pos;

    let height_range = wind.aabb_max.y - wind.aabb_min.y;
    cache.normalized_height = (local_pos.y - wind.aabb_min.y) / max(height_range, 0.0001);
    cache.height_attenuation_factor = pow(cache.normalized_height, wind.bend_exponent);

    cache.bend_curve = 0.0;

#ifdef STATIC_BEND
    if (cache.normalized_height > 0.0001 && wind.bend_exponent > 0.0) {
        cache.bend_curve = wind.bend_exponent
            * pow(cache.normalized_height, wind.bend_exponent - 1.0) * (1.0 / max(height_range, 0.0001));
    }

    let bend_offset = static_bend * cache.height_attenuation_factor;
    pos.x += bend_offset.x;
    pos.z += bend_offset.y;
#endif

    cache.macro_wind_factor = 0.0;
    cache.twist_angle = 0.0;
    cache.horizontal_wind_dir = vec3<f32>(0.0);
    cache.micro_wind_factor = 0.0;
    cache.macro_wind = vec3<f32>(0.0);

#ifdef WIND_AFFECTED
    cache.horizontal_wind_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);

    let clamped_macro_noise = clamp(noise.macro_noise, 0.001, 1.0 - 0.001);
    cache.macro_wind_factor = clamped_macro_noise * 2.0 - 1.0;

    let clamped_micro_noise = clamp(noise.micro_noise, 0.001, 1.0 - 0.001);
    cache.micro_wind_factor = clamped_micro_noise * 2.0 - 1.0;

    cache.macro_wind = cache.horizontal_wind_dir
        * wind.strength
        * cache.macro_wind_factor
        * cache.height_attenuation_factor;

#ifndef BILLBOARDING

    let twist = cache.macro_wind_factor * wind.twist_strength;
    cache.twist_angle = twist * cache.height_attenuation_factor;

    let cos_a = cos(cache.twist_angle);
    let sin_a = sin(cache.twist_angle);

    let rotated_x = pos.x * cos_a - pos.z * sin_a;
    let rotated_z = pos.x * sin_a + pos.z * cos_a;
    pos = vec3<f32>(rotated_x, pos.y, rotated_z);

#endif // NOT BILLBOARDING
#endif // WIND_AFFECTED

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
    let macro_displacement = cache.macro_wind;
    total_world_offset = cache.horizontal_wind_dir * macro_displacement;

#ifndef WIND_LOW_QUALITY
    let micro_displacement = cache.micro_wind_factor * wind.micro_strength * cache.height_attenuation_factor;
    let micro_wind = cache.horizontal_wind_dir * micro_displacement;

    let s_curve = calculate_s_curve_displacement(
        wind,
        cache.height_attenuation_factor,
        cache.normalized_height,
        instance.wrapped_time,
        noise.phase_noise.x,
        cache.horizontal_wind_dir
    );

    let bop = calculate_bop_displacement(
        wind,
        cache.height_attenuation_factor,
        instance.wrapped_time,
        noise.phase_noise.y
    );

    total_world_offset += micro_wind + s_curve + bop;
#endif // NOT WIND_LOW_QUALITY
#endif // WIND_AFFECTED

    var final_world_pos = (instance.world_from_local * vec4<f32>(pos, 1.0)).xyz;
    final_world_pos += total_world_offset;

#ifdef BILLBOARDING
    final_world_pos = billboarding(wind, instance, pos, total_world_offset);
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
        instance,
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

#ifdef ANALYTICAL_NORMALS
fn calculate_analytical_normal(
    cache: DisplacementCache,
    wind: Wind,
    instance: InstanceInfo,
    vertex_pos: vec3<f32>,
#ifdef STATIC_BEND
    static_bend:vec2<f32>,
#endif
#ifdef VERTEX_NORMALS
    in_normal: vec3<f32>,
#endif
#ifdef VERTEX_TANGENTS
    in_tangent: vec4<f32>,
#endif
) -> DisplacedVertex {
    var out: DisplacedVertex;

#ifdef VERTEX_NORMALS
    let normal = in_normal;
#else
    let normal = vec3<f32>(0.0, 0.0, 1.0);
#endif
#ifdef VERTEX_TANGENTS
    let tangent = in_tangent;
#else
    let tangent = vec4<f32>(0.0, 1.0, 0.0, 1.0);
#endif

    var local_tangent = tangent.xyz;
    var local_normal = normal;
    let local_tangent_w = tangent.w;

#ifdef STATIC_BEND
    let bend_x = static_bend.x * cache.bend_curve;
    let bend_z = static_bend.y * cache.bend_curve;

    let bent_tangent_vec = vec3<f32>(bend_x, 1.0, bend_z);

    let local_bitangent = cross(local_normal, local_tangent);

    local_normal = normalize(cross(bent_tangent_vec, local_bitangent));
    local_tangent = normalize(bent_tangent_vec);
#endif // STATIC_BEND

#ifndef BILLBOARDING
    let cos_a = cos(cache.twist_angle);
    let sin_a = sin(cache.twist_angle);

    local_normal = normalize(vec3<f32>(
        local_normal.x * cos_a - local_normal.z * sin_a,
        local_normal.y,
        local_normal.x * sin_a + local_normal.z * cos_a
    ));

    local_tangent = normalize(vec3<f32>(
        local_tangent.x * cos_a - local_tangent.z * sin_a,
        local_tangent.y,
        local_tangent.x * sin_a + local_tangent.z * cos_a
    ));
#endif // NOT BILLBOARDING

    // Wind displacement is in world space.
    let model_3x3 = mat3x3<f32>(
        instance.world_from_local[0].xyz,
        instance.world_from_local[1].xyz,
        instance.world_from_local[2].xyz
    );

    let world_normal_dir = normalize(model_3x3 * local_normal);
    let world_tangent_dir = normalize(model_3x3 * local_tangent);

#ifdef WIND_AFFECTED
    let macro_wind = cache.macro_wind;
    let world_bitangent = cross(world_normal_dir, world_tangent_dir);

    let final_bitangent = world_bitangent + macro_wind;
    let final_world_normal = normalize(
        world_normal_dir + cross(world_tangent_dir, macro_wind)
    );
    let final_world_tangent = normalize(cross(final_bitangent, final_world_normal));

    out.world_normal = final_world_normal;
    out.world_tangent = vec4<f32>(final_world_tangent, local_tangent_w);
#else
    out.world_normal = world_normal_dir;
    out.world_tangent = vec4<f32>(world_tangent_dir, local_tangent_w);
#endif

    return out;
}
#endif

fn calculate_numerical_normal(
    final_pos_xyz: vec3<f32>,
    vertex_pos: vec3<f32>,
    wind: Wind,
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
) -> DisplacedVertex {
    var out: DisplacedVertex;
    let small_offset = 0.01;

#ifdef VERTEX_NORMALS
    let local_normal = normal;
#else
    let local_normal = vec3<f32>(0.0, 0.0, 1.0);
#endif

#ifdef VERTEX_TANGENTS
    let local_tangent_vec4 = tangent;
#else
    let local_tangent_vec4 = vec4<f32>(0.0, 1.0, 0.0, 1.0);
#endif

    let local_tangent = local_tangent_vec4.xyz;
    let local_bitangent = cross(local_normal, local_tangent) * local_tangent_vec4.w;

    let local_pos_tangent = vertex_pos + local_tangent * small_offset;
    let noise_tangent = sample_noise(instance, local_pos_tangent);

    let neighbor_pos_tangent = calculate_vertex_displacement(
        local_pos_tangent,
        wind,
        noise_tangent,
        instance,
    #ifdef STATIC_BEND
        static_bend,
    #endif
    );

    let local_pos_bitangent = vertex_pos + local_bitangent * small_offset;
    let noise_bitangent = sample_noise(instance, local_pos_bitangent);

    let neighbor_pos_bitangent = calculate_vertex_displacement(
        local_pos_bitangent,
        wind,
        noise_bitangent,
        instance,
    #ifdef STATIC_BEND
        static_bend,
    #endif
    );

    let surface_delta_tangent_dir = neighbor_pos_tangent - final_pos_xyz;
    let surface_delta_bitangent_dir = neighbor_pos_bitangent - final_pos_xyz;

    out.world_normal = normalize(cross(surface_delta_tangent_dir, surface_delta_bitangent_dir));

    let approximated_world_tangent = normalize(surface_delta_tangent_dir);
    let orthogonalized_tangent = normalize(approximated_world_tangent
        - dot(approximated_world_tangent, out.world_normal) * out.world_normal);

    out.world_tangent = vec4<f32>(orthogonalized_tangent, local_tangent_vec4.w);

    return out;
}

fn calculate_s_curve_displacement(
    wind: Wind,
    height_attenuation_factor: f32,
    normalized_height: f32,
    wrapped_time: f32,
    s_curve_seed: f32,
    horizontal_wind_dir: vec3<f32>
) -> vec3<f32> {
    let s_curve_phase_offset = s_curve_seed * 6.28318;
    let s_curve_anim = sin(wrapped_time * wind.s_curve_speed + s_curve_phase_offset);
    let s_curve_wiggles = sin(normalized_height * wind.s_curve_frequency);

    let final_s_curve_shape = height_attenuation_factor
        + (s_curve_wiggles * wind.s_curve_strength * height_attenuation_factor);
    let s_curve_amount = s_curve_anim * wind.s_curve_strength * final_s_curve_shape;

    return horizontal_wind_dir * s_curve_amount;
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