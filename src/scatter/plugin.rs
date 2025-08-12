use crate::prelude::*;
use crate::scatter::observers::*;
use crate::scatter::systems::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct ScatterPlugin<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
> {
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone>
    ScatterPlugin<TIn, TOut>
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone> Default
    for ScatterPlugin<TIn, TOut>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone> Plugin
    for ScatterPlugin<TIn, TOut>
{
    fn build(&self, app: &mut App) {
        app.add_event::<Scatter<TIn, TOut>>()
            .init_state::<ScatterState>()
            .init_asset::<ScatterAsset<TIn>>()
            .init_asset::<ScatterAsset<TOut>>()
            .add_event::<ScatterChunk<TIn, TOut>>()
            .add_event::<ScatterResults<TIn, TOut>>()
            .add_event::<SplitChunk>()
            .add_observer(on_add_scatter_root::<TIn, TOut>)
            .add_observer(on_add_scatter_layer::<TIn, TOut>)
            .add_observer(on_add_scatter_item)
            .add_systems(
                PostUpdate,
                setup_root_aabb.run_if(in_state(ScatterState::Setup)),
            )
            .add_systems(
                Update,
                (
                    transition_to_ready_state.run_if(in_state(ScatterState::Setup)),
                    init::<TIn, TOut>.in_set(ChunkSet::Ready),
                ),
            );
    }
}
