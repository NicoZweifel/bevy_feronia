use crate::asset::systems::*;
use crate::prelude::*;
use bevy::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct ScatterAssetsPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    _marker: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> ScatterAssetsPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<TIn, TOut> Default for ScatterAssetsPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TOut, TIn> Plugin for ScatterAssetsPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (collect_assets::<TIn, TOut>,));
    }
}
