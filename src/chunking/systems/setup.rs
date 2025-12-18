use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;

use bevy_asset::Assets;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{IVec2, Vec3};
use bevy_transform::prelude::{GlobalTransform, Transform};

pub fn setup_chunks(
    mut cmd: Commands,
    images: Res<Assets<Image>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    q_root: Query<
        (
            Entity,
            &LodConfig,
            &ChunkSizeScalarConfig,
            &Aabb,
            &GlobalTransform,
            &ChunkRootSizeDim,
        ),
        (With<ChunkRoot>, Without<BaseChunkSize>),
    >,
) {
    let has_height_map = height_map_cfg.is_some();

    for (entity, lod_cfg, scalar_config, aabb, gtf, chunk_root_size) in &q_root {
        let height_sampler = height_map
            .as_deref()
            .and_then(|height_map_image| {
                height_map_cfg.as_deref().and_then(|cfg| {
                    images
                        .get(&height_map_image.0)
                        .map(|img| HeightMapSampler::Cpu(HeightMapCpuSampler::new(img, cfg, gtf)))
                })
            })
            .unwrap_or(HeightMapSampler::Default(DefaultSampler));

        let root_scale = gtf.compute_transform().scale;

        let local_aabb_size = Vec3::from(aabb.half_extents * 2.0);
        let center_offset = Vec3::from(aabb.half_extents);

        let world_aabb_size = local_aabb_size * root_scale;

        let top_lod = lod_cfg.get_max_lod();
        let top_scalar_config = scalar_config.get_scalar_config(top_lod);

        let top_chunk_size_local =
            (local_aabb_size / **chunk_root_size as f32).with_y(local_aabb_size.y);
        let top_chunk_size_world =
            (world_aabb_size / **chunk_root_size as f32).with_y(world_aabb_size.y);

        let base_chunk_size_world =
            BaseChunkSize(top_chunk_size_world / **top_scalar_config as f32);

        let chunk_lod_cfg =
            ChunkLodConfig::from_sources(lod_cfg, scalar_config, &base_chunk_size_world, 5.0);

        cmd.entity(entity)
            .insert((base_chunk_size_world, chunk_lod_cfg.clone()));

        let inverse_transform = gtf.affine().inverse();

        for z in 0..**chunk_root_size {
            for x in 0..**chunk_root_size {
                let grid_offset = Vec3::new(x as f32, 0., z as f32)
                    * top_chunk_size_local.with_y(0.)
                    + top_chunk_size_local.with_y(0.) / 2.
                    - center_offset.with_y(0.);

                let mut local_pos = Vec3::from(aabb.center) + grid_offset;

                if has_height_map {
                    let world_pos = gtf.transform_point(local_pos);

                    let height = height_sampler.sample(world_pos);

                    let target_world_pos = Vec3::new(world_pos.x, height, world_pos.z);

                    local_pos = inverse_transform.transform_point3(target_world_pos);
                }

                let child_lod_config =
                    chunk_lod_cfg.get_lod_config(chunk_lod_cfg.get_max_lod() - 1);

                cmd.spawn((
                    Chunk,
                    ChunkLevel(top_lod),
                    ChunkSize(**top_scalar_config),
                    Transform::from_translation(local_pos),
                    ChildOf(entity),
                    ChunkOf(entity),
                    SplitDistance(*child_lod_config),
                    ChunkCoord(IVec2::new(x as i32, z as i32)),
                ));
            }
        }
    }
}
