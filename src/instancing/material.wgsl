#define_import_path bevy_feronia::instancing::material

#import bevy_feronia::wind::Wind

struct InstancedMaterial {
    wind: Wind,
    top_color: vec4<f32>,
    bottom_color: vec4<f32>,
    tint_factor: f32,
    gradient_start: f32,
    gradient_end: f32,
    static_bend_strength: f32,
    curve_factor: f32,
    translucency: f32,
    specular_strength: f32,
    specular_power: f32,
};

