use std::f32::consts::TAU;
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

#[derive(Clone, Debug, Reflect)]
pub struct MaterialOptions {
    // Determines whether the material is controlled externally and
    // isn't automatically updated and kept in sync with the Wind resource.
    pub controlled: bool,

    pub debug_color: Color,

    pub debug: bool,

    pub enable_billboarding: bool,
    pub fast_normals: bool,
    pub edge_correction_factor: f32,
    pub curve_factor: f32,
    pub lod_threshold: f32,
}

impl Default for MaterialOptions {
    fn default() -> Self {
        Self {
            controlled: false,
            debug_color: Default::default(),
            debug: false,
            enable_billboarding: false,
            fast_normals: false,
            edge_correction_factor: 0.0,
            curve_factor: 0.0,
            // TODO sync/cleanup with LOD systems / chunks systems
            lod_threshold: 50.,
        }
    }
}

impl<TOut, TIn> From<On<'_, '_, ScatterResults<TOut, TIn>>> for SpawnTrigger
where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    fn from(value: On<ScatterResults<TOut, TIn>>) -> Self {
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
    fn wind(&self) -> &Wind;
    fn aabb(&self) -> &Aabb;
    fn lod(&self) -> &LevelOfDetail;
    fn material_options(&self) -> &MaterialOptions;
}

pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LevelOfDetail(pub u32);

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EnableDebug;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EnableBillboarding;

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct EdgeCorrectionFactor(pub f32);

impl Default for EdgeCorrectionFactor {
    fn default() -> Self {
        Self(0.001)
    }
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct CurveFactor(pub f32);

impl Default for CurveFactor {
    fn default() -> Self {
       Self(TAU)
    }
}

#[derive(Clone, Default)]
pub struct ThreadSafeImage {
    /// Raw pixel data.
    pub pixels: Vec<u8>,
    pub dimensions: UVec2,
}
