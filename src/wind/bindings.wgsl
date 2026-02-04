#define_import_path bevy_feronia::wind::bindings
#import bevy_feronia::wind::{BindlessWindIndices}

#ifdef BINDLESS
#import bevy_pbr::pbr_bindings::{material_array, material_indices}
#else
#import bevy_pbr::pbr_bindings::material
#endif

#ifdef BINDLESS
@group(3) @binding(100) var<storage> wind_affected_material_indices:
    array<BindlessWindIndices>;
#else
@group(3) @binding(51) var noise_texture: texture_2d<f32>;
@group(3) @binding(52) var noise_texture_sampler: sampler;
#endif
