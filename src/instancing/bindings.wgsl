#define_import_path bevy_feronia::instancing::bindings

#import bevy_feronia::instancing::material::InstancedMaterial

@group(3) @binding(50) var<uniform> material_uniforms: InstancedMaterial;


