use crate::prelude::*;
use bevy::prelude::*;
use std::{borrow::Cow, fmt::Debug};

pub fn scatter_layer<TOut, TIn>(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
    TIn: Material,
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone + Debug,
{
    (
        Name::new(name),
        ScatterLayer::default(),
        ScatterLayerType::<TOut, TIn>::default(),
    )
}
