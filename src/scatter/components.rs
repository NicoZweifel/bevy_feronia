use crate::prelude::*;
use crate::scatter::utils::*;
use bevy_asset::{Assets, Handle};
use bevy_camera::prelude::Visibility;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{Quat, Vec3};
use bevy_pbr::StandardMaterial;
use bevy_reflect::Reflect;
use bevy_tasks::Task;
use bevy_transform::prelude::Transform;
use rand::Rng;

use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Component used to trigger a scatter operation for a specific target, layer and material type.
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterRequest<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    /// The entity that triggered the scatter (e.g., a chunk or the root).
    pub target_entity: Entity,
    /// The [`ScatterLayer`] entity that this request belongs to.
    pub layer_entity: Entity,
    /// The [`Chunk`] entity this request is for, if any (for chunked scattering).
    pub chunk_entity: Option<Entity>,

    #[reflect(ignore)]
    _marker: PhantomData<T>,
}

impl<T> ScatterRequest<T>
where
    T: ScatterMaterial,
{
    pub fn new(target_entity: Entity, layer_entity: Entity, chunk_entity: Option<Entity>) -> Self {
        Self {
            target_entity,
            layer_entity,
            chunk_entity,
            _marker: Default::default(),
        }
    }
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
    /// Optional [`InstanceScaleRange`] configuration.
    pub scale: Option<InstanceScaleRange>,
    /// Optional [`InstanceRotationYawRange`] configuration.
    pub rotation: Option<InstanceRotationYawRange>,
    /// Optional [`InstanceJitterStrength`] configuration.
    pub jitter: Option<InstanceJitterStrength>,
    /// Optional [`Avoidance`] radius configuration.
    pub avoidance: Option<Avoidance>,
    /// Optional height map [`Image`] handle.
    pub height_map_image: Option<Image>,
    /// Optional [`HeightMapConfig`].
    pub height_map_config: Option<HeightMapConfig>,
    /// Optional density map [`Image`] handle.
    pub density_map_image: Option<Image>,
    /// [`ScatterOccupancyMap`] containing a rasterized map of obstacles.
    pub external_avoidance_data: ScatterOccupancyMap,
    /// Optional [`LodDensity`] for this scatter operation.
    pub density: Option<LodDensity>,
}

/// Component that holds a [`Task`] for an in-progress CPU-based scatter job.
#[derive(Component, Debug)]
pub struct CpuScatterTask<T>(pub Task<T>);

/// Component that holds the result `T` from a completed [`CpuScatterTask`].
#[derive(Component, Debug)]
pub struct CpuScatterResult<T>(pub T);

/// Marker component defining a "prototype" or "source" entity to be scattered by a [`ScatterLayer`].
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterItem;

/// Marker component indicating that a [`ScatterRoot`] has been processed (e.g., its layers discovered).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterRootProcessed;

/// Marker component on a [`ScatterLayer`] indicating its scattering should be chunked.
///
/// If this is present, scattering will be tied to the [`Chunk`] lifecycle.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterChunked;

/// Component on a [`ScatterLayer`]'s [`ScatterItem`] holding a handle to a [`ScatterAsset`], which defines the properties
/// (mesh, material, LOD, etc.) of a scatterable object.
///
/// This is similar to [`ScatteredAsset`], but this component is on the original [`ScatterItem`] definition, in a [`ScatterLayer`].
#[derive(Component, Reflect, Debug, Clone, Deref, Default)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterItemAsset<T>(pub Handle<ScatterAsset<T>>)
where
    T: ScatterMaterialAsset;

/// Relational component linking a [`ScatterItem`] entity to its parent [`ScatterLayer`].
#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component, Debug, Clone)]
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
#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
pub struct ChunkInitScatter<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    #[reflect(ignore)]
    _marker: PhantomData<T>,
}

impl<T> Default for ChunkInitScatter<T>
where
    T: ScatterMaterial,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

/// Component that specifies the material types (`TOut`, `TIn`) for a [`ScatterLayer`].
///
/// This acts as a generic type marker to associate the layer with the correct scatter systems
/// and material pipelines.
#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
#[require(ScatterLayer)]
pub struct ScatterLayerType<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    #[reflect(ignore)]
    _marker: PhantomData<T>,
}

impl<T> Default for ScatterLayerType<T>
where
    T: ScatterMaterial,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
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
#[reflect(Component, Debug)]
#[relationship(relationship_target = ScatterRoot)]
pub struct ScatterLayerOf(pub Entity);

/// The root component of a scatter hierarchy, parenting multiple [`ScatterLayer`]s.
///
/// It holds overall configuration like [`LodConfig`] and state like the [`ScatterOccupancyMap`].
#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component, Debug)]
#[require(Transform, Visibility, LodConfig, ScatterOccupancyMap)]
#[relationship_target(relationship = ScatterLayerOf)]
pub struct ScatterRoot(Vec<Entity>);

/// Component to enable or disable scattering for a [`ScatterLayer`].
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
pub struct ScatterLayerEnabled(pub bool);

/// Controls the density for a specific [`ScatterLayer`].
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
pub struct DistributionDensity(pub f32);

