use crate::prelude::*;
use bevy_ecs::prelude::*;

pub fn update_root_lod_config(
    mut cmd: Commands,
    q_root: Query<(Entity, &LodConfig, &ChunkSizeScalarConfig, &BaseChunkSize), Changed<LodConfig>>,
) {
    for (entity, lod_config, size_scalars, base_size) in &q_root {
        let derived_chunk_lod_config =
            ChunkLodConfig::from_sources(lod_config, size_scalars, base_size);

        cmd.entity(entity).insert(derived_chunk_lod_config);
    }
}
