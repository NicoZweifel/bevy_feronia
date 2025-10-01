use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

#[derive(Event, Message, Debug, Clone)]
pub struct SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    pub items: Vec<ScatterItemAsset<T>>,
    pub trigger: SpawnTrigger,
}

#[derive(Clone, Debug)]
pub struct SpawnTrigger {
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
    pub target: Entity,
    pub data: Vec<ScatterResult>,
    pub seed: u64,
}

impl<TIn, TOut> From<On<'_, '_, ScatterResults<TIn, TOut>>> for SpawnTrigger
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn from(value: On<ScatterResults<TIn, TOut>>) -> Self {
        Self {
            chunk: value.chunk,
            layer: value.layer,
            target: value.entity,
            data: value.data.clone(),
            root: value.root,
            seed: value.seed,
        }
    }
}

impl<T> SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    pub fn new(items: Vec<ScatterItemAsset<T>>, trigger: SpawnTrigger) -> Self {
        Self { items, trigger }
    }

    pub fn with_items(mut self, items: Vec<ScatterItemAsset<T>>) -> Self {
        self.items = items;
        self
    }
}

impl<T> From<SpawnTrigger> for SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    fn from(value: SpawnTrigger) -> Self {
        Self::new(Vec::new(), value)
    }
}

pub trait ProtoType<T>
where
    T: Asset + Clone,
{
    fn mesh(&self) -> &Handle<Mesh>;
    fn material(&self) -> &Handle<T>;
    fn wind(&self) -> Option<&Wind>;
    fn aabb(&self) -> &Aabb;
    fn lod(&self) -> &LevelOfDetail;
}

pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LevelOfDetail(pub u32);

#[derive(Clone, Default)]
pub struct ThreadSafeImage {
    /// Raw pixel data.
    pub pixels: Vec<u8>,
    pub dimensions: UVec2,
}
