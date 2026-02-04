#define_import_path bevy_feronia::instancing::material

#import bevy_feronia::wind::Wind

struct InstancedMaterial {
    current: Wind,
    previous: Wind,
    top_color: vec4<f32>,
    bottom_color: vec4<f32>,
    tint_factor: f32,
    gradient_start: f32,
    gradient_end: f32,
    curve_factor: f32,
    translucency: f32,
    specular_strength: f32,
    specular_power: f32,
    diffuse_scaling: f32,
    light_intensity: f32,
    ambient_light_intensity: f32,
    edge_correction_factor: f32,
    metallic: f32,
    roughness: f32,
    reflectance: f32,
    static_bend_strength: f32,
    static_bend_direction: vec2<f32>,
    static_bend_control_point: vec2<f32>,
    static_bend_min_max: vec2<f32>,
    subsurface_scattering_scale: f32,
    subsurface_scattering_intensity: f32,
};

