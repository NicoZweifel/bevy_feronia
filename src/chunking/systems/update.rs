use bevy::prelude::{Entity, EventWriter, GlobalTransform, Query, Res, With};
use bevy::platform::collections::HashMap;
use bevy::math::IVec3;
use std::num::NonZeroU32;
use crate::chunking::components::{Chunk, ChunkCenter};
use crate::chunking::events::{MergeChunks, SplitChunk};
use crate::chunking::prelude::ChunkConfig;

pub fn update_chunk_lods(
    config: Res<ChunkConfig>,
    center_query: Query<&GlobalTransform, With<ChunkCenter>>,
    chunk_query: Query<(Entity, &Chunk, &GlobalTransform)>,
    mut ew_split: EventWriter<SplitChunk>,
    mut ew_merge: EventWriter<MergeChunks>,
) {
    let Ok(center) = center_query.single() else {
        return;
    };

    let center_translation = center.translation();
    let max_lod_level = (config.lods.len() - 1) as u32;
    let mut potential_parents: HashMap<IVec3, Vec<Entity>> = HashMap::new();

    for (entity, chunk, chunk_transform) in &chunk_query {
        if chunk.level > 0 {
            let dist = center_translation.distance(chunk_transform.translation());

            let child_chunk_data =
                config.calculate_child_data(NonZeroU32::new(chunk.level).unwrap(), chunk.size);
            let child_lod_config = config.get_lod_config(child_chunk_data.level);

            if dist < child_lod_config.distance {
                ew_split.write(SplitChunk(entity));
            }

            continue;
        }

        if chunk.level < max_lod_level {
            let parent_level = chunk.level + 1;
            let parent_world_size = config.get_chunk_world_size(parent_level);
            let parent_grid_coord = (chunk_transform.translation() / parent_world_size)
                .floor()
                .as_ivec3();

            potential_parents
                .entry(parent_grid_coord)
                .or_default()
                .push(entity);
        }
    }

    for (parent_grid_coord, siblings) in potential_parents {
        if siblings.len() < 4 {
            continue;
        };

        let chunk_level = chunk_query.get(siblings[0]).unwrap().1.level;
        let merge_dist = config.lods[chunk_level as usize].distance;

        let parent_level = chunk_level + 1;
        let parent_center = config.get_center(parent_grid_coord, parent_level);

        if center_translation.distance(parent_center) <= merge_dist {
            continue;
        }

        ew_merge.write(MergeChunks {
            siblings,
            parent_center,
            parent_level,
        });
    }
}