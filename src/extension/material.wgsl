#define_import_path bevy_feronia::extension::material

#import bevy_feronia::wind::Wind

struct ExtendedMaterial {
    wind: Wind,
    sss_scale: f32,
    sss_intensity: f32,
};
