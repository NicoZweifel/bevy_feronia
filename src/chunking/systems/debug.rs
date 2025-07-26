use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::math::Vec3;
use bevy::prelude::*;

pub fn draw_aabbs(
    mut gizmos: Gizmos,
    q: Query<(&Aabb, &GlobalTransform), With<InstanceMaterialData>>,
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
    q: Query<(&Chunk, &GlobalTransform), With<InstanceMaterialData>>,
    chunk_cfg: Res<ChunkConfig>,
    debug_cfg: Res<ChunkDebugConfig>,
) {
    for (chunk, tf) in &q {
        gizmos.cuboid(
            Transform::from_translation(tf.translation())
                .with_rotation(tf.rotation())
                .with_scale(Vec3::splat(chunk.size as f32 * chunk_cfg.base_chunk_size)),
            debug_cfg.lod_colors[chunk.level as usize],
        );
    }
}
