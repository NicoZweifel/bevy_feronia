#define_import_path bevy_feronia::wind

struct Wind {
    direction: vec2<f32>,
    strength: f32,
    noise_scale: f32,
    scroll_speed: f32,
    bend_exponent: f32,
    curve_factor: f32,
    micro_strength: f32,
    s_curve_speed: f32,
    s_curve_strength: f32,
    s_curve_frequency: f32,
    bop_speed: f32,
    bop_strength: f32,
    twist_strength: f32,
    edge_correction_factor: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
    debug_color: vec4<f32>
};

struct WindMaterialUniform {
    wind: Wind,
};

struct BindlessWindIndices {
    material: u32,
    noise_texture: u32,
    noise_texture_sampler: u32,
}


