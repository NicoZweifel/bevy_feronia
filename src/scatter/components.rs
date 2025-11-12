use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;
use bevy::tasks::Task;
use std::marker::PhantomData;

/// Component used to trigger a scatter operation for a specific target, layer and material type.
#[derive(Component)]
pub struct ScatterRequest<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    /// The entity that triggered the scatter (e.g., a chunk or the root).
    pub target_entity: Entity,
    /// The [`ScatterLayer`] entity that this request belongs to.
    pub layer_entity: Entity,
    /// The [`Chunk`] entity this request is for, if any (for chunked scattering).
    pub chunk_entity: Option<Entity>,

    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> ScatterRequest<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
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

/// Defines a 2D avoidance zone used by the scatter systems.
#[derive(Clone, Debug)]
pub struct AvoidanceData {
    /// The center of the avoidance zone in world space.
    pub world_pos: Vec3,
    /// The squared radius of the zone.
    pub radius_sq: f32,
    /// The scale of the object at this position, influencing the final avoidance radius.
    pub scale: f32,
}

/// Collection of all necessary data and configuration for a single scatter task.
///
/// Sent to a [`CpuScatterTask`] task for processing.
#[derive(Clone)]
pub struct ScatterTaskData {
    /// The scattering [`Container`] (e.g., AABB) to scatter within.
    pub container: Container,
    /// Optional [`MapHeight`] configuration.
    pub map_height: Option<MapHeight>,
    /// Optional [`InstanceScale`] configuration.
    pub scale: Option<InstanceScale>,
    /// Optional [`InstanceRotationYaw`] configuration.
    pub rotation: Option<InstanceRotationYaw>,
    /// Optional [`InstanceJitter`] configuration.
    pub jitter: Option<InstanceJitter>,
    /// Optional [`Avoidance`] radius configuration.
    pub avoidance: Option<Avoidance>,
    /// Optional height map [`Image`] handle.
    pub height_map_image: Option<Image>,
    /// Optional [`HeightMapConfig`].
    pub height_map_config: Option<HeightMapConfig>,
    /// Optional density map [`Image`] handle.
    pub density_map_image: Option<Image>,
    /// A list of pre-existing [`AvoidanceData`] zones to avoid (e.g., from other layers).
    pub external_avoidance_data: Vec<AvoidanceData>,
    /// Optional [`LodDensity`] for this scatter operation.
    pub density: Option<LodDensity>,
}

/// Component that holds a [`Task`] for an in-progress CPU-based scatter job.
#[derive(Component)]
pub struct CpuScatterTask<T>(pub Task<T>);

/// Component that holds the result `T` from a completed [`CpuScatterTask`].
#[derive(Component)]
pub struct CpuScatterResult<T>(pub T);

/// Marker component defining a "prototype" or "source" entity to be scattered by a [`ScatterLayer`].
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterItem;

/// Marker component indicating that a [`ScatterRoot`] has been processed (e.g., its layers discovered).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterRootProcessed;

/// Marker component on a [`ScatterLayer`] indicating its scattering should be chunked.
///
/// If this is present, scattering will be tied to the [`Chunk`] lifecycle.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterChunked;

/// Component on a [`ScatterLayer`]'s [`ScatterItem`] holding a handle to a [`ScatterAsset`], which defines the properties
/// (mesh, material, LOD, etc.) of a scatterable object.
///
/// This is similar to [`ScatteredAsset`], but this component is on the original [`ScatterItem`] definition, in a [`ScatterLayer`].
#[derive(Component, Reflect, Debug, Clone, Deref)]
#[reflect(Component)]
pub struct ScatterItemAsset<T>(pub Handle<ScatterAsset<T>>)
where
    T: Asset + Clone;

/// Relational component linking a [`ScatterItem`] entity to its parent [`ScatterLayer`].
#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterLayer)]
pub struct ScatterItemOf(pub Entity);

/// Component defining a "layer" of scatterable objects (e.g., "grass", "rocks").
///
/// Acts as a parent for [`ScatterItem`] entities via the [`ScatterItemOf`] relationship.
#[derive(Component, Reflect, Default)]
#[require(Transform, Visibility)]
#[relationship_target(relationship = ScatterItemOf)]
#[reflect(Component)]
pub struct ScatterLayer(Vec<Entity>);

/// Marker component on a [`Chunk`] to trigger scattering when the chunk is initialized.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkInitScatter<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for ChunkInitScatter<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

