use crate::core::LevelOfDetail;
use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct LodConfig(pub Vec<LodLevelDistance>);

impl LodConfiguration for LodConfig {
    fn get(&self) -> &Vec<LodLevelDistance> {
        &self.0
    }
}

impl Default for LodConfig {
    fn default() -> Self {
        Self(
            // LODs are ordered from High (0) to Low (n).
            vec![
                50.0.into(),
                // Level 1: Medium
                100.0.into(),
                // Level 2: Low
                LodLevelDistance::default(),
            ],
        )
    }
}

pub trait LodConfiguration {
    fn get(&self) -> &Vec<LodLevelDistance>;

    fn get_max_lod_level(&self) -> u32 {
        (self.get().len() - 1) as u32
    }

    fn get_lod_config(&self, level: u32) -> &LodLevelDistance {
        &self.get()[level as usize]
    }

    fn get_visibility_range(&self, lod_level: LevelOfDetail) -> VisibilityRange {
        let current_lod_dist = self
            .get()
            .get(*lod_level as usize)
            .map(|x| **x)
            .unwrap_or(*LodLevelDistance::default());

        let fade_band = current_lod_dist * 0.1;

        let start_margin = if *lod_level == 0 {
            0.0..0.0
        } else {
            let prev_lod_dist = self
                .get()
                .get(*lod_level as usize - 1)
                .map(|x| **x)
                .unwrap_or(*LodLevelDistance::default());

            prev_lod_dist - fade_band..prev_lod_dist
        };

        let end_margin = if *lod_level == self.get_max_lod_level() {
            f32::MAX..f32::MAX
        } else {
            current_lod_dist - fade_band..current_lod_dist
        };

        VisibilityRange {
            start_margin,
            end_margin,
            use_aabb: true,
        }
    }
}

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkLodConfig(pub Vec<LodLevelDistance>);

impl Default for ChunkLodConfig {
    fn default() -> Self {
        Self(
            // LODs are ordered from High (0) to Low (n).
            vec![
                60.0.into(),
                // Level 1: Medium
                120.0.into(),
                // Level 2: Low
                180.0.into(),
                LodLevelDistance::default(),
            ],
        )
    }
}

impl LodConfiguration for ChunkLodConfig {
    fn get(&self) -> &Vec<LodLevelDistance> {
        &self.0
    }
}

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkSizeScalarConfig(pub Vec<ChunkSizeScalar>);

/// The size of a `ChunkRoot` dimension in top-level (Low LOD) chunks.
/// TODO use/sync with ChunkSizeScalar
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkRootSizeDim(pub u32);

impl Default for ChunkRootSizeDim {
    fn default() -> Self {
        Self(2)
    }
}

#[derive(Component, Reflect, Deref, DerefMut, Debug, Hash)]
#[reflect(Component)]
pub struct ChunkCoord(pub IVec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ChunkInitialize;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkCenter;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSplit;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanMerge;

#[derive(Component, Reflect, Deref, DerefMut, Default, Debug, Clone)]
#[reflect(Component)]
pub struct ChunkLevel(pub u32);

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkSize(pub u32);

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct BaseChunkSize(pub Vec3);

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[require(CanMerge)]
#[reflect(Component)]
pub struct MergeDistance(pub f32);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Merging;

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[require(CanSplit)]
#[reflect(Component)]
pub struct SplitDistance(pub f32);

#[derive(Component, Debug, Clone, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
#[derive(Default)]
pub struct Chunk;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ChunkRoot)]
pub struct ChunkOf(pub Entity);

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

/// The distance until this LOD Level is visible.
#[derive(Reflect, Debug, Deref, DerefMut)]
pub struct LodLevelDistance(pub f32);

/// The size of a chunk at this level, as a multiple of the highest-LOD chunk size.
#[derive(Reflect, Debug, Deref, DerefMut)]
pub struct ChunkSizeScalar(pub u32);

impl Default for LodLevelDistance {
    fn default() -> Self {
        f32::MAX.into()
    }
}

impl From<f32> for LodLevelDistance {
    fn from(val: f32) -> Self {
        LodLevelDistance(val)
    }
}

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
                1.into(),
                // Level 1: Medium
                2.into(),
                // Level 2: Low
                4.into(),
                // Root NOTE: interacts with chunk root size dim at the moment TODO
                8.into(),
            ],
        )
    }
}

impl ChunkSizeScalarConfig {
    pub fn get_size_scalar(&self, level: u32) -> Option<u32> {
        self.0.get(level as usize).map(|x| **x)
    }

    pub fn get_max_lod_level(&self) -> u32 {
        (self.0.len() - 1) as u32
    }

    pub fn get_scalar_config(&self, level: u32) -> &ChunkSizeScalar {
        &self.0[level as usize]
    }
}
