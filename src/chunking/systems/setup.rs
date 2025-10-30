use crate::prelude::*;
use crate::scatter::utils::get_height_map_sampler;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

pub fn setup_chunks(
    mut cmd: Commands,
    images: Res<Assets<Image>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    q_root: Query<
        (
            Entity,
            &ChunkLodConfig,
            &ChunkSizeScalarConfig,
            &Aabb,
            &GlobalTransform,
            &ChunkRootSizeDim,
        ),
        (With<ChunkRoot>, Without<BaseChunkSize>),
    >,
) {
    let height_sampler = get_height_map_sampler(&images, height_map_cfg, height_map);

    for (entity, chunk_cfg, scalar_config, aabb, gtf, chunk_root_size) in &q_root {
        let world_size = Vec3::from(aabb.half_extents * 2.0);

        let center_offset = Vec3::from(aabb.half_extents);

        let top_chunk_size = (world_size / **chunk_root_size as f32).with_y(world_size.y);

        let top_LOD = chunk_cfg.get_max_lod();
        let top_scalar_config = scalar_config.get_scalar_config(top_lod);

        let base_chunk_size = top_chunk_size / **top_scalar_config as f32;

        cmd.entity(entity)
            .insert((ChunkRoot::default(), BaseChunkSize(base_chunk_size)));

        for z in 0..**chunk_root_size {
            for x in 0..**chunk_root_size {
                let mut world_pos = gtf.translation()
                    + Vec3::new(x as f32, 0., z as f32) * top_chunk_size.with_y(0.)
                    + top_chunk_size.with_y(0.) / 2.
                    - center_offset.with_y(0.);

                world_pos.y = height_sampler.sample(world_pos);

                let child_lod_config = chunk_cfg.get_lod_config(chunk_cfg.get_max_lod() - 1);

                cmd.spawn((
                    Chunk,
                    ChunkLevel(top_lod),
                    ChunkSize(**top_scalar_config),
                    Transform::from_translation(world_pos),
                    ChildOf(entity),
                    ChunkOf(entity),
                    SplitDistance(**child_lod_config),
                    ChunkCoord(IVec2::new(x as i32, z as i32)),
                ));
            }
        }
    }
}
