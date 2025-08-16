use crate::prelude::*;
use crate::scatter::utils::Container;
use bevy::prelude::*;
use bevy::render::render_resource::Buffer;
use bevy::tasks::Task;
use std::marker::PhantomData;

#[derive(Component)]
pub struct ScatterRequest<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub target_entity: Entity,
    pub layer_entity: Entity,
    pub chunk_entity: Option<Entity>,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> ScatterRequest<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub fn new(target_entity: Entity, layer_entity: Entity, chunk_entity: Option<Entity>) -> Self {
        Self {
            target_entity,
            layer_entity,
            chunk_entity,
            _phantom: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct ScatterTaskData {
    pub container: Container,
    pub map_height: Option<MapHeight>,
    pub scale: Option<InstanceScale>,
    pub rotation: Option<InstanceRotationYaw>,
    pub jitter: Option<InstanceJitter>,
    pub height_map_image: Option<Image>,
    pub height_map_config: Option<HeightMapConfig>,
    pub density_map_image: Option<Image>,
}

#[derive(Component)]
pub struct GpuScatterJobInProgress;

#[derive(Component)]
pub struct GpuScatterResult {
    pub instance_buffer: Buffer,
    pub instance_count: u32,
}

#[derive(Component)]
pub struct CpuScatterTask<T>(pub Task<T>);

#[derive(Component)]
pub struct CpuScatterResult<T>(pub T);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterItem;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterRootProcessed;

#[derive(Component, Reflect, Debug, Clone, Deref)]
#[reflect(Component)]
pub struct ScatterItemAsset<T>(pub Handle<ScatterAsset<T>>)
where
    T: Asset + Clone;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterLayer)]
pub struct ScatterItemOf(pub Entity);

#[derive(Component, Reflect, Default)]
#[require(Transform, Visibility)]
#[relationship_target(relationship = ScatterItemOf)]
#[reflect(Component)]
pub struct ScatterLayer(Vec<Entity>);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkInitScatter<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
> {
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> Default for ChunkInitScatter<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScatterLayerType<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
> {
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> Default for ScatterLayerType<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

/// A marker component to signify that a `ScatterLayer` has already had its
/// sources discovered and its `ScatterItem's generated.
#[derive(Component)]
pub struct ScatterLayerProcessed;

#[derive(Component)]
pub struct ScatterObserver;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterRoot)]
pub struct ScatterLayerOf(pub Entity);

#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, LodConfig)]
#[relationship_target(relationship = ScatterLayerOf)]
pub struct ScatterRoot(Vec<Entity>);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatterLayerEnabled(pub bool);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct DistributionDensity(pub f32);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DistributionPattern {
    pub density_map: Handle<Image>,
    pub scale: f32,
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct InstanceRotationYaw {
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct InstanceScale {
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Reflect, Deref, DerefMut, Clone)]
#[reflect(Component)]
pub struct InstanceJitter(pub f32);
