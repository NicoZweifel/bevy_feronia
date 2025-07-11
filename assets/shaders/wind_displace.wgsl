#import bevy_render::globals
#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_normal_local_to_world},
    mesh_view_bindings::{view,globals}
};

#import "./shaders/wind.wgsl"::Wind


struct SampledNoise {
    macro_noise: f32,
    micro_noise: f32,
    phase_noise: vec2<f32>,
};

struct DisplacedVertex {
    world_position: vec4<f32>,
    world_normal: vec3<f32>,
}

struct InstanceInfo {
    world_from_local: mat4x4<f32>,
    instance_position: vec4<f32>,
    scale: f32,
    billboard_matrix: mat3x3<f32>,
    wrapped_time: f32,
    instance_index: u32
}

fn calculate_vertex_displacement(
    local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
    instance: InstanceInfo,
    dist_to_camera: f32
) -> vec3<f32> {
    // 1. Curve and Twist
    let normalized_height = local_pos.y;
    let c_curve_shape = pow(normalized_height, wind.bend_exponent);

    let twisted_local_pos = calculate_twist(wind, noise.macro_noise, c_curve_shape, local_pos);
/* 
    let lod_fade = smoothstep(wind.lod_threshold, wind.lod_threshold - (wind.lod_threshold * 0.5), dist_to_camera);
    let main_wind = calculate_main_wind_displacement(wind, c_curve_shape, noise.macro_noise, noise.micro_noise);
    let s_curve = calculate_s_curve_displacement(wind, c_curve_shape, normalized_height, instance.wrapped_time, noise.phase_noise.x);
    let bop = calculate_bop_displacement(wind, c_curve_shape, instance.wrapped_time, noise.phase_noise.y);
    var total_displacement =mix(mix(main_wind, s_curve, lod_fade),bop,lod_fade );
*/

    // 2. Billboarding
    var base_world_pos= (instance.world_from_local * vec4<f32>(twisted_local_pos, 1.0)).xyz;

    if wind.enable_billboarding == 1u{
        let billboarded_pos = instance.instance_position.xyz + (instance.billboard_matrix * (twisted_local_pos * instance.scale));
        base_world_pos = mix(base_world_pos, billboarded_pos, f32(wind.enable_billboarding));
    }

    // 3. Displacement effects
    let main_wind = calculate_main_wind_displacement(wind, c_curve_shape, noise.macro_noise, noise.micro_noise);
    let s_curve = calculate_s_curve_displacement(wind, c_curve_shape, normalized_height, instance.wrapped_time, noise.phase_noise.x);
    let bop = calculate_bop_displacement(wind, c_curve_shape, instance.wrapped_time, noise.phase_noise.y);

    let total_displacement = main_wind + s_curve + bop;

    return base_world_pos + total_displacement;
}

fn displace_vertex_and_calc_normal(
    wind: Wind,
    noise: SampledNoise,
    vertex_pos: vec3<f32>,
    instance: InstanceInfo,
#ifdef VERTEX_NORMALS
    normal: vec3<f32>,
#endif
) -> DisplacedVertex {
    var out: DisplacedVertex;
    let small_offset = 0.01;

    let dist_to_camera = distance(instance.instance_position.xyz, view.world_position.xyz);

    // CALCULATE POSITION
    let final_pos_xyz = calculate_vertex_displacement(vertex_pos, wind, noise, instance,dist_to_camera);
    out.world_position = vec4<f32>(final_pos_xyz, 1.0);

    // CALCULATE NORMALS
#ifdef VERTEX_NORMALS
    let lod_threshold = 75.0;

    // Calculate the normal by displacing neighbors
    let neighbor_pos_x = calculate_vertex_displacement(vertex_pos + vec3<f32>(small_offset, 0.0, 0.0), wind, noise, instance,dist_to_camera);
    let neighbor_pos_z = calculate_vertex_displacement(vertex_pos + vec3<f32>(0.0, 0.0, small_offset), wind, noise, instance,dist_to_camera);
    let tangent_x = neighbor_pos_x - final_pos_xyz;
    let tangent_z = neighbor_pos_z - final_pos_xyz;
    let calculated_normal = normalize(cross(tangent_z, tangent_x));

    // Get the original normal
    let mesh_normal = mesh_normal_local_to_world(normal, instance.instance_index);

    // Smoothly blend between the two normals based on distance to avoid a hard pop/ring.
    let lod_fade = smoothstep(lod_threshold, lod_threshold - 50.0, dist_to_camera);
    out.world_normal = mix(mesh_normal, calculated_normal, lod_fade);
#endif

    return out;
}

fn calculate_main_wind_displacement(
    wind: Wind,
    c_curve_shape: f32,
    macro_noise: f32,
    micro_noise: f32,
) -> vec3<f32> {
    let macro_displacement = (macro_noise * 2.0 - 1.0) * wind.strength * c_curve_shape;
    let micro_displacement = (micro_noise * 2.0 - 1.0) * wind.micro_strength * c_curve_shape;

    let combined_displacement = macro_displacement + micro_displacement;
    let horizontal_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);

    return horizontal_dir * combined_displacement;
}

fn calculate_s_curve_displacement(
    wind: Wind,
    c_curve_shape: f32,
    normalized_height: f32,
    wrapped_time: f32,
    s_curve_seed: f32,
) -> vec3<f32> {
    let s_curve_phase_offset = s_curve_seed * 6.28318;
    let s_curve_anim = sin(wrapped_time * wind.s_curve_speed + s_curve_phase_offset);
    let s_curve_wiggles = sin(normalized_height * wind.s_curve_frequency);

    let final_s_curve_shape = c_curve_shape + (s_curve_wiggles * wind.s_curve_strength * c_curve_shape);
    let s_curve_amount = s_curve_anim * wind.s_curve_strength * final_s_curve_shape;
    let horizontal_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);

    return horizontal_dir * s_curve_amount;
}

fn calculate_bop_displacement(
    wind: Wind,
    c_curve_shape: f32,
    wrapped_time: f32,
    bop_seed: f32,
) -> vec3<f32> {
    let bop_phase_offset = bop_seed * 6.28318;
    let bop_value = sin(wrapped_time * wind.bop_speed + bop_phase_offset);
    let vertical_amount = bop_value * wind.bop_strength * c_curve_shape;

    return vec3<f32>(0.0, vertical_amount, 0.0);
}

fn calculate_twist(
    wind: Wind,
    macro_noise: f32,
    c_curve_shape: f32,
    local_pos: vec3<f32>,
) -> vec3<f32> {
    let twist = (macro_noise * 2.0 - 1.0) * wind.twist_strength;
    let wind_angle = atan2(wind.direction.y, wind.direction.x);
    let top_twist_angle = wind_angle + twist;

    let twist_angle = top_twist_angle * c_curve_shape;

    let cos_a = cos(twist_angle);
    let sin_a = sin(twist_angle);

    let rotated_x = local_pos.x * cos_a + local_pos.z * sin_a;
    let rotated_z = -local_pos.x * sin_a + local_pos.z * cos_a;

    return vec3<f32>(rotated_x, local_pos.y, rotated_z);
}

fn calculate_billboard_matrix(
    instance_position: vec4<f32>,
    camera_world_pos: vec3<f32>,
) -> mat3x3<f32> {
    let to_camera = camera_world_pos - instance_position.xyz;
    let new_z = normalize(vec3<f32>(to_camera.x, 0.0, to_camera.z));
    let new_y = vec3<f32>(0.0, 1.0, 0.0);
    let new_x = normalize(cross(new_y, new_z));

    return mat3x3<f32>(new_x, new_y, new_z);
}
