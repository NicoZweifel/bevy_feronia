use crate::prelude::*;
use bevy::prelude::*;
use std::num::NonZeroU32;

pub fn split(
    q_center: Query<&GlobalTransform, With<ChunkCenter>>,
    q_chunk: Query<(Entity, &GlobalTransform, &SplitDistance), (With<CanSplit>, With<Chunk>)>,
    mut ew_split: EventWriter<SplitChunk>,
) {
    let Ok(center) = q_center.single() else {
        warn!(
            "Couldn't get ChunkCenter for split! Did you forgot to add it to your Camera or Player entity?"
        );
        return;
    };

    let center = center.translation();

    for (entity, chunk_transform, split_distance) in &q_chunk {
        let distance = center.distance(chunk_transform.translation());
        if distance < **split_distance {
            ew_split.write(SplitChunk(entity));
        }
    }
}

pub fn handle_split(
    mut cmd: Commands,
    mut er_split: EventReader<SplitChunk>,
    q_chunk: Query<(&ChunkLevel, &ChunkSize, &ChunkOf), (With<CanSplit>, With<Chunk>)>,
    q_chunk_config: Query<(&BaseChunkSize, &ChunkLodConfig, &ChunkSizeScalarConfig)>,
) {
    for e in er_split.read() {
        let parent_entity = e.get();
        debug!("Splitting Chunk: {parent_entity}");

        let Ok((parent_chunk_level, parent_chunk_size, root_chunk)) = q_chunk.get(parent_entity)
        else {
            warn!("Couldn't get Chunk for split: {parent_entity}");
            continue;
        };

        let (base_chunk_size, lod_cfg, scalar_cfg) = q_chunk_config.get(**root_chunk).unwrap();

        let mut child_chunk_data = lod_cfg.calculate_child_data(
            NonZeroU32::new(**parent_chunk_level).expect("Cannot split chunk at level 0!"),
            **parent_chunk_size,
            **base_chunk_size,
            scalar_cfg
                .0
                .get(**parent_chunk_level as usize - 1)
                .map_or(*ChunkSizeScalar::default(), |x| **x),
        );

        for offset in &mut child_chunk_data.offsets {
            let child_entity = cmd
                .spawn((
                    Chunk,
                    ChunkSize(child_chunk_data.size),
                    ChunkLevel(child_chunk_data.level),
                    Transform::from_translation(*offset),
                    ChunkOf(**root_chunk),
                    ChildOf(parent_entity),
                ))
                .id();

            if child_chunk_data.level > 0 {
                let child_lod_config = lod_cfg.get_lod_config(child_chunk_data.level - 1);
                cmd.entity(child_entity)
                    .insert(SplitDistance(**child_lod_config));
            }

            if child_chunk_data.level < lod_cfg.get_max_lod_level() {
                cmd.entity(child_entity).insert(MergeDistance(
                    **lod_cfg.get_lod_config(child_chunk_data.level),
                ));
            }

            cmd.entity(parent_entity).remove::<CanSplit>();
        }
    }
}
