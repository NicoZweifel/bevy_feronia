use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::IntoScheduleConfigs;
use crate::chunking::prelude::*;
use crate::chunking::systems::prelude::*;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .init_resource::<ChunkConfig>()
            .add_event::<SplitChunk>()
            .add_event::<MergeChunks>()
            .add_systems(Startup, setup_chunks)
            .add_systems(
                Update,
                (
                    update_chunk_lods,
                    (apply_splits, apply_merges).after(update_chunk_lods),
                ),
            );
    }
}