/// Component that specifies the material types (`TOut`, `TIn`) for a [`ScatterLayer`].
///
/// This acts as a generic type marker to associate the layer with the correct scatter systems
/// and material pipelines.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScatterLayerType<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for ScatterLayerType<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

/// Marker component to signify that a `ScatterLayer` has already had its
/// sources discovered and its `ScatterItem's generated.
#[derive(Component)]
pub struct ScatterLayerProcessed;

/// Marker component indicating that a child entity of a [`ScatterLayer`] (e.g., a `ScatterItem`) has been processed.
#[derive(Component)]
pub struct ScatterLayerChildProcessed;

/// Marker component for [`ScatterLayer]` Observers that observes the scatter system (e.g., chunked scatter, normal scatter).
#[derive(Component)]
pub struct ScatterObserver;

/// Relational component linking a [`ScatterLayer`] entity to its parent [`ScatterRoot`].
#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterRoot)]
pub struct ScatterLayerOf(pub Entity);

/// The root component of a scatter hierarchy, parenting multiple [`ScatterLayer`]s.
///
/// It holds overall configuration like [`LodConfig`] and state like the [`ScatterOccupancyMap`].
#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, LodConfig, ScatterOccupancyMap)]
#[relationship_target(relationship = ScatterLayerOf)]
pub struct ScatterRoot(Vec<Entity>);

/// Component to enable or disable scattering for a [`ScatterLayer`].
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatterLayerEnabled(pub bool);

/// Controls the density for a specific [`ScatterLayer`].
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct DistributionDensity(pub f32);

/// Enables density scaling when using chunks.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScaleDensity;

/// Marker component placed on a spawned entity, indicating it was created by a scatter system.
///
/// Contains the [`Entity`] of the [`ScatterLayer`] it belongs to.
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatteredInstance(pub Entity);

/// Marker component placed on a spawned entity, indicating it was created by a scatter system.
///
/// Contains the [`Handle`] of the [`ScatterAsset`] it belongs to.
///
/// This is similar to [`ScatterItemAsset`], which is on the original [`ScatterItem`] definition, in a [`ScatterLayer`].
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatteredAsset<T>(pub Handle<ScatterAsset<T>>)
where
    T: Asset + Clone;

/// Defines a texture-based density map for scattering.
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct DistributionPattern(pub Handle<Image>);

/// Specifies a random yaw (Y-axis) rotation range for scattered instances.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct InstanceRotationYaw {
    /// The minimum rotation angle (in radians).
    pub min: f32,
    /// The maximum rotation angle (in radians).
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

/// Specifies a random uniform scale range for scattered instances.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct InstanceScale {
    /// The minimum scale.
    pub min: f32,
    /// The maximum scale.
    pub max: f32,
}

impl Default for InstanceScale {
    fn default() -> Self {
        Self { min: 1., max: 2. }
    }
}

/// Specifies a random positional offset (jitter) applied to scattered instances.
#[derive(Component, Reflect, Deref, DerefMut, Clone)]
#[reflect(Component)]
pub struct InstanceJitter(pub f32);

impl Default for InstanceJitter {
    fn default() -> Self {
        Self(1.)
    }
}

/// Specifies the density for scattering.
#[derive(Component, Reflect, Deref, DerefMut, Clone)]
#[reflect(Component)]
pub struct InstanceDensity(pub f32);

/// Specifies the minimum distance between the centers of scattered objects.
///
/// Gets scaled by the [`InstanceScale`].
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Avoidance(pub f32);

impl Default for Avoidance {
    fn default() -> Self {
        Self(1.)
    }
}

/// Temporary component that manages the state of a hierarchical scatter.
///
/// Used to process [`ScatterLayer`]s sequentially, allowing one layer
/// to fill the [`ScatterOccupancyMap`] before the next one runs.
///
/// Required to prevent foliage from being scattered onto rocks etc.
#[derive(Component)]
pub struct HierarchicalScatterState<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    /// Layers of the root, in the order they should be processed.
    pub ordered_layers: Vec<Entity>,
    /// Index of the layer currently being processed.
    pub current_layer_index: usize,
    pub _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for HierarchicalScatterState<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
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

/// A component on the [`ScatterRoot`] that accumulates [`AvoidanceData`]
/// from processed layers.
///
/// This allows later layers to avoid spawning on top of instances from previous layers, e.g., no foliage on rocks.
#[derive(Component, Default)]
pub struct ScatterOccupancyMap {
    /// A list of occupied zones from previously scattered layers.
    pub occupied_zones: Vec<AvoidanceData>,
}
