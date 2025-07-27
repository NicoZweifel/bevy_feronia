use crate::chunking::prelude::*;
use bevy::prelude::*;

pub fn setup_chunks(mut cmd: Commands, cfg: Res<ChunkConfig>) {
    let top_lod_level = cfg.get_max_lod_level();
    let top_lod_config = cfg.get_lod_config(top_lod_level);

    let total_world_size = cfg.get_total_world_size();
    let center_offset = total_world_size / 2.0;

    for z in 0..cfg.world_size_in_chunks {
        for x in 0..cfg.world_size_in_chunks {
            // TODO height data
            let world_pos =
                cfg.get_chunk_world_center(IVec3::new(x as i32, 0, z as i32), top_lod_level);

            cmd.spawn((
                Chunk {
                    level: top_lod_level,
                    size: top_lod_config.chunk_size_scalar,
                },
                Transform::from_xyz(
                    world_pos.x - center_offset,
                    0.0,
                    world_pos.z - center_offset,
                ),
                GlobalTransform::default(),
                Visibility::Visible,
                ViewVisibility::default(),
                CanSplit,
            ));
        }
    }
}
