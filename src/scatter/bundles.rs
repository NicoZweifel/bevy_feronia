use crate::prelude::*;
use bevy_ecs::prelude::*;
use std::{borrow::Cow, fmt::Debug};

pub fn scatter_layer<T>(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
    T: ScatterMaterial + Debug,
{
    (
        Name::new(name),
        ScatterLayer::default(),
        ScatterLayerType::<T>::default(),
    )
}
