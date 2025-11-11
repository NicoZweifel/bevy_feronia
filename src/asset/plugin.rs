use crate::asset::systems::*;
use crate::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct ScatterAssetsPlugin<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    _marker: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> ScatterAssetsPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<TOut, TIn> Default for ScatterAssetsPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TOut, TIn> Plugin for ScatterAssetsPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                queue_asset_creation_requests::<TOut, TIn>,
                process_distinct_material_requests::<TOut, TIn>
                    .after(queue_asset_creation_requests::<TOut, TIn>),
            )
                .run_if(in_state(ScatterState::Ready)),
        );
    }
}
