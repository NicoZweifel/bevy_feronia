use crate::core::components::LevelOfDetail;
use bevy_camera::prelude::Visibility;
use bevy_camera::visibility::VisibilityRange;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_math::{IVec2, Vec3};
use bevy_reflect::Reflect;
use bevy_transform::prelude::Transform;
use bevy_utils::default;
use derive_more::From;

#[cfg(feature = "trace")]
use tracing::warn;

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
                20.0.into(),
                // Level 1: Medium
                60.0.into(),
                // Level 2: Low
                180.into(),
                default(), // f32::MAX
            ],
            density: vec![
                // Level 0: High
                1.0.into(),
                // Level 1: Medium
                0.3.into(),
                // Level 2: Low
                0.1.into(),
                // Root
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
    fn get_lod_config(&self, level: u32) -> LodDistance {
        self.get().get(level as usize).cloned().unwrap_or_default()
    }

    /// Calculates the [`VisibilityRange`] for a given `lod`.
    fn get_visibility_range(&self, lod: LevelOfDetail) -> VisibilityRange {
        let current_lod_dist = *self.get_lod_config(*lod);

        // If this is too large, it causes artifacts/smearing with DLSS/TAA
        // TODO expose/create field
        let fade_band_multiplier = 0.05;

        let start_margin = if *lod == 0 {
            0.0..0.0
        } else {
            let prev_lod_dist = self
                .get()
                .get(*lod as usize - 1)
                .map(|d| **d)
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

/// Marker component indicating that a chunk is allowed to split into sub-chunks.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSplit;

/// Marker component indicating that a chunk is allowed to merge back into a parent chunk.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanMerge;

/// Component storing the current LOD of a chunk (0 is the highest detail).
#[derive(Component, Reflect, Deref, DerefMut, Default, Debug, Clone, Copy)]
#[reflect(Component, Clone, Debug)]
pub struct ChunkLevel(pub u32);

/// Component storing the scalar size in [`BaseChunkSize`] units of this chunk.
#[derive(Component, Reflect, Deref, DerefMut, Debug, Clone, Copy)]
#[reflect(Component, Debug, Clone)]
pub struct ChunkSize(pub u32);

/// Component storing the base size of a level 0 (the highest detail) chunk.
#[derive(Component, Reflect, Deref, DerefMut, Debug, Clone)]
#[reflect(Component, Debug, Clone)]
pub struct BaseChunkSize(pub Vec3);

impl BaseChunkSize {
    /// Calculate the bounding radius (half-diagonal) of a chunk.
    fn get_chunk_radius(&self, scalar: u32) -> f32 {
        let scalar = scalar as f32;

        let scaled_size = **self * scalar;

        let diagonal_sq = scaled_size.x.powi(2) + scaled_size.y.powi(2) + scaled_size.z.powi(2);
        let diagonal = diagonal_sq.sqrt();

        diagonal / 2.0
    }
}

/// Component specifying the distance at which a chunk should merge with its siblings.
///
/// Requires the [`CanMerge`] component.
#[derive(Component, Reflect, Deref, DerefMut, Debug, From)]
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
#[require(Transform, Visibility, ChunkInitialize)]
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
#[require(Transform, Visibility, ChunkSizeScalarConfig, ChunkRootSizeDim)]
#[relationship_target(relationship = ChunkOf)]
pub struct ChunkRoot(Vec<Entity>);

/// A wrapper type for `f32` representing the distance threshold for an LOD.
#[derive(Reflect, Debug, Deref, DerefMut, Clone, Copy, PartialEq, From)]
pub struct LodDistance(pub f32);

impl Default for LodDistance {
    /// Defaults to `f32::MAX`, indicating "visible to infinity".
    fn default() -> Self {
        f32::MAX.into()
    }
}

impl From<i32> for LodDistance {
    fn from(val: i32) -> Self {
        LodDistance(val as f32)
    }
}

/// Wrapper type for `f32` representing the density multiplier for an LOD.
///
/// This value should be between 0.0 (nothing) and 1.0 (full density).
#[derive(Reflect, Debug, Deref, DerefMut, Clone, Default, From)]
pub struct LodDensity(pub f32);

impl From<i32> for LodDensity {
    fn from(val: i32) -> Self {
        Self(val as f32)
    }
}

impl From<usize> for LodDensity {
    fn from(val: usize) -> Self {
        Self(val as f32)
    }
}

/// Component specifying the size multipliers for chunks at each LOD.
///
/// NOTE: Interacts with chunk root size dim at the moment.
/// Changing this might cause inconsistent/buggy behavior.
#[derive(Component, Reflect, Deref, Debug)]
#[reflect(Component)]
pub struct ChunkSizeScalarConfig(pub Vec<ChunkSizeScalar>);

/// Wrapper type for `u32` representing the size scalar for a chunk at a specific LOD.
///
/// This is a multiplier relative to the [`BaseChunkSize`]. See also [`ChunkSize`].
///
/// NOTE: Changing this might cause inconsistent/buggy behavior.
#[derive(Reflect, Debug, Deref, From)]
pub struct ChunkSizeScalar(pub u32);

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
                8.into(),
            ],
        )
    }
}

