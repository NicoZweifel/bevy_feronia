use std::hash::{Hash, Hasher};
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

impl PartialEq for ScatterResult {
    fn eq(&self, other: &Self) -> bool {
        self.0.translation.x.to_bits() == other.0.translation.x.to_bits()
            && self.0.translation.y.to_bits() == other.0.translation.y.to_bits()
            && self.0.translation.z.to_bits() == other.0.translation.z.to_bits()
    }
}

impl Eq for ScatterResult {}

impl Hash for ScatterResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.translation.x.to_bits().hash(state);
        self.0.translation.y.to_bits().hash(state);
        self.0.translation.z.to_bits().hash(state);
    }
}

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
    pub seed: u64,
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
        seed: u64,
    ) -> Self {
        Self {
            entity,
            root,
            layer,
            chunk,
            data,
            seed,
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
            value.entity,
            value.root_entity,
            value.layer_entity,
            value.chunk_entity,
            vec![],
            value.seed,
        )
    }
}
