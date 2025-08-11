use crate::asset::systems::*;
use crate::prelude::*;
use bevy::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct ScatterAssetPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    _marker: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> Default for ScatterAssetPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone + Debug,
{
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<TOut, TIn> Plugin for ScatterAssetPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone + Debug,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (collect_assets::<TIn, TOut>,));
    }
}