impl ChunkSizeScalarConfig {
    /// Gets the size scalar `u32` for a given LOD `level` if it exists.
    pub fn get_size_scalar(&self, level: u32) -> Option<u32> {
        self.0.get(level as usize).map(|s| **s)
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

/// Component specifying the LOD distance thresholds specifically for a chunk hierarchy.
#[derive(Component, Clone, Reflect, Deref, DerefMut, Debug, PartialEq)]
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
                90.0.into(),
                // Level 2: Low
                120.0.into(),
                // Level 3: Root
                default(), // f32::MAX
            ],
        )
    }
}

impl ChunkLodConfig {
    /// Creates a new `ChunkLodConfig` by combining base LOD distances
    /// with chunk radii calculated from size scalars and a base chunk size.
    ///
    /// This ensures LOD switches are "pop-free" by compensating for the
    /// chunk's physical size at each LOD and adding a buffer.
    pub fn from_sources(
        lod_config: &LodConfig,
        size_scalars: &ChunkSizeScalarConfig,
        base_size: &BaseChunkSize,
        buffer: f32,
    ) -> Self {
        let radii = size_scalars
            .iter()
            .map(|scalar| base_size.get_chunk_radius(**scalar))
            .collect::<Vec<_>>();

        lod_config
            .distance
            .iter()
            .enumerate()
            .map(|(i, base_dist)| {
                if **base_dist == f32::MAX {
                    return *base_dist;
                }

                radii
                    .get(i + 1)
                    .map(|r_parent|LodDistance(**base_dist + r_parent + buffer))
                    .unwrap_or_else(|| {
                        #[cfg(feature = "trace")]
                        warn!("Failed to calculate LOD distance for chunk at level {}. Using base distance.", i);
                        LodDistance(**base_dist + buffer)
                    })
            })
            .collect::<Vec<_>>().into()
    }
}

impl From<Vec<LodDistance>> for ChunkLodConfig {
    fn from(value: Vec<LodDistance>) -> Self {
        Self(value)
    }
}

impl LodConfiguration for ChunkLodConfig {
    fn get(&self) -> &Vec<LodDistance> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // For floating point comparisons
    const EPSILON: f32 = 0.0001;

    #[test]
    fn test_get_chunk_radius_1d_should_get_correct_radius() {
        // This is a 1D "sanity check" to test the simplest case.
        // A 1D "chunk" (a line) of length 20 should have a radius (half-diagonal) of 10.
        let base_size_1d = BaseChunkSize(Vec3::new(20.0, 0.0, 0.0));

        // sqrt(20^2 + 0^2 + 0^2) / 2 = 10.0
        assert!((base_size_1d.get_chunk_radius(1) - 10.0).abs() < EPSILON);
    }

