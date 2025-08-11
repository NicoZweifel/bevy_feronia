use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::math::Vec3;
use bevy::prelude::*;

pub fn draw_aabbs(
    mut gizmos: Gizmos,
    q: Query<(&Aabb, &GlobalTransform), With<Chunk>>,
    cfg: Res<ChunkDebugConfig>,
) {
    for (aabb, tf) in &q {
        gizmos.cuboid(
            Transform::from_translation(tf.transform_point(aabb.center.into()))
                .with_rotation(tf.rotation())
                .with_scale((aabb.half_extents * 2.0).into()),
            cfg.aabb_color,
        );
    }
}

pub fn draw_chunks(
    mut gizmos: Gizmos,
    q: Query<(&ChunkSize, &ChunkLevel, &GlobalTransform, &ChunkOf), With<Chunk>>,
    q_chunk_config: Query<&BaseChunkSize, With<LodConfig>>,
    debug_cfg: Res<ChunkDebugConfig>,
) {
    for (chunk_size, chunk_level, tf, root_chunk) in &q {
        let base_chunk_size = q_chunk_config.get(**root_chunk).unwrap();
        gizmos.cuboid(
            Transform::from_translation(tf.translation())
                .with_rotation(tf.rotation())
                .with_scale(Vec3::splat(**chunk_size as f32) * **base_chunk_size),
            debug_cfg.lod_colors[**chunk_level as usize],
        );
    }
}
