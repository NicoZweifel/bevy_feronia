#define_import_path bevy_feronia::noise

#import bevy_feronia::types::{SampledNoise, InstanceInfo}
#import bevy_feronia::wind::{Wind}

#import bevy_pbr::utils::rand_f
#import bevy_pbr::mesh_bindings::mesh

#ifdef WIND_AFFECTED
#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#import bevy_feronia::wind::bindings::wind_affected_material_indices
#else
#import bevy_feronia::wind::bindings::{noise_texture, noise_texture_sampler}
#endif // BINDLESS
#endif // WIND_AFFECTED

fn sample_noise(instance: InstanceInfo, wind: Wind, local_vertex_pos: vec3<f32>) -> SampledNoise {
    var noise: SampledNoise;

#ifdef WIND_AFFECTED
#ifdef BINDLESS
    let slot = mesh[instance.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let noise_texture =  bindless_textures_2d[wind_affected_material_indices[slot].noise_texture];
    let noise_texture_sampler = bindless_samplers_filtering[wind_affected_material_indices[slot].noise_texture_sampler];
#endif // BINDLESS

    // Need to use local_vertex_pos here.
    // Sampling the same noise for neighbors, breaks motion vectors / DLSS / TAA
    let world_pos = (instance.world_from_local * vec4<f32>(local_vertex_pos, 1.0)).xyz;

    let base_uv = world_pos.xz * wind.noise_scale;
    let uv_scroll_offset = instance.wrapped_time * wind.scroll_speed * wind.direction;
    let tex_coord = base_uv + uv_scroll_offset;

#ifdef WIND_LOW_QUALITY
    noise.macro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, tex_coord, 0.0).r;
    noise.micro_noise = 0.0;
    noise.phase_noise = vec2<f32>(0.0);
#else // NOT WIND_LOW_QUALITY
    let packed_noise = textureSampleLevel(noise_texture, noise_texture_sampler, tex_coord, 0.0);

    noise.macro_noise = packed_noise.r;
    noise.micro_noise = packed_noise.g;
    noise.phase_noise = unpack2x16unorm(instance.seed);
#endif // WIND_LOW_QUALITY

#else // NOT WIND_AFFECTED
    noise.macro_noise = 0.0;
    noise.micro_noise = 0.0;
    noise.phase_noise = vec2<f32>(0.0);
#endif // WIND_AFFECTED

    return noise;
}



