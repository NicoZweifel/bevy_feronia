use crate::chunking::systems::prelude::*;
use crate::prelude::*;
use bevy::prelude::*;

pub struct ChunkPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum ChunkSet {
    Loading,
    Ready,
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .add_event::<SplitChunk>()
            .add_event::<MergeCheck>()
            .add_event::<MergeChunks>()
            .init_state::<HeightMapState>()
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
            .add_observer(on_add_chunk)
            .add_systems(Update, check_height_map.in_set(ChunkSet::Loading))
            .add_systems(
                Update,
                (
                    init,
                    setup_chunks,
                    update_chunk_height.run_if(resource_exists_and_changed::<HeightMap>),
                    (split, handle_split).chain(),
                    (merge, handle_merge_check, handle_merge).chain(),
                    (draw_aabbs, draw_chunks).run_if(resource_exists::<ChunkDebugConfig>),
                )
                    .in_set(ChunkSet::Ready),
            );
    }
}

fn check_height_map(
    res: Option<Res<HeightMapConfig>>,
    mut state: ResMut<NextState<HeightMapState>>,
) {
    if res.is_none() {
        state.set(HeightMapState::Ready);
    }
}
