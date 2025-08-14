use crate::prelude::*;
use crate::scatter::observers::*;
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
            .add_event::<Scatter<TIn, TOut>>()
            .init_asset::<ScatterAsset<TIn>>()
            .init_asset::<ScatterAsset<TOut>>()
            .add_event::<ScatterChunk<TIn, TOut>>()
            .add_event::<ScatterResults<TIn, TOut>>()
            .add_observer(on_add_scatter_root::<TIn, TOut>)
            .add_observer(on_add_scatter_layer::<TIn, TOut>)
            .add_observer(on_chunk_add::<TIn, TOut>)
            .add_systems(Update, chunk_init_scatter::<TIn, TOut>.in_set(ChunkSet::Ready));
    }
}

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChunkPlugin, HeightMapPlugin, WindPlugin))
            .init_state::<ScatterState>()
            .add_observer(on_add_scatter_item)
            .add_systems(
                PostUpdate,
                setup_root_aabb.run_if(in_state(ScatterState::Setup)),
            )
            .add_systems(
                Update,
                (
                    transition_to_ready_state.run_if(in_state(ScatterState::Setup)),
                    check_unprocessed_layers,
                ),
            );
    }
}