    #[test]
    fn test_get_chunk_radius_3d_should_get_correct_radius() {
        let base_size_3d = BaseChunkSize(Vec3::new(10.0, 10.0, 10.0));

        // sqrt(10^2 + 10^2 + 10^2) / 2 = sqrt(300) / 2 = 8.66025...
        let r1 = base_size_3d.get_chunk_radius(1);

        assert!((r1 - 8.66025).abs() < EPSILON);
    }

    #[test]
    fn test_get_chunk_radius_should_scale() {
        let base_size_3d = BaseChunkSize(Vec3::new(10.0, 10.0, 10.0));

        // sqrt(10^2 + 10^2 + 10^2) / 2 = sqrt(300) / 2 = 8.66025...
        let r1 = base_size_3d.get_chunk_radius(1);
        // Should be 2 * r1
        let r2 = base_size_3d.get_chunk_radius(2);

        assert!((r2 - 17.3205).abs() < EPSILON);
        assert!((r2 - (r1 * 2.0)).abs() < EPSILON);
    }

    #[test]
    fn test_from_sources_standard_should_calculate_correctly() {
        // Arrange
        // Use a simple 1D chunk for easy math: radius = scalar * 10
        let base_size = BaseChunkSize(Vec3::new(20.0, 0.0, 0.0));

        // Radii list will be: R = [10.0, 20.0, 40.0, 80.0]
        let size_scalars = ChunkSizeScalarConfig(vec![
            1.into(), // R0 (i=0) = 10.0
            2.into(), // R1 (i=1) = 20.0
            4.into(), // R2 (i=2) = 40.0
            8.into(), // R3 (i=3) = 80.0
        ]);

        // Ideal distances (D)
        let lod_config = LodConfig {
            distance: vec![
                30.0.into(),  // D0 (i=0)
                60.0.into(),  // D1 (i=1)
                120.0.into(), // D2 (i=2)
                default(),    // f32::MAX
            ],
            ..default()
        };

        let buffer = 5.0;

        // CD_i = D_i + R_{i+1} + Buffer
        // CD_0 = 30.0 + 20.0 + 5.0 = 55.0
        // CD_1 = 60.0 + 40.0 + 5.0 = 105.0
        // CD_2 = 120.0 + 80.0 + 5.0 = 205.0
        // CD_3 = f32::MAX
        let expected_config = ChunkLodConfig(vec![
            55.0.into(),
            105.0.into(),
            205.0.into(),
            LodDistance::default(),
        ]);

        // Act
        let calculated_config =
            ChunkLodConfig::from_sources(&lod_config, &size_scalars, &base_size, buffer);

        // Assert
        assert_eq!(calculated_config, expected_config);
    }

    #[test]
    fn test_from_sources_should_fallback_when_scalars_too_short() {
        // Arrange
        let base_size = BaseChunkSize(Vec3::new(20.0, 0.0, 0.0));

        // Radii list: R = [10.0, 20.0]
        // This list is too short!
        let size_scalars = ChunkSizeScalarConfig(vec![
            1.into(), // R0 (i=0)
            2.into(), // R1 (i=1)
        ]);

        let lod_config = LodConfig {
            distance: vec![
                30.0.into(),  // D0 (i=0)
                60.0.into(),  // D1 (i=1)
                120.0.into(), // D2 (i=2)
                default(),    // f32::MAX
            ],
            ..default()
        };
        let buffer = 10.0;

        // CD_0 = D_0 + R_1 = 30.0 + 20.0 + 10.0 = 60.0
        // CD_1 = D_1. `radii.get(2)` is None. Fallback: pushes `D_1` (70.0)
        // CD_2 = D_2. `radii.get(3)` is None. Fallback: pushes `D_2` (130.0)
        // CD_3 = f32::MAX
        let expected_config = ChunkLodConfig(vec![
            60.0.into(),
            70.0.into(),
            130.0.into(),
            LodDistance::default(),
        ]);

        // Act
        let calculated_config =
            ChunkLodConfig::from_sources(&lod_config, &size_scalars, &base_size, buffer);

        // Assert
        assert_eq!(calculated_config, expected_config);
    }
}
