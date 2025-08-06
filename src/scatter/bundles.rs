use crate::scatter::components::ScatterLayer;
use bevy::prelude::{Bundle, Name};
use std::borrow::Cow;

pub fn scatter_layer(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (Name::new(name), ScatterLayer::default())
}
