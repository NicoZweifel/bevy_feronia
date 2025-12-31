#define_import_path bevy_feronia::extension::bindings

#import bevy_feronia::extension::material::ExtendedMaterial
#import bevy_feronia::wind::{Wind, BindlessWindIndices}

#ifdef BINDLESS
#import bevy_pbr::pbr_bindings::{material_array, material_indices}
#else
#import bevy_pbr::pbr_bindings::material
#endif

#ifdef BINDLESS
@group(3) @binding(101) var<storage> material_uniforms_array:
    array<ExtendedMaterial>;
#else
@group(3) @binding(50) var<uniform> material_uniforms: ExtendedMaterial;
#endif
