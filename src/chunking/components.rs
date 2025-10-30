use crate::core::LevelOfDetail;
use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;

/// Component configuring Level of Detail (LOD) settings, including distances and densities.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct LodConfig {
    /// A list of distance thresholds, one for each LOD.
    /// Ordered from the highest detail (LOD 0) to the lowest (LOD n).
    pub distance: Vec<LodDistance>,
    /// A list of density multipliers (0.0 to 1.0), one for each LOD.
    /// Ordered from the highest detail (LOD 0) to the lowest (LOD n).
    pub density: Vec<LodDensity>,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            distance:
            // LODs are ordered from High (0) to Low (n).
            vec![
                // Level 0: High
                30.0.into(),
                // Level 1: Medium
                60.0.into(),
                // Level 2: Low
                default(), // f32::MAX
            ],
            density: vec![
                // Level 0: High
                1.0.into(),
                // Level 1: Medium
                0.3.into(),
                // Level 2: Low
                0.1.into(),
                // Fallback
                default() // 0.0
            ]
        }
    }
}

impl From<Vec<LodDistance>> for LodConfig {
    fn from(value: Vec<LodDistance>) -> Self {
        Self {
            distance: value,
            ..default()
        }
    }
}

impl LodConfiguration for LodConfig {
    fn get(&self) -> &Vec<LodDistance> {
        &self.distance
    }
}

/// Trait defining an interface for accessing LOD distance configurations.
pub trait LodConfiguration {
    /// Returns the list of [`LodDistance`] thresholds.
    fn get(&self) -> &Vec<LodDistance>;

    /// Returns the maximum LOD (index) defined by this configuration.
    fn get_max_lod(&self) -> u32 {
        (self.get().len() - 1) as u32
    }

    /// Gets the [`LodDistance`] for a specific LOD `level`.
    fn get_lod_config(&self, level: u32) -> &LodDistance {
        &self.get()[level as usize]
    }

    /// Calculates the [`VisibilityRange`] for a given `lod`.
    fn get_visibility_range(&self, lod: LevelOfDetail) -> VisibilityRange {
        let current_lod_dist = self
            .get()
            .get(*lod as usize)
            .map(|x| **x)
            .unwrap_or(*LodDistance::default());

        let fade_band_multiplier = 0.1;

        let start_margin = if *lod == 0 {
            0.0..0.0
        } else {
            let prev_lod_dist = self
                .get()
                .get(*lod as usize - 1)
                .map(|x| **x)
                .unwrap_or(*LodDistance::default());

            let fade_band = prev_lod_dist * fade_band_multiplier;

            prev_lod_dist..(prev_lod_dist + fade_band)
        };

        let end_margin = if *lod == self.get_max_lod() {
            f32::MAX..f32::MAX
        } else {
            let fade_band = current_lod_dist * fade_band_multiplier;

            current_lod_dist..(current_lod_dist + fade_band)
        };

        VisibilityRange {
            start_margin,
            end_margin,
            use_aabb: false,
        }
    }
}

/// Component specifying the LOD distance thresholds specifically for a chunk hierarchy.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkLodConfig(pub Vec<LodDistance>);

impl Default for ChunkLodConfig {
    fn default() -> Self {
        Self(
            // LODs are ordered from High (0) to Low (n).
            vec![
                // Level 0: High
                60.0.into(),
                // Level 1: Medium
                120.0.into(),
                // Level 2: Low
                250.0.into(),
                // Level 3: Root
                LodDistance::default(), // f32::MAX
            ],
        )
    }
}

impl LodConfiguration for ChunkLodConfig {
    fn get(&self) -> &Vec<LodDistance> {
        &self.0
    }
}

/// Component specifying the size multipliers for chunks at each LOD.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkSizeScalarConfig(pub Vec<ChunkSizeScalar>);

/// Size of a `ChunkRoot` dimension in top-level (Low LOD) chunks.
///
/// Defines how many root-level chunks exist (e.g., 2 means a 2x2 grid).
// TODO use/sync with ChunkSizeScalar (depends on correct configuration at the moment)
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkRootSizeDim(pub u32);

impl Default for ChunkRootSizeDim {
    fn default() -> Self {
        Self(2)
    }
}

/// Component storing the 2D grid coordinate of a chunk.
#[derive(Component, Reflect, Deref, DerefMut, Debug, Hash)]
#[reflect(Component)]
pub struct ChunkCoord(pub IVec2);

/// Marker component to trigger initialization logic for a new chunk.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ChunkInitialize;

