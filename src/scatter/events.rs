use crate::prelude::{ScatterAsset, WindAffectable};
use crate::scatter::utils::Container;
use bevy::asset::Asset;
use bevy::pbr::Material;
use bevy::prelude::*;
use std::marker::PhantomData;
use std::slice::Iter;

#[derive(EntityEvent, Message, Component, Reflect)]
pub struct Scatter<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
> {
    pub entity: Entity,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> Scatter<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData,
        }
    }
}

#[derive(EntityEvent, Message, Component, Reflect)]
pub struct ScatterChunk<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub entity: Entity,
    pub scatter_layer: Entity,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<Tin, TOut> ScatterChunk<Tin, TOut>
where
    Tin: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, Tin, TOut> + Asset + Clone,
{
    pub fn new(entity: Entity, scatter_layer: Entity) -> Self {
        Self {
            entity,
            scatter_layer,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ScatterResult(pub Transform);

#[derive(EntityEvent, Message, Clone, Debug)]
pub struct ScatterResults<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub entity: Entity,
    pub data: Vec<ScatterResult>,
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> ScatterResults<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub fn get(&self) -> &Vec<ScatterResult> {
        &self.data
    }

    pub fn iter(&self) -> Iter<'_, ScatterResult> {
        self.data.iter()
    }

    pub fn new(
        entity: Entity,
        root: Entity,
        layer: Entity,
        chunk: Option<Entity>,
        data: Vec<ScatterResult>,
    ) -> Self {
        Self {
            entity,
            root,
            layer,
            chunk,
            data,
            _phantom: PhantomData,
        }
    }

    pub fn with_data(mut self, data: Vec<ScatterResult>) -> Self {
        self.data = data;
        self
    }
}

impl<TIn, TOut> From<&Container> for ScatterResults<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn from(value: &Container) -> Self {
        Self::new(
            value.layer_entity,
            value.root_entity,
            value.layer_entity,
            value.chunk_entity,
            vec![],
        )
    }
}
