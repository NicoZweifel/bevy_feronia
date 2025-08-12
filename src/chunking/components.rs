use bevy::prelude::*;
use std::num::NonZeroU32;

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct LodConfig(pub Vec<LodLevelDistance>);

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkSizeScalarConfig(pub Vec<ChunkSizeScalar>);

/// The size of the `ChunkRoot` in top-level (Low LOD) chunks.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkRootSize(pub u32);

impl Default for ChunkRootSize {
    fn default() -> Self {
        Self(16)
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkCenter;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CanSplit;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ChunkInitialize;

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
#[require(Transform, Visibility, LodConfig, ChunkSizeScalarConfig, ChunkRootSize)]
#[relationship_target(relationship = ChunkOf)]
pub struct ChunkRoot(Vec<Entity>);

/// The distance at which a chunk of this level is merged.
#[derive(Reflect, Debug, Deref, DerefMut)]
pub struct LodLevelDistance(pub f32);

/// The size of a chunk at this level, as a multiple of the highest-LOD chunk size.
#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct ChunkSizeScalar(pub u32);

impl Default for LodLevelDistance {
    fn default() -> Self {
        f32::MAX.into()
    }
}

impl Into<LodLevelDistance> for f32 {
    fn into(self) -> LodLevelDistance {
        LodLevelDistance(self)
    }
}

impl Default for LodConfig {
    fn default() -> Self {
        Self(
            // LODs are ordered from High (0) to Low (n).
            vec![
                30.0.into(),
                // Level 1: Medium
                90.0.into(),
                // Level 2: Low
                LodLevelDistance::default(),
            ],
        )
    }
}

impl Default for ChunkSizeScalar {
    fn default() -> Self {
        4.into()
    }
}

impl Into<ChunkSizeScalar> for u32 {
    fn into(self) -> ChunkSizeScalar {
        ChunkSizeScalar(self)
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
                ChunkSizeScalar::default(),
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

impl LodConfig {
    pub fn get_max_lod_level(&self) -> u32 {
        (self.0.len() - 1) as u32
    }

    pub fn get_lod_config(&self, level: u32) -> &LodLevelDistance {
        &self.0[level as usize]
    }

    /// Calculates the level, size, and world-space offsets for a chunk's children.
    pub fn calculate_child_data(
        &self,
        level: NonZeroU32,
        size: u32,
        base_chunk_size: Vec3,
        child_size_scalar: u32,
    ) -> ChildChunkData {
        let parent_world_size = size as f32 * base_chunk_size;
        let child_level = level.get() - 1;
        let offset = parent_world_size / 4.0;
        let offsets = [
            Vec3::new(-offset.x, 0.0, -offset.z),
            Vec3::new(offset.x, 0.0, -offset.z),
            Vec3::new(-offset.x, 0.0, offset.z),
            Vec3::new(offset.x, 0.0, offset.z),
        ];

        ChildChunkData {
            level: child_level,
            size: child_size_scalar,
            offsets,
        }
    }
}

pub struct ChildChunkData {
    pub level: u32,
    pub size: u32,
    pub offsets: [Vec3; 4],
}
