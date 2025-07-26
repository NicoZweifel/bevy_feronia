use crate::prelude::*;
use bevy::prelude::*;
use std::num::NonZeroU32;

pub fn split(
    cfg: Res<ChunkConfig>,
    q_center: Query<&GlobalTransform, With<ChunkCenter>>,
    q_chunk: Query<(Entity, &Chunk, &GlobalTransform), (Without<Children>, With<CanSplit>)>,
    mut ew_split: EventWriter<SplitChunk>,
) {
    let Ok(center) = q_center.single() else {
        return;
    };

    let center_translation = center.translation();

    for (entity, chunk, chunk_transform) in &q_chunk {
        if chunk.level == 0 {
            continue;
        };

        let dist = center_translation.distance(chunk_transform.translation());

        let child_chunk_data =
            cfg.calculate_child_data(NonZeroU32::new(chunk.level).unwrap(), chunk.size);

        let child_lod_config = cfg.get_lod_config(child_chunk_data.level);

        if dist < child_lod_config.distance {
            ew_split.write(SplitChunk(entity));
            continue;
        }
    }
}


pub fn handle_split(
    mut cmd: Commands,
    cfg: Res<ChunkConfig>,
    mut er_split: EventReader<SplitChunk>,
    q_chunk: Query<&Chunk>,
) {
    for e in er_split.read() {
        let parent_entity = e.get();

        let Ok(parent_chunk) = q_chunk.get(parent_entity) else {
            continue;
        };

        let child_chunk_data = cfg.calculate_child_data(
            NonZeroU32::new(parent_chunk.level).expect("Cannot split chunk at level 0!"),
            parent_chunk.size,
        );

        cmd.entity(parent_entity).with_children(|cmd| {
            for offset in &child_chunk_data.offsets {
                let child_entity = cmd.spawn(
                    (
                        Chunk {
                            level: child_chunk_data.level,
                            size: child_chunk_data.size,
                        },
                        Transform::from_translation(*offset),
                        GlobalTransform::from_translation(*offset),
                        Visibility::Visible,
                        ViewVisibility::default(),
                    )
                ).id();

                if child_chunk_data.level > 0 {
                    cmd.commands().entity(child_entity).insert(CanSplit);
                }

                if child_chunk_data.level < cfg.get_max_lod_level() {
                    cmd.commands().entity(child_entity).insert(CanMerge);
                }
            }
        });
    }
}
