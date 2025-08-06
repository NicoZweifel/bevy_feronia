use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rand::prelude::{IndexedRandom, IteratorRandom};
use std::borrow::Cow;

#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    pub items: Vec<ScatterItemType<T>>,
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
    pub fn new(items: Vec<ScatterItemType<T>>, trigger: SpawnTrigger) -> Self {
        Self { items, trigger }
    }
}

pub trait ProtoTypes<TOut, TType>
where
    TOut: Asset + Clone,
    TType: ProtoType<TOut> + Asset + Clone,
{
    fn choose(
        &self,
        scatter_items: &Vec<ScatterItemType<TOut>>,
    ) -> Option<HashMap<LodLevel, Handle<TType>>>;
}

pub trait ProtoType<T>
where
    T: Asset + Clone,
{
    fn mesh(&self) -> &Handle<Mesh>;
    fn material(&self) -> &Handle<T>;
    fn wind(&self) -> &Wind;
    fn aabb(&self) -> &Aabb;
    fn lod(&self) -> &LodLevel;
}

pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct ScatterAssets<T>(pub Vec<Handle<ScatterAsset<T>>>)
where
    T: Asset + Clone;

impl<T> Default for ScatterAssets<T>
where
    T: Asset + Clone,
{
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: Asset + Clone> ProtoTypes<T, ScatterAsset<T>> for ScatterAssets<T> {
    fn choose(
        &self,
        scatter_items: &Vec<ScatterItemType<T>>,
    ) -> Option<HashMap<LodLevel, Handle<ScatterAsset<T>>>> {
        let mut rng = rand::rng();

        info!("Scattering Assets! {:?}", scatter_items.len());

        let items = scatter_items.iter().map(|x| match x {
            ScatterItemType::Handle(x) => Some(x.clone()),
            _ => self.0.choose(&mut rng).map(|x| x.clone()),
        });

        items
            .choose(&mut rand::rng())?
            .map(|x| HashMap::from([(LodLevel(0), x)]))
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct ScatterAssetsNameMap<T>(HashMap<Name, HashMap<LodLevel, Handle<ScatterAsset<T>>>>)
where
    T: Asset + Clone;

impl<T> Default for ScatterAssetsNameMap<T>
where
    T: Asset + Clone,
{
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T: Asset + Clone> ProtoTypes<T, ScatterAsset<T>> for ScatterAssetsNameMap<T> {
    fn choose(
        &self,
        scatter_items: &Vec<ScatterItemType<T>>,
    ) -> Option<HashMap<LodLevel, Handle<ScatterAsset<T>>>> {
        let mut rng = rand::rng();

        info!("Scattering Assets! {:?}", scatter_items.len());

        let scatter_type = scatter_items.choose(&mut rng)?;
        match scatter_type {
            ScatterItemType::Handle(x) => Some(HashMap::from([(LodLevel(0), x.clone())])),
            ScatterItemType::Name(x) => {
                info!("Scattering Asset {x}!");
                self.get(x)
                    .map_or(self.0.values().choose(&mut rng).map(|x| x.clone()), |x| {
                        Some(x.clone())
                    })
            }
        }
    }
}

pub fn scatter_item<T>(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
    T: Asset + Clone,
{
    (ScatterItem, ScatterItemType::<T>::Name(Name::new(name)))
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct ScatterAsset<T>
where
    T: Asset + Clone,
{
    pub mesh: Handle<Mesh>,
    pub material: Handle<T>,
    pub wind: Wind,
    pub aabb: Aabb,
    pub lod_level: LodLevel,
    pub name: Option<Name>,
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LodLevel(pub u32);

impl<T: Asset + Clone> ProtoType<T> for ScatterAsset<T> {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn material(&self) -> &Handle<T> {
        &self.material
    }

    fn wind(&self) -> &Wind {
        &self.wind
    }

    fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    fn lod(&self) -> &LodLevel {
        &self.lod_level
    }
}
