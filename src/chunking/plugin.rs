use crate::backend::ScatterApp;
use crate::chunking::systems::*;
use crate::prelude::*;

use bevy_app::*;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_state::prelude::*;
use bevy_transform::TransformSystems;

pub struct ChunkPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkSet {
    Loading,
    Ready,
}

pub trait ChunkApp {
    fn configure_chunk_set<M: ScheduleLabel + Default>(&mut self) -> &mut App;
}

impl ChunkApp for App {
    fn configure_chunk_set<M: ScheduleLabel + Default>(&mut self) -> &mut App {
        self.configure_sets(
            M::default(),
            (
                ChunkSet::Loading
                    .run_if(in_state(ScatterState::Setup))
                    .run_if(not(in_state(HeightMapState::Ready))),
                ChunkSet::Ready
                    .run_if(in_state(ScatterState::Ready))
                    .run_if(in_state(HeightMapState::Ready)),
            ),
        )
    }
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_scatter_set::<PreUpdate>()
            .configure_scatter_set::<Update>()
            .configure_scatter_set::<PostUpdate>()
            .register_type::<Chunk>()
            .add_message::<SplitChunk>()
            .add_message::<MergeCheck>()
            .add_message::<MergeChunks>()
            .add_systems(
                PostUpdate,
                setup_chunks
                    .after(TransformSystems::Propagate)
                    .in_set(ChunkSet::Ready),
            )
            .add_systems(Update, update_root_lod_config)
            .add_systems(
                Update,
                (
                    (split, handle_split).chain(),
                    (merge_check, handle_merge_check).chain(),
                    (merge, handle_merge).chain(),
                    show_aabb_gizmos.run_if(|res: Option<Res<ChunkDebugConfig>>| {
                        res.is_some_and(|x| x.show_aabbs)
                    }),
                    hide_aabb_gizmos.run_if(|res: Option<Res<ChunkDebugConfig>>| {
                        res.is_none() || res.is_some_and(|x| !x.show_aabbs)
                    }),
                    draw_lod_ranges.run_if(|res: Option<Res<ChunkDebugConfig>>| {
                        res.is_some_and(|x| x.show_lod_ranges)
                    }),
                    draw_chunks.run_if(resource_exists::<ChunkDebugConfig>),
                )
                    .in_set(ChunkSet::Ready),
            );
    }
}