impl From<f32> for DistributionDensity {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<i32> for DistributionDensity {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}

impl From<usize> for DistributionDensity {
    fn from(value: usize) -> Self {
        Self(value as f32)
    }
}

/// Enables density scaling when using chunks.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component, Debug)]
pub struct ScaleDensity;

/// Marker component placed on a spawned entity, indicating it was created by a scatter system.
///
/// Contains the [`Entity`] of the [`ScatterLayer`] it belongs to.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
pub struct ScatteredInstance(pub Entity);

/// Marker component placed on a spawned entity, indicating it was created by a scatter system.
///
/// Contains the [`Handle`] of the [`ScatterAsset`] it belongs to.
///
/// This is similar to [`ScatterItemAsset`], which is on the original [`ScatterItem`] definition, in a [`ScatterLayer`].
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
pub struct ScatteredAsset<T>(pub Handle<ScatterAsset<T>>)
where
    T: ScatterMaterialAsset;

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
#[component(on_add = Self::on_add)]
pub struct ScatteredPart<T>(pub (Handle<ScatterAsset<T>>, usize))
where
    T: ScatterMaterialAsset;

impl<T: ScatterMaterialAsset> ScatteredPart<T> {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let scatter_assets = world.get_resource::<Assets<ScatterAsset<T>>>().unwrap();
        let ScatteredPart((handle, index)) = world.get::<Self>(ctx.entity).unwrap();
        let asset = scatter_assets.get(handle).unwrap();
        let part = asset.parts.get(*index).cloned().unwrap();

        let mut cmd = world.commands();
        if part.properties.wind_affected {
            cmd.entity(ctx.entity).insert(WindAffected);
        }
        if let Some(render_layers) = part.properties.options.render_layers {
            cmd.entity(ctx.entity).insert(render_layers);
        }

        #[cfg(feature = "avian")]
        if let Some(collider) = part.o_collider.clone() {
            cmd.entity(ctx.entity).insert(collider);
        }
    }
}

/// Defines a texture-based density map for scattering.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
pub struct DistributionPattern(pub Handle<Image>);

/// Adds default random rotation yaw to the scattered instances.
///
/// See [`InstanceRotationYawRange`].

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Debug, Clone)]
#[require(InstanceRotationYawRange)]
pub struct InstanceRotationYaw;

/// Specifies a random yaw (Y-axis) rotation range for scattered instances.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Debug)]
pub struct InstanceRotationYawRange {
    /// The minimum rotation angle (in radians).
    pub min: f32,
    /// The maximum rotation angle (in radians).
    pub max: f32,
}

impl InstanceRotationYawRange {
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.min == self.max
    }

    pub fn into_quad(self, rng: &mut impl Rng) -> Quat {
        Quat::from_rotation_y(if self.is_fixed() {
            self.min
        } else {
            rng.random_range(self.min..self.max)
        })
    }
}

impl Default for InstanceRotationYawRange {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: std::f32::consts::TAU,
        }
    }
}

/// Adds default random uniform scaling to the scattered instances.
///
/// See [`InstanceScaleRange`].

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Debug, Clone)]
#[require(InstanceScaleRange)]
pub struct InstanceScale;

/// Specifies a random uniform scale range for scattered instances.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Debug, Clone)]
pub struct InstanceScaleRange {
    /// The minimum scale.
    pub min: f32,
    /// The maximum scale.
    pub max: f32,
}

impl InstanceScaleRange {
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.min == self.max
    }

    pub fn into_f32(self, rng: &mut impl Rng) -> f32 {
        if self.is_fixed() {
            self.min
        } else {
            rng.random_range(self.min..self.max)
        }
    }

    pub fn into_vec3(self, rng: &mut impl Rng) -> Vec3 {
        Vec3::splat(self.into_f32(rng))
    }
}

impl Default for InstanceScaleRange {
    fn default() -> Self {
        Self { min: 1., max: 2. }
    }
}

/// Adds a default random positional offset (jitter) to the scattered instances.
///
/// See [`InstanceJitterStrength`].
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Debug, Clone)]
#[require(InstanceJitterStrength)]
pub struct InstanceJitter;

/// Specifies the strength of a random positional offset (jitter) applied to scattered instances.
#[derive(Component, Reflect, Deref, DerefMut, Clone, Debug)]
#[reflect(Component, Debug, Clone)]
pub struct InstanceJitterStrength(pub f32);

impl Default for InstanceJitterStrength {
    fn default() -> Self {
        Self(1.)
    }
}

/// Specifies the density for scattering.
#[derive(Component, Reflect, Deref, DerefMut, Clone, Debug)]
#[reflect(Component, Debug)]
pub struct InstanceDensity(pub f32);

/// Adds a default random avoidance area to the scattered instances.
///
/// See [`Avoidance`].
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Debug, Clone)]
#[require(Avoidance)]
pub struct Avoid;

/// Specifies the minimum distance between the centers of scattered objects.
///
/// Gets scaled by the [`InstanceScaleRange`].
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component, Debug)]
pub struct Avoidance(pub f32);

impl Default for Avoidance {
    fn default() -> Self {
        Self(1.)
    }
}

/// Manages the state of a hierarchical scatter.
///
/// Used to process [`ScatterLayer`]s sequentially, allowing one layer
/// to fill the [`ScatterOccupancyMap`] before the next one runs.
///
/// Required to prevent foliage from being scattered onto rocks etc.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Debug)]
pub struct HierarchicalScatterState<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    /// Layers of the root, in the order they should be processed.
    pub ordered_layers: Vec<Entity>,
    /// Index of the layer currently being processed.
    pub current_layer_index: usize,
    pub pending_tasks: usize,
    #[reflect(ignore)]
    pub _marker: PhantomData<T>,
}
