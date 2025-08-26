#define_import_path bevy_feronia::noise
#import bevy_feronia::types::{SampledNoise, InstanceInfo}
#import bevy_feronia::wind::{Wind}
#import bevy_pbr::utils::rand_f
#import bevy_pbr::mesh_bindings::mesh

#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#endif

#ifdef BINDLESS
#import bevy_feronia::bindings::{wind_indices, wind_material}
#else
#import bevy_feronia::bindings::{wind, noise_texture, noise_texture_sampler}
#endif

fn sample_noise(instance: InstanceInfo) -> SampledNoise {
    var noise: SampledNoise;

#ifdef BINDLESS
    let slot = mesh[instance.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let wind =  wind_material[wind_indices[slot].material];
    let noise_texture =   bindless_textures_2d[wind_indices[slot].noise_texture];
    let noise_texture_sampler =  bindless_samplers_filtering[wind_indices[slot].noise_texture_sampler];
#endif

    let base_uv = instance.instance_position.xz * wind.noise_scale;
    let uv_scroll_offset = instance.wrapped_time * wind.scroll_speed * wind.direction;
    let tex_coord = base_uv + uv_scroll_offset;

    let packed_noise = textureSampleLevel(noise_texture, noise_texture_sampler, tex_coord, 0.0);

    noise.macro_noise = packed_noise.r;
    noise.micro_noise = 0.0;
    noise.phase_noise = vec2<f32>(0.0);

#ifdef WIND_HIGH_QUALITY
    noise.micro_noise = packed_noise.g;

    let seed_x = bitcast<u32>(instance.instance_position.x);
    let seed_y = bitcast<u32>(instance.instance_position.y);
    let seed_z = bitcast<u32>(instance.instance_position.z);
    var seed = seed_x ^ seed_y ^ seed_z;

    noise.phase_noise.x = rand_f(&seed);
    noise.phase_noise.y = rand_f(&seed);
#endif

    return noise;
}



