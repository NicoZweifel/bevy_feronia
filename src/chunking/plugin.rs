use crate::chunking::systems::debug::*;
use crate::chunking::systems::prelude::*;
use crate::prelude::*;
use bevy::prelude::*;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .add_event::<SplitChunk>()
            .add_event::<MergeCheck>()
            .add_event::<MergeChunks>()
            .add_systems(
                Update,
                (
                    setup_chunks,
                    update_chunk_height.run_if(resource_exists_and_changed::<HeightMap>),
                    (split, handle_split).chain(),
                    (merge, handle_merge_check, handle_merge).chain(),
                    (draw_aabbs, draw_chunks).run_if(resource_exists::<ChunkDebugConfig>),
                ),
            );
    }
}
