use crate::prelude::*;
use bevy::prelude::*;
use std::{borrow::Cow, fmt::Debug};

pub fn scatter_layer<TIn, TOut>(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone + Debug,
{
    (
        Name::new(name),
        ScatterLayer::default(),
        ScatterLayerType::<TIn, TOut>::default(),
    )
}
