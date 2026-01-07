#define_import_path bevy_feronia::types

struct SampledNoise {
    macro_noise: f32,
    micro_noise: f32,
    phase_noise: vec2<f32>,
};

struct DisplacedVertex {
    world_position: vec4<f32>,
    world_normal: vec3<f32>,
    world_tangent: vec4<f32>
}

struct InstanceInfo {
    world_from_local: mat4x4<f32>,
    instance_position: vec4<f32>,
    wrapped_time: f32,
    instance_index: u32,
    edge_correction_factor: f32,
#ifdef VERTEX_TANGENTS
    tangent: vec4<f32>
#endif
}
