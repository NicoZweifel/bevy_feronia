use crate::prelude::*;
use bevy::prelude::*;

pub fn setup_chunks(
    mut cmd: Commands,
    images: Res<Assets<Image>>,
    height_map: Option<Res<HeightMap>>,
    q_cfg: Query<(Entity, &ChunkConfig), Without<ChunkRoot>>,
) {
    for (entity, cfg) in &q_cfg {
        let height_sampler = cfg.get_height_map_sampler(&images, &height_map);

        let top_lod_level = cfg.get_max_lod_level();
        let top_lod_config = cfg.get_lod_config(top_lod_level);

        let total_world_size = cfg.get_total_world_size();
        let center_offset = total_world_size / 2.0;

        cmd.entity(entity).insert(ChunkRoot::default());

        for z in 0..cfg.world_size_in_chunks {
            for x in 0..cfg.world_size_in_chunks {
                let mut world_pos =
                    cfg.get_chunk_world_center(Vec3::new(x as f32, 0.0, z as f32), top_lod_level);

                if let Some(sampler) = &height_sampler {
                    world_pos.y =
                        sampler.sample(world_pos) + cfg.get_chunk_world_size(top_lod_level) / 2.0;
                };

                let child_lod_config = cfg.get_lod_config(cfg.get_max_lod_level() - 1);

                cmd.spawn((
                    Chunk,
                    ChunkLevel(top_lod_level),
                    ChunkSize(top_lod_config.chunk_size_scalar),
                    Transform::from_xyz(
                        world_pos.x - center_offset,
                        world_pos.y,
                        world_pos.z - center_offset,
                    ),
                    ChildOf(entity),
                    ChunkOf(entity),
                    CanSplit,
                    SplitDistance(child_lod_config.distance),
                ));
            }
        }
    }
}