/// Marker component identifying the entity representing the center of the chunking system.
///
/// This should be added to the camera or the player controller.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkCenter;

/// Marker component indicating that a chunk is allowed to split into sub-chunks.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSplit;

/// Marker component indicating that a chunk is allowed to merge back into a parent chunk.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanMerge;

/// Component storing the current LOD of a chunk (0 is the highest detail).
#[derive(Component, Reflect, Deref, DerefMut, Default, Debug, Clone)]
#[reflect(Component)]
pub struct ChunkLevel(pub u32);

/// Component storing the scalar size in [`BaseChunkSize`] units of this chunk.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkSize(pub u32);

/// Component storing the base size of a level 0 (the highest detail) chunk.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct BaseChunkSize(pub Vec3);

/// Component specifying the distance at which a chunk should merge with its siblings.
///
/// Requires the [`CanMerge`] component.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[require(CanMerge)]
#[reflect(Component)]
pub struct MergeDistance(pub f32);

/// Marker component indicating a chunk is currently in the process of merging.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Merging;

/// Component specifying the distance at which a chunk should split into sub-chunks.
///
/// Requires the [`CanSplit`] component.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[require(CanSplit)]
#[reflect(Component)]
pub struct SplitDistance(pub f32);

/// Marker component identifying a chunk entity.
#[derive(Component, Debug, Clone, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
#[derive(Default)]
pub struct Chunk;

/// Relational component linking a [`Chunk`] entity to its [`ChunkRoot`].
#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ChunkRoot)]
pub struct ChunkOf(pub Entity);

/// Component identifying the root entity of a chunk hierarchy.
///
/// It holds references to its direct child chunks (which may be `Chunk` or other `ChunkRoot` entities).
#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(
    Transform,
    Visibility,
    ChunkLodConfig,
    ChunkSizeScalarConfig,
    ChunkRootSizeDim
)]
#[relationship_target(relationship = ChunkOf)]
pub struct ChunkRoot(Vec<Entity>);

/// A wrapper type for `f32` representing the distance threshold for an LOD.
#[derive(Reflect, Debug, Deref, DerefMut)]
pub struct LodDistance(pub f32);

impl Default for LodDistance {
    /// Defaults to `f32::MAX`, indicating "visible to infinity".
    fn default() -> Self {
        f32::MAX.into()
    }
}

impl From<f32> for LodDistance {
    fn from(val: f32) -> Self {
        LodDistance(val)
    }
}

/// Wrapper type for `f32` representing the density multiplier for an LOD.
///
/// This value should be between 0.0 (nothing) and 1.0 (full density).
#[derive(Reflect, Debug, Deref, DerefMut, Clone)]
pub struct LodDensity(pub f32);

impl Default for LodDensity {
    fn default() -> Self {
        0.0.into()
    }
}

impl From<f32> for LodDensity {
    fn from(val: f32) -> Self {
        LodDensity(val)
    }
}

/// Wrapper type for `u32` representing the size scalar for a chunk at a specific LOD.
///
/// This is a multiplier relative to the [`BaseChunkSize`]. See also [`ChunkSize`].
#[derive(Reflect, Debug, Deref, DerefMut)]
pub struct ChunkSizeScalar(pub u32);

impl From<u32> for ChunkSizeScalar {
    fn from(val: u32) -> Self {
        ChunkSizeScalar(val)
    }
}

impl Default for ChunkSizeScalarConfig {
    fn default() -> Self {
        Self(
            // LODs are ordered from High (0) to Low (n).
            vec![
                // Level 0: High (1x base size)
                1.into(),
                // Level 1: Medium (2x base size)
                2.into(),
                // Level 2: Low (4x base size)
                4.into(),
                // Root (8x base size)
                // NOTE: interacts with chunk root size dim at the moment TODO
                8.into(),
            ],
        )
    }
}

impl ChunkSizeScalarConfig {
    /// Gets the size scalar `u32` for a given LOD `level` if it exists.
    pub fn get_size_scalar(&self, level: u32) -> Option<u32> {
        self.0.get(level as usize).map(|x| **x)
    }

    /// Returns the maximum LOD (index) defined by this configuration.
    pub fn get_max_lod(&self) -> u32 {
        (self.0.len() - 1) as u32
    }

    /// Gets the [`ChunkSizeScalar`] for a specific LOD `level`.
    pub fn get_scalar_config(&self, level: u32) -> &ChunkSizeScalar {
        &self.0[level as usize]
    }
}
