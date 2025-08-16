#define_import_path bevy_feronia::bindings
#import bevy_feronia::wind::{Wind, BindlessWindIndices}

#ifdef BINDLESS
#import bevy_pbr::pbr_bindings::{material_array, material_indices}
#else
#import bevy_pbr::pbr_bindings::material
#endif

#ifdef BINDLESS
@group(3) @binding(100) var<storage> wind_indices:
    array<BindlessWindIndices>;
@group(3) @binding(101) var<storage> wind_material:
    array<Wind>;

#else

@group(3) @binding(50) var<uniform> wind: Wind;
@group(3) @binding(51) var noise_texture: texture_2d<f32>;
@group(3) @binding(52) var noise_texture_sampler: sampler;

#endif
