use bevy::prelude::{Commands, EventReader, GlobalTransform, Query, Res, Transform, ViewVisibility, Visibility};
use std::num::NonZeroU32;
use crate::chunking::components::Chunk;
use crate::chunking::events::{MergeChunks, SplitChunk};
use crate::chunking::prelude::ChunkConfig;

pub fn apply_splits(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    mut split_events: EventReader<SplitChunk>,
    chunk_query: Query<(&Chunk, &GlobalTransform)>,
) {
    for event in split_events.read() {
        let parent_entity = event.0;

        let Ok((parent_chunk, parent_transform)) = chunk_query.get(parent_entity) else {
            continue;
        };

        let child_chunk_data = config.calculate_child_data(
            NonZeroU32::new(parent_chunk.level)
                .expect("Cannot split chunk at level 0!"),
            parent_chunk.size,
        );

        commands.entity(parent_entity).despawn();

        commands.spawn_batch(child_chunk_data.offsets.map(|offset| {
            (
                Chunk {
                    level: child_chunk_data.level,
                    size: child_chunk_data.size,
                },
                Transform::from_translation(parent_transform.translation() + offset),
                GlobalTransform::from_translation(parent_transform.translation() + offset),
                Visibility::Visible,
                ViewVisibility::default(),
            )
        }));
    }
}

pub fn apply_merges(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    mut merge_events: EventReader<MergeChunks>,
) {
    for event in merge_events.read() {
        for sibling_entity in &event.siblings {
            commands.entity(*sibling_entity).despawn();
        }

        commands.spawn((
            Chunk {
                level: event.parent_level,
                size: config.get_size_scalar(event.parent_level),
            },
            Transform::from_translation(event.parent_center),
            GlobalTransform::from_translation(event.parent_center),
            Visibility::Visible,
            ViewVisibility::default(),
        ));
    }
}