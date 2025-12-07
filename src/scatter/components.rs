use crate::prelude::*;
use crate::scatter::utils::*;
use bevy_asset::Handle;
use bevy_camera::prelude::Visibility;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{IVec2, Quat, Vec3};
use bevy_pbr::StandardMaterial;
use bevy_platform::collections::HashMap;
use bevy_reflect::Reflect;
use bevy_tasks::Task;
use bevy_transform::prelude::Transform;
use rand::Rng;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Component used to trigger a scatter operation for a specific target, layer and material type.
#[derive(Component)]
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

    _phantom: PhantomData<T>,
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
            _phantom: Default::default(),
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
    /// A list of pre-existing [`SpatialAvoidanceGrid`] zones to avoid (e.g., from other layers).
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
    _phantom: PhantomData<T>,
}

impl<T> Default for ChunkInitScatter<T>
where
    T: ScatterMaterial,
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
#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
#[require(ScatterLayer)]
pub struct ScatterLayerType<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    _phantom: PhantomData<T>,
}

impl<T> Default for ScatterLayerType<T>
where
    T: ScatterMaterial,
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

/// Defines a texture-based density map for scattering.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component, Debug)]
pub struct DistributionPattern(pub Handle<Image>);

/// Specifies a random yaw (Y-axis) rotation range for scattered instances.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Debug)]
pub struct InstanceRotationYaw {
    /// The minimum rotation angle (in radians).
    pub min: f32,
    /// The maximum rotation angle (in radians).
    pub max: f32,
}

impl InstanceRotationYaw {
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.min == self.max
    }

    pub fn into_quad(self, rng: &mut impl Rng) -> Quat {
        Quat::from_rotation_y(
            self.is_fixed()
                .then(|| self.min)
                .unwrap_or_else(|| rng.random_range(self.min..self.max)),
        )
    }
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
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Debug)]
pub struct InstanceScale {
    /// The minimum scale.
    pub min: f32,
    /// The maximum scale.
    pub max: f32,
}

impl InstanceScale {
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.min == self.max
    }

    pub fn into_f32(self, rng: &mut impl Rng) -> f32 {
        self.is_fixed()
            .then(|| self.min)
            .unwrap_or_else(|| rng.random_range(self.min..self.max))
    }

    pub fn into_vec3(self, rng: &mut impl Rng) -> Vec3 {
        Vec3::splat(self.into_f32(rng))
    }
}

impl Default for InstanceScale {
    fn default() -> Self {
        Self { min: 1., max: 2. }
    }
}

/// Specifies a random positional offset (jitter) applied to scattered instances.
#[derive(Component, Reflect, Deref, DerefMut, Clone, Debug)]
#[reflect(Component, Debug)]
pub struct InstanceJitter(pub f32);

impl Default for InstanceJitter {
    fn default() -> Self {
        Self(1.)
    }
}

/// Specifies the density for scattering.
#[derive(Component, Reflect, Deref, DerefMut, Clone, Debug)]
#[reflect(Component, Debug)]
pub struct InstanceDensity(pub f32);

/// Specifies the minimum distance between the centers of scattered objects.
///
/// Gets scaled by the [`InstanceScale`].
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component, Debug)]
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
    pub _phantom: PhantomData<T>,
}

/// A component on the [`ScatterRoot`] that accumulates obstacle data from processed layers.
///
/// This allows later layers to avoid spawning on top of instances from previous layers, e.g., no foliage on rocks.
///
/// Defines a 2.5D avoidance zone used by the scatter systems.
///
/// It stores the height of the obstacle at the occupied location,
/// which is used to avoid spawning on top of it while still spawning above rocks in the ground.
///
/// TODO
/// https://github.com/NicoZweifel/bevy_feronia/issues/56
/// https://github.com/NicoZweifel/bevy_feronia/issues/43
#[derive(Component, Reflect, Clone)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterOccupancyMap {
    pub cell_size: f32,
    pub cells: HashMap<IVec2, f32>,
}

impl Default for ScatterOccupancyMap {
    fn default() -> Self {
        Self {
            cell_size: 1.,
            cells: HashMap::default(),
        }
    }
}

impl Debug for ScatterOccupancyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScatterOccupancyMap")
            .field("cell_size", &self.cell_size)
            .field("cells", &self.cells.len())
            .finish()
    }
}

