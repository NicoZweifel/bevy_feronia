use crate::prelude::*;
use bevy_camera::primitives::Aabb;
use bevy_color::palettes::css::{PURPLE, RED};
use bevy_ecs::prelude::*;
use bevy_gizmos::prelude::Gizmos;
use bevy_math::*;
use bevy_transform::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

pub fn draw_aabbs(
    mut gizmos: Gizmos,
    q: Query<(&Aabb, &GlobalTransform)>,
    cfg: Res<ChunkDebugConfig>,
) {
    for (aabb, tf) in &q {
        gizmos.cube(
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
    q_chunk_config: Query<&BaseChunkSize, With<ChunkLodConfig>>,
    debug_cfg: Res<ChunkDebugConfig>,
) {
    for (chunk_size, chunk_level, tf, root_chunk) in &q {
        let base_chunk_size = q_chunk_config.get(**root_chunk).unwrap();
        gizmos.cube(
            Transform::from_translation(tf.translation())
                .with_rotation(tf.rotation())
                .with_scale(**chunk_size as f32 * **base_chunk_size),
            debug_cfg.lod_colors[**chunk_level as usize],
        );
    }
}

pub fn draw_lod_ranges(
    mut gizmos: Gizmos,
    q_center: Query<&GlobalTransform, With<Center>>,
    q_root_config: Query<(&LodConfig, &ChunkLodConfig)>,
) {
    let Ok(center_tf) = q_center.single() else {
        return;
    };
    let center_pos = center_tf.translation();

    let Ok((lod_config, chunk_lod_config)) = q_root_config.single() else {
        return;
    };

    for (i, dist) in lod_config
        .distance
        .iter()
        .map(|d| **d)
        .take_while(|d| *d < f32::MAX)
        .enumerate()
    {
        gizmos.sphere(
            Isometry3d {
                translation: center_pos.into(),
                rotation: Quat::from_rotation_x(FRAC_PI_2)
                    * Quat::from_rotation_z(FRAC_PI_8)
                    * Quat::from_rotation_z(FRAC_PI_8 * 0.2 * i as f32),
            },
            dist,
            PURPLE,
        );
    }

    for (i, dist) in chunk_lod_config
        .0
        .iter()
        .map(|d| **d)
        .take_while(|d| *d < f32::MAX)
        .enumerate()
    {
        gizmos.sphere(
            Isometry3d {
                translation: center_pos.into(),
                rotation: Quat::from_rotation_x(FRAC_PI_2)
                    * Quat::from_rotation_z(FRAC_PI_8 * 0.2 * i as f32),
            },
            dist,
            RED,
        );
    }
}
