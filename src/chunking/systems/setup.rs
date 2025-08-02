use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;

pub fn setup_chunks(
    mut cmd: Commands,
    images: Res<Assets<Image>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    q_cfg: Query<
        (
            Entity,
            &ChunkLodConfig,
            &Aabb,
            &GlobalTransform,
            &ChunkRootSize,
        ),
        (With<ChunkRoot>, Without<BaseChunkSize>),
    >,
) {
    let height_map_image = match height_map {
        None => None,
        Some(x) => images.get(&x.0),
    };

    let height_sampler = height_map_cfg.map_or_else(
        || HeightMapSampler::Default(DefaultSampler),
        |cfg| {
            height_map_image.map_or_else(
                || HeightMapSampler::Default(DefaultSampler),
                |img| HeightMapSampler::CpuHeightMap(HeightMapCpuSampler::new(img, cfg.world_size)),
            )
        },
    );

    for (entity, cfg, aabb, gtf, chunk_root_size) in &q_cfg {
        let world_size = Vec3::from(aabb.half_extents * 2.0);

        let center_offset = Vec3::from(aabb.half_extents);

        let top_chunk_size = world_size / **chunk_root_size as f32;

        let top_lod_level = cfg.get_max_lod_level();
        let top_lod_config = cfg.get_lod_config(top_lod_level);

        let base_chunk_size = top_chunk_size / top_lod_config.chunk_size_scalar as f32;

        cmd.entity(entity)
            .insert((ChunkRoot::default(), BaseChunkSize(base_chunk_size)));

        for z in 0..**chunk_root_size {
            for x in 0..**chunk_root_size {
                let mut world_pos = gtf.translation()
                    + Vec3::new(x as f32, 0., z as f32) * top_chunk_size
                    + top_chunk_size / 2.;

                world_pos.y = height_sampler.sample(world_pos);

                let child_lod_config = cfg.get_lod_config(cfg.get_max_lod_level() - 1);

                cmd.spawn((
                    Chunk,
                    ChunkLevel(top_lod_level),
                    ChunkSize(top_lod_config.chunk_size_scalar),
                    Transform::from_translation(world_pos - Vec3::from(center_offset)),
                    ChildOf(entity),
                    ChunkOf(entity),
                    SplitDistance(child_lod_config.distance),
                ));
            }
        }
    }
}
