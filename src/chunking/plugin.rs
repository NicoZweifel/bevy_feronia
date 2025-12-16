use crate::chunking::systems::*;
use crate::prelude::*;
use bevy_app::*;
use bevy_ecs::prelude::*;
use bevy_state::prelude::*;
use bevy_transform::TransformSystems;

pub struct ChunkPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkSet {
    Loading,
    Ready,
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .add_message::<SplitChunk>()
            .add_message::<MergeCheck>()
            .add_message::<MergeChunks>()
            .configure_sets(
                Update,
                (
                    ChunkSet::Loading
                        .run_if(in_state(ScatterState::Setup))
                        .run_if(not(in_state(HeightMapState::Ready))),
                    ChunkSet::Ready
                        .run_if(in_state(ScatterState::Ready))
                        .run_if(in_state(HeightMapState::Ready)),
                ),
            )
            .configure_sets(
                PostUpdate,
                (
                    ChunkSet::Loading
                        .run_if(in_state(ScatterState::Setup))
                        .run_if(not(in_state(HeightMapState::Ready))),
                    ChunkSet::Ready
                        .run_if(in_state(ScatterState::Ready))
                        .run_if(in_state(HeightMapState::Ready)),
                ),
            )
            .add_systems(Update, update_root_lod_config)
            .add_systems(
                PostUpdate,
                setup_chunks
                    .after(TransformSystems::Propagate)
                    .in_set(ChunkSet::Ready),
            )
            .add_systems(
                Update,
                (
                    update_chunk_height.run_if(resource_exists_and_changed::<HeightMap>),
                    (split, handle_split).chain(),
                    (merge_check, handle_merge_check).chain(),
                    (merge, handle_merge).chain(),
                    (draw_aabbs, draw_chunks, draw_lod_ranges)
                        .run_if(resource_exists::<ChunkDebugConfig>),
                )
                    .in_set(ChunkSet::Ready),
            );
    }
}
