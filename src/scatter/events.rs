use crate::prelude::ScatterMaterial;
use crate::scatter::utils::Container;
use bevy::asset::Asset;
use bevy::pbr::Material;
use bevy::prelude::*;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::slice::Iter;

#[derive(EntityEvent, Message, Component, Reflect)]
pub struct Scatter<
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone = StandardMaterial,
    TIn: Material = StandardMaterial,
> {
    pub entity: Entity,
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Scatter<TOut, TIn>
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData,
        }
    }
}

#[derive(EntityEvent, Message, Component, Reflect)]
pub struct ScatterChunk<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    pub entity: Entity,
    pub scatter_layer: Entity,
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> ScatterChunk<TOut, TIn>
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    pub fn new(entity: Entity, scatter_layer: Entity) -> Self {
        Self {
            entity,
            scatter_layer,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScatterResult {
    pub transform: Transform,
    pub seed: u64,
}

impl PartialEq for ScatterResult {
    fn eq(&self, other: &Self) -> bool {
        self.transform.translation.x.to_bits() == other.transform.translation.x.to_bits()
            && self.transform.translation.y.to_bits() == other.transform.translation.y.to_bits()
            && self.transform.translation.z.to_bits() == other.transform.translation.z.to_bits()
    }
}

impl Eq for ScatterResult {}

impl Hash for ScatterResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transform.translation.x.to_bits().hash(state);
        self.transform.translation.y.to_bits().hash(state);
        self.transform.translation.z.to_bits().hash(state);
    }
}

#[derive(EntityEvent, Message, Clone, Debug)]
pub struct ScatterResults<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    pub entity: Entity,
    pub data: Vec<ScatterResult>,
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
    pub seed: u64,
    pub container_transform: Transform,
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> ScatterResults<TOut, TIn>
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
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
        seed: u64,
        container_transform: Transform,
    ) -> Self {
        Self {
            entity,
            root,
            layer,
            chunk,
            data,
            seed,
            container_transform,
            _phantom: PhantomData,
        }
    }

    pub fn with_data(mut self, data: Vec<ScatterResult>) -> Self {
        self.data = data;
        self
    }
}

impl<TOut, TIn> From<&Container> for ScatterResults<TOut, TIn>
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    fn from(value: &Container) -> Self {
        Self::new(
            value.entity,
            value.root_entity,
            value.layer_entity,
            value.chunk_entity,
            vec![],
            value.seed,
            value.transform,
        )
    }
}

#[derive(EntityEvent, Message, Clone)]
pub struct ClearScatterLayer(pub Entity);
