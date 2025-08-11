use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

#[derive(Event, BufferedEvent, Debug, Clone)]
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
}

impl From<On<'_, ScatterResults>> for SpawnTrigger {
    fn from(value: On<ScatterResults>) -> Self {
        Self {
            chunk: value.chunk,
            layer: value.layer,
            target: value.target(),
            data: value.data.clone(),
            root: value.root,
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
}

pub trait ProtoTypes<TOut, TType>
where
    TOut: Asset + Clone,
    TType: ProtoType<TOut> + Asset + Clone,
{
    // TODO use trait or remove
}

pub trait ProtoType<T>
where
    T: Asset + Clone,
{
    fn mesh(&self) -> &Handle<Mesh>;
    fn material(&self) -> &Handle<T>;
    fn wind(&self) -> Option<&Wind>;
    fn aabb(&self) -> &Aabb;
    fn lod(&self) -> &LodLevel;
}

pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LodLevel(pub u32);
