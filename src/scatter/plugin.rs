use crate::prelude::*;
use crate::scatter::observers::*;
use crate::scatter::systems::handle_scatter_requests::{
    handle_finished_scatter_tasks, handle_scatter_requests,
};
use crate::scatter::systems::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct ScatterAssetPlugin<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
> {
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone>
    ScatterAssetPlugin<TIn, TOut>
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone> Default
    for ScatterAssetPlugin<TIn, TOut>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone> Plugin
    for ScatterAssetPlugin<TIn, TOut>
{
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ScatterPlugin>() {
            app.add_plugins(ScatterPlugin);
        }

        app.add_plugins(ScatterAssetsPlugin::<TIn, TOut>::new())
            .add_message::<Scatter<TIn, TOut>>()
            .init_asset::<ScatterAsset<TIn>>()
            .init_asset::<ScatterAsset<TOut>>()
            .add_message::<ScatterChunk<TIn, TOut>>()
            .add_message::<ScatterResults<TIn, TOut>>()
            .add_observer(on_add_scatter_root::<TIn, TOut>)
            .add_observer(on_add_scatter_layer::<TIn, TOut>)
            .add_observer(on_chunk_add::<TIn, TOut>)
            .add_systems(
                Update,
                chunk_init_scatter::<TIn, TOut>.in_set(ChunkSet::Ready),
            )
            .add_systems(
                Update,
                (
                    handle_scatter_requests::<TIn, TOut>,
                    handle_finished_scatter_tasks::<TIn, TOut>,
                ),
            );
    }
}

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChunkPlugin, HeightMapPlugin, WindPlugin))
            .init_state::<ScatterState>()
            .init_resource::<WorldSeed>()
            .add_observer(on_add_scatter_item)
            .add_systems(
                PostUpdate,
                setup_root_aabb.run_if(in_state(ScatterState::Setup)),
            )
            .add_systems(
                Update,
                (transition_to_ready_state,).run_if(in_state(ScatterState::Setup)),
            )
            .add_systems(
                Update,
                (check_unprocessed_layers, check_unprocessed_items)
                    .run_if(in_state(ScatterState::Ready)),
            );
    }
}
