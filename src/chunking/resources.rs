use bevy::prelude::*;
use std::num::NonZeroU32;

pub struct LodConfig {
    /// The distance at which a chunk of this level subdivides into its children.
    pub distance: f32,
    /// The size of a chunk at this level, as a multiple of the highest-LOD chunk size.
    pub chunk_size_scalar: u32,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum LodLevel {
    High,
    Medium,
    Low,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            // LODs are ordered from High (0) to Low (n).
            lods: vec![
                // Level 0: High
                LodConfig {
                    distance: 50.0,
                    chunk_size_scalar: 1,
                },
                // Level 1: Medium
                LodConfig {
                    distance: 75.0,
                    chunk_size_scalar: 2,
                },
                // Level 2: Low
                LodConfig {
                    distance: f32::MAX,
                    chunk_size_scalar: 4,
                },
            ],
            base_chunk_size: 8.0,
            world_size_in_chunks: 8,
        }
    }
}


impl ChunkConfig {
    /// Calculates the world size (width/depth) of a chunk at a given LOD level.
    pub fn get_chunk_world_size(&self, level: u32) -> f32 {
        self.lods.get(level as usize).expect("Level out of bounds").chunk_size_scalar as f32 * self.base_chunk_size
    }

    pub fn get_total_world_size(&self) -> f32 {
        self.world_size_in_chunks as f32 * self.get_chunk_world_size(self.get_max_lod_level())
    }

    /// Calculates the world-space center of a chunk from its grid coordinate and level.
    pub fn get_chunk_world_center(&self, grid_coord: IVec3, level: u32) -> Vec3 {
        let world_size = self.get_chunk_world_size(level);
        let half_size = world_size / 2.0;

        grid_coord.as_vec3() * world_size + Vec3::new(half_size, 0.0, half_size)
    }

    pub fn get_size_scalar(&self, level: u32) -> u32 {
         self.lods.get(level as usize).expect("Level out of bounds").chunk_size_scalar
    }

    pub fn get_max_lod_level(&self) -> u32 {
        (self.lods.len() - 1) as u32
    }

    pub fn get_lod_config(&self, level: u32) -> &LodConfig {
        &self.lods[level as usize]
    }

    /// Calculates the level, size, and world-space offsets for a chunk's children.
    pub fn calculate_child_data(&self, level: NonZeroU32, size: u32) -> ChildChunkData {
        let parent_world_size = size as f32 * self.base_chunk_size;
        let child_level = level.get() - 1;
        let child_size = self.get_size_scalar(child_level);
        let offset = parent_world_size / 4.0;
        let offsets = [
            Vec3::new(-offset, 0.0, -offset),
            Vec3::new(offset, 0.0, -offset),
            Vec3::new(-offset, 0.0, offset),
            Vec3::new(offset, 0.0, offset),
        ];

        ChildChunkData {
            level: child_level,
            size: child_size,
            offsets,
        }
    }
}

pub struct ChildChunkData {
    pub level: u32,
    pub size: u32,
    pub offsets: [Vec3; 4],
}

#[derive(Resource)]
pub struct ChunkConfig {
    pub lods: Vec<LodConfig>,
    /// The size of the world in top-level (Low LOD) chunks.
    pub world_size_in_chunks: u32,
    pub base_chunk_size: f32,
}
