use bevy::prelude::{BufferedEvent, Component, Deref, Entity, EntityEvent, Reflect, Transform};
use std::marker::PhantomData;
use std::slice::Iter;

#[derive(EntityEvent, BufferedEvent, Component, Reflect, Deref, Default)]
pub struct Scatter<T> {
    _phantom: PhantomData<T>,
}

#[derive(Clone, Debug)]
pub struct ScatterResult {
    pub global_transform: Transform,
    pub layer: Entity,
}

#[derive(EntityEvent, BufferedEvent, Clone, Debug)]
pub struct ScatterResults {
    pub results: Vec<ScatterResult>,
    pub chunk: Option<Entity>,
}

impl ScatterResults {
    pub fn get(&self) -> &Vec<ScatterResult> {
        &self.results
    }

    pub fn iter(&self) -> Iter<'_, ScatterResult> {
        self.results.iter()
    }
}
