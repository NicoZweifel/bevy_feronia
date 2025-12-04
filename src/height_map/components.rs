use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component, Clone)]
pub struct MapHeight;

#[derive(Component, Debug)]
pub struct HeightMapped;

#[derive(Component, Debug)]
pub struct HeightMapCamera;

#[derive(Component, Debug)]
pub struct HeightMapGhost;
