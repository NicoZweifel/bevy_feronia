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
    wrapped_time: f32,
    instance_index: u32
}

fn calculate_vertex_displacement(
    local_pos: vec3<f32>,
    wind: Wind,
    noise: SampledNoise,
    instance: InstanceInfo,
    lod_fade: f32
) -> vec3<f32> {
    let normalized_height = local_pos.y;
    let c_curve_shape = pow(normalized_height, wind.bend_exponent);
    let twisted_local_pos = calculate_twist(wind, noise.macro_noise, c_curve_shape, local_pos);

    let macro_displacement = (noise.macro_noise * 2.0 - 1.0) * wind.strength * c_curve_shape;
    let horizontal_dir = vec3<f32>(wind.direction.x, 0.0, wind.direction.y);
    var total_world_offset = horizontal_dir * macro_displacement;

    if (lod_fade > 0.0) {
        let micro_displacement = (noise.micro_noise * 2.0 - 1.0) * wind.micro_strength * c_curve_shape;
        let micro_wind = horizontal_dir * micro_displacement;

        let s_curve = calculate_s_curve_displacement(
            wind,
            c_curve_shape,
            normalized_height,
            instance.wrapped_time,
            noise.phase_noise.x
        );

        let bop = calculate_bop_displacement(
            wind,
            c_curve_shape,
            instance.wrapped_time,
            noise.phase_noise.y
        );

        total_world_offset += (micro_wind + s_curve + bop) * lod_fade;
    }

    var final_world_pos: vec3<f32>;

    if (wind.enable_billboarding == 1u) {
        let billboard_matrix = calculate_billboard_matrix(instance.instance_position, view.world_position.xyz);
        let rotated_position = instance.instance_position.xyz + (billboard_matrix * (twisted_local_pos * instance.scale));
        final_world_pos = rotated_position + total_world_offset;

        if (wind.enable_edge_correction == 1u) {
            final_world_pos = calculate_edge_correction(
                final_world_pos,
                local_pos,
                wind
            );
        }
    } else {
        let world_pos = (instance.world_from_local * vec4<f32>(twisted_local_pos, 1.0)).xyz;
        final_world_pos = world_pos + total_world_offset;
    }

    return final_world_pos;
}

fn displace_vertex_and_calc_normal(
    wind: Wind,
    noise: SampledNoise,
    vertex_pos: vec3<f32>,
    instance: InstanceInfo,
    dist_to_camera: f32,
#ifdef VERTEX_NORMALS
    normal: vec3<f32>,
#endif
) -> DisplacedVertex {
    var out: DisplacedVertex;
    let small_offset = 0.01;
    let lod_fade = smoothstep(wind.lod_threshold * 2.0, wind.lod_threshold, dist_to_camera);

    let final_pos_xyz = calculate_vertex_displacement(vertex_pos, wind, noise, instance, lod_fade);
    out.world_position = vec4<f32>(final_pos_xyz, 1.0);

#ifdef VERTEX_NORMALS
    let mesh_normal = mesh_normal_local_to_world(normal, instance.instance_index);

    let lod = lod_fade > 0.0;

    if (wind.enable_billboarding == 1u || lod) {
        let neighbor_pos_x = calculate_vertex_displacement(vertex_pos + vec3<f32>(small_offset, 0.0, 0.0), wind, noise, instance, lod_fade);
        let neighbor_pos_z = calculate_vertex_displacement(vertex_pos + vec3<f32>(0.0, 0.0, small_offset), wind, noise, instance, lod_fade);
        let tangent_x = neighbor_pos_x - final_pos_xyz;
        let tangent_z = neighbor_pos_z - final_pos_xyz;
        let calculated_normal = normalize(cross(tangent_z, tangent_x));

        if (wind.enable_billboarding == 0u) {
            out.world_normal = mix(mesh_normal, calculated_normal, lod_fade);
        } else {
            out.world_normal = calculated_normal;
        }
    } else {
        out.world_normal = mesh_normal;
    }
#endif

    return out;
}

fn calculate_edge_correction(
    world_pos: vec3<f32>,
    local_pos: vec3<f32>,
    wind: Wind
) -> vec3<f32> {
    let view_vector = normalize(world_pos - view.world_position.xyz);
    
    let to_camera_flat = normalize(vec3(view.world_position.x, 0.0, view.world_position.z) - vec3(world_pos.x, 0.0, world_pos.z));
    let world_right = normalize(cross(vec3(0.0, 1.0, 0.0), to_camera_flat));

    let ortho_factor = 1.0 - abs(dot(view_vector, world_right));

    let offset_direction = world_right * sign(local_pos.x);

    let final_offset = offset_direction  * wind.edge_correction_factor * ortho_factor;
    
    return world_pos + final_offset;
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
    let twist_angle = twist * c_curve_shape;

    let cos_a = cos(twist_angle);
    let sin_a = sin(twist_angle);

    let rotated_x = local_pos.x * cos_a - local_pos.z * sin_a;
    let rotated_z = local_pos.x * sin_a + local_pos.z * cos_a;

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