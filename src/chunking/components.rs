use crate::core::Sampler;
use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::HeightMap;
use bevy::prelude::*;
use std::num::NonZeroU32;

#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkLodConfig(Vec<LodConfig>);

/// The size of the `ChunkRoot` in top-level (Low LOD) chunks.
#[derive(Component, Reflect, Deref, DerefMut, Debug)]
#[reflect(Component)]
pub struct ChunkRootSize(pub u32);

impl Default for ChunkRootSize {
    fn default() -> Self {
        Self(8)
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
pub struct CanMerge;

#[derive(Component, Reflect, Deref, DerefMut, Default, Debug)]
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
#[require(Transform, Visibility, ChunkLodConfig, ChunkRootSize)]
#[relationship_target(relationship = ChunkOf)]
pub struct ChunkRoot(Vec<Entity>);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct LodConfig {
    /// The distance at which a chunk of this level is merged.
    pub distance: f32,
    /// The size of a chunk at this level, as a multiple of the highest-LOD chunk size.
    pub chunk_size_scalar: u32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            distance: f32::MAX,
            chunk_size_scalar: 4,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum LodLevel {
    High,
    Medium,
    Low,
}

impl Default for ChunkLodConfig {
    fn default() -> Self {
        Self(
            // LODs are ordered from High (0) to Low (n).
            vec![
                // Level 0: High
                LodConfig {
                    distance: 30.0,
                    chunk_size_scalar: 1,
                },
                // Level 1: Medium
                LodConfig {
                    distance: 60.0,
                    chunk_size_scalar: 2,
                },
                // Level 2: Low
                LodConfig {
                    distance: f32::MAX,
                    chunk_size_scalar: 4,
                },
            ],
        )
    }
}

impl ChunkLodConfig {
    pub fn get_size_scalar(&self, level: u32) -> u32 {
        self.0
            .get(level as usize)
            .expect("Level out of bounds")
            .chunk_size_scalar
    }

    pub fn get_max_lod_level(&self) -> u32 {
        (self.0.len() - 1) as u32
    }

    pub fn get_lod_config(&self, level: u32) -> &LodConfig {
        &self.0[level as usize]
    }

    /// Calculates the level, size, and world-space offsets for a chunk's children.
    pub fn calculate_child_data(
        &self,
        level: NonZeroU32,
        size: u32,
        base_chunk_size: Vec3,
    ) -> ChildChunkData {
        let parent_world_size = size as f32 * base_chunk_size;
        let child_level = level.get() - 1;
        let child_size = self.get_size_scalar(child_level);
        let offset = parent_world_size / 4.0;
        let offsets = [
            Vec3::new(-offset.x, 0.0, -offset.z),
            Vec3::new(offset.x, 0.0, -offset.z),
            Vec3::new(-offset.x, 0.0, offset.z),
            Vec3::new(offset.x, 0.0, offset.z),
        ];

        ChildChunkData {
            level: child_level,
            size: child_size,
            offsets,
        }
    }

    pub fn get_height_map_sampler(
        &self,
        images: &Res<Assets<Image>>,
        height_map: &Option<Res<HeightMap>>,
        total_world_size: f32,
    ) -> Option<impl Sampler> {
        match height_map {
            None => None,
            Some(x) => match images.get(&x.0) {
                None => None,
                Some(img) => Some(HeightMapCpuSampler::new(img, total_world_size)),
            },
        }
    }
}

pub struct ChildChunkData {
    pub level: u32,
    pub size: u32,
    pub offsets: [Vec3; 4],
}
