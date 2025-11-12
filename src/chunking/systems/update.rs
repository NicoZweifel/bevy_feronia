use crate::core::Sampler;
use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::prelude::*;

pub fn update_chunk_height(
    images: Res<Assets<Image>>,
    mut q_chunk: Query<&mut Transform, (With<ChunkRoot>, With<ChunkOf>)>,
    height_map: Res<HeightMap>,
    height_map_config: Res<HeightMapConfig>,
) {
    let height_sampler = images
        .get(&height_map.0)
        .map(|img| HeightMapCpuSampler::new(img, height_map_config.into_inner()));

    let Some(sampler) = height_sampler else {
        return;
    };

    for mut tf in &mut q_chunk {
        tf.translation.y = sampler.sample(tf.translation);
    }
}