impl ScatterOccupancyMap {
    /// Convert a world position to a grid space coordinate.
    #[inline]
    fn to_grid(&self, pos: Vec3) -> IVec2 {
        IVec2::new(
            (pos.x / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Check if a position is occupied and if it is, check if it is below the stored height in this cell, e.g.,
    /// it's not above a rock in the ground, but inside/on it.
    pub fn is_occupied(&self, pos: Vec3) -> bool {
        let grid_pos = self.to_grid(pos);

        self.cells
            .get(&grid_pos)
            .map(|height| pos.y <= *height)
            .unwrap_or_default()
    }

    /// Adds a circular obstacle to the map.
    ///
    /// # Arguments
    /// * `center` - World position of the object.
    /// * `radius` - Scaled radius of the circle in world units.
    pub fn add_circle(&mut self, center: Vec3, radius: f32) {
        if radius <= 0.0 {
            return;
        }

        let min_world = center - Vec3::new(radius, 0.0, radius);
        let max_world = center + Vec3::new(radius, 0.0, radius);

        let min_grid = self.to_grid(min_world);
        let max_grid = self.to_grid(max_world);

        let radius_sq = radius.powi(2);
        let half_cell = self.cell_size / 2.0;

        for x in min_grid.x..=max_grid.x {
            for z in min_grid.y..=max_grid.y {
                let grid_pos = IVec2::new(x, z);

                let world_cell_x = (x as f32 * self.cell_size) + half_cell;
                let world_cell_z = (z as f32 * self.cell_size) + half_cell;

                let dist_x = world_cell_x - center.x;
                let dist_z = world_cell_z - center.z;
                let dist_sq = dist_x.powi(2) + dist_z.powi(2);

                if dist_sq <= radius_sq {
                    self.cells
                        .entry(grid_pos)
                        .and_modify(|h| *h = h.max(center.y))
                        .or_insert(center.y);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{IVec2, Vec3};

    #[test]
    fn test_to_grid_should_return_correct_coordinates() {
        // Arrange
        let map = ScatterOccupancyMap {
            cell_size: 2.0,
            ..Default::default()
        };
        let input_position = Vec3::new(2.5, 0.0, -1.5);
        let expected = IVec2::new(1, -1);

        // Act
        let result = map.to_grid(input_position);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_position_should_be_occupied() {
        // Arrange
        let mut map = ScatterOccupancyMap::default();
        let height = 5.0;
        let cell_coord = IVec2::new(0, 0);

        map.cells.insert(cell_coord, height);

        //  Position is inside the rock (y < height)
        let pos = Vec3::new(0.5, 4.0, 0.5);

        // Act
        let result = map.is_occupied(pos);

        // Assert
        assert!(result);
    }

    #[test]
    fn test_positions_should_be_free() {
        // Arrange
        let mut map = ScatterOccupancyMap::default();
        let height = 5.0;
        let cell_coord = IVec2::new(0, 0);

        map.cells.insert(cell_coord, height);

        // Position is above the rock (y > height)
        let pos = Vec3::new(0.5, 6.0, 0.5);

        // Act
        let result = !map.is_occupied(pos);

        // Assert
        assert!(result);
    }

    #[test]
    fn test_circle_should_occupy_area() {
        // Arrange
        let mut map = ScatterOccupancyMap {
            cell_size: 1.0,
            ..Default::default()
        };
        let center = Vec3::new(0.5, 0.0, 0.5);

        // Radius of 1.1 covers neighbor at 1,0 with distance 1.0,
        // but not diagonal at 1,1 with distance 1.41 (sqrt(2)).
        let radius = 1.1;

        // Act
        map.add_circle(center, radius);

        // Assert
        let center_occupied = map.cells.contains_key(&IVec2::new(0, 0));
        let neighbor_occupied = map.cells.contains_key(&IVec2::new(1, 0));
        let diagonal_occupied = map.cells.contains_key(&IVec2::new(1, 1));

        assert!(center_occupied, "Center should be occupied");
        assert!(neighbor_occupied, "Neighbor should be occupied");
        assert!(!diagonal_occupied, "Diagonal should not be occupied");
    }

    #[test]
    fn test_circle_should_store_height() {
        // Arrange
        let mut map = ScatterOccupancyMap::default();
        let center = Vec3::new(0.5, 12.5, 0.5);
        let radius = 0.5;

        // Act
        map.add_circle(center, radius);

        // Assert
        let stored_height = *map.cells.get(&IVec2::new(0, 0)).unwrap();
        assert_eq!(stored_height, 12.5);
    }
}
