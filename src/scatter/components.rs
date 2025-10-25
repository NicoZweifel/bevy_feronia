use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;
use bevy::render::render_resource::Buffer;
use bevy::tasks::Task;
use std::marker::PhantomData;

#[derive(Component)]
pub struct ScatterRequest<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    pub target_entity: Entity,
    pub layer_entity: Entity,
    pub chunk_entity: Option<Entity>,
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> ScatterRequest<TOut, TIn>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
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

#[derive(Clone, Debug)]
pub struct AvoidanceData {
    pub world_pos: Vec3,
    pub radius_sq: f32,
    pub scale: f32,
}

#[derive(Clone)]
pub struct ScatterTaskData {
    pub container: Container,
    pub map_height: Option<MapHeight>,
    pub scale: Option<InstanceScale>,
    pub rotation: Option<InstanceRotationYaw>,
    pub jitter: Option<InstanceJitter>,
    pub avoidance: Option<Avoidance>,
    pub height_map_image: Option<Image>,
    pub height_map_config: Option<HeightMapConfig>,
    pub density_map_image: Option<Image>,
    pub external_avoidance_data: Vec<AvoidanceData>,
    pub density: Option<LodLevelDensity>,
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

/// Determines whether a `ScatterLayer` inside a `ScatterRoot` is scattered in chunks.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterChunked;

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
pub struct ChunkInitScatter<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for ChunkInitScatter<TOut, TIn>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScatterLayerType<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for ScatterLayerType<TOut, TIn>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
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
pub struct ScatterLayerChildProcessed;

#[derive(Component)]
pub struct ScatterObserver;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterRoot)]
pub struct ScatterLayerOf(pub Entity);

#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, LodConfig, ScatterOccupancyMap)]
#[relationship_target(relationship = ScatterLayerOf)]
pub struct ScatterRoot(Vec<Entity>);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatterLayerEnabled(pub bool);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct DistributionDensity(pub f32);

/// Enables density scaling when using chunks.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScaleDensity;

/// Indicated if an Entity was added by a scatter system layer.
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatteredInstance(pub Entity);

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

impl Default for InstanceRotationYaw {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: std::f32::consts::TAU,
        }
    }
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

impl Default for InstanceJitter {
    fn default() -> Self {
        Self(1.)
    }
}

#[derive(Component, Reflect, Deref, DerefMut, Clone)]
#[reflect(Component)]
pub struct InstanceDensity(pub f32);

/// Specifies the minimum distance between the centers of scattered objects.
/// Prevents them from spawning on top of each other.
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Avoidance(pub f32);

impl Default for Avoidance {
    fn default() -> Self {
        Self(1.)
    }
}

/// Temporary component that manages the state of a hierarchical scatter.
#[derive(Component)]
pub struct HierarchicalScatterState<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    /// Layers of the root, in the order they should be processed.
    pub ordered_layers: Vec<Entity>,
    /// Index of the layer currently being processed.
    pub current_layer_index: usize,
    pub(crate) _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for HierarchicalScatterState<TOut, TIn>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    fn default() -> Self {
        Self {
            ordered_layers: vec![],
            current_layer_index: 0,
            _phantom: Default::default(),
        }
    }
}

/// Temporary component on the `ScatterRoot` that accumulates occupied zones.
#[derive(Component, Default)]
pub struct ScatterOccupancyMap {
    pub occupied_zones: Vec<AvoidanceData>,
}
