use crate::core::Sampler;
use crate::prelude::*;
use bevy::asset::Assets;
use bevy::image::Image;
use bevy::prelude::*;

pub fn update_chunk_height(
    images: Res<Assets<Image>>,
    mut q_chunk: Query<(&mut Transform, &ChunkOf), With<ChunkRoot>>,
    q_cfg: Query<&ChunkLodConfig>,
    height_map: Res<HeightMap>,
    height_map_config: Res<HeightMapConfig>,
) {
    let height_map = Some(height_map);

    for (mut tf, root_chunk) in &mut q_chunk {
        let height_sampler = q_cfg.get(**root_chunk).unwrap().get_height_map_sampler(
            &images,
            &height_map,
            height_map_config.world_size,
        );

        if let Some(sampler) = &height_sampler {
            tf.translation.y = sampler.sample(tf.translation);
        };
    }
}
