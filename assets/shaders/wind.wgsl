struct Wind {
    direction: vec2<f32>,
    strength: f32,
    noise_scale: f32,
    scroll_speed: f32,
    bend_exponent: f32,
    round_exponent: f32,
    micro_strength: f32,
    micro_noise_scale: f32,
    micro_scroll_speed: f32,
    s_curve_speed: f32,
    s_curve_strength: f32,
    s_curve_frequency: f32,
    bop_speed: f32,
    bop_strength: f32,
    twist_strength: f32,
    enable_billboarding: u32,
    enable_edge_correction: u32,
    lod_threshold: f32,
};

struct WindMaterialUniform {
    wind: Wind,
};

struct BindlessWindIndices {
    material: u32,
    noise_texture: u32,
    noise_texture_sampler: u32,
}


