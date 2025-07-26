use crate::chunking::prelude::*;
use crate::chunking::systems::debug::{draw_aabbs, draw_chunks};
use crate::chunking::systems::prelude::*;
use bevy::prelude::*;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .init_resource::<ChunkConfig>()
            .add_event::<SplitChunk>()
            .add_event::<MergeCheck>()
            .add_event::<MergeChunks>()
            .add_systems(Startup, setup_chunks)
            .add_systems(
                Update,
                (
                    (split, handle_split).chain(),
                    (merge, handle_merge_check, handle_merge).chain(),
                    (draw_aabbs, draw_chunks).run_if(resource_exists::<ChunkDebugConfig>),
                ),
            );
    }
}
