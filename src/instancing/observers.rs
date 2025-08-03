use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::render::primitives::Aabb;
use bevy::prelude::*;

pub fn scatter_observer<T, P>(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: Res<T>,
    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
) where
    T: Resource + ProtoTypes<InstancedWindAffectedMaterial, P>,
    P: ProtoType<InstancedWindAffectedMaterial>,
{
    crate::scatter::observers::scatter_observer::<T, P, StandardMaterial, InstancedWindAffectedMaterial>(
        trigger,
        cmd,
        &prototypes,
        q_chunks,
        q_chunk_config,
    );
}

pub fn wind_affected_scatter_observer(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: Res<WindAffectedTypes<InstancedWindAffectedMaterial>>,
    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
) {
    scatter_observer::<
        WindAffectedTypes<InstancedWindAffectedMaterial>,
        WindAffectedType<InstancedWindAffectedMaterial>,
    >(trigger, cmd, prototypes, q_chunks, q_chunk_config);
}
