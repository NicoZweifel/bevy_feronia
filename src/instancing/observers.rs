use crate::prelude::{
    Chunk, ChunkLevel, ChunkLodConfig, ChunkOf, ChunkRoot, ExtendedWindAffectedMaterial,
    InstancedWindAffectedMaterial, ScatterResults, WindAffectedType, WindAffectedTypes,
};
use bevy::camera::primitives::Aabb;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Commands, GlobalTransform, On, Query, Res, With};

pub fn scatter_observer(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: Res<WindAffectedTypes<InstancedWindAffectedMaterial>>,

    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
) {
    crate::observers::scatter_observer::<
        WindAffectedTypes<InstancedWindAffectedMaterial>,
        WindAffectedType<InstancedWindAffectedMaterial>,
        StandardMaterial,
        InstancedWindAffectedMaterial,
    >(trigger, cmd, &prototypes, q_chunks, q_chunk_config);
}
