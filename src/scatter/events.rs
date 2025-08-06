use bevy::prelude::{BufferedEvent, Component, Deref, Entity, EntityEvent, Reflect, Transform};
use std::marker::PhantomData;
use std::slice::Iter;

#[derive(EntityEvent, BufferedEvent, Component, Reflect, Default)]
pub struct Scatter;

#[derive(EntityEvent, BufferedEvent, Component, Reflect)]
pub struct ScatterChunk {
    pub scatter_layer: Entity,
}

#[derive(Clone, Debug, Deref)]
pub struct ScatterResult(pub Transform);

#[derive(EntityEvent, BufferedEvent, Clone, Debug)]
pub struct ScatterResults {
    pub data: Vec<ScatterResult>,
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
}

impl ScatterResults {
    pub fn get(&self) -> &Vec<ScatterResult> {
        &self.data
    }

    pub fn iter(&self) -> Iter<'_, ScatterResult> {
        self.data.iter()
    }
}
