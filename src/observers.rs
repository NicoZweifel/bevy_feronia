use crate::core::WindAffectable;
use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::asset::Asset;
use bevy::camera::primitives::Aabb;
use bevy::pbr::Material;
use bevy::prelude::*;

pub fn scatter_observer<T, P, M, A>(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: &Res<T>,
    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
) where
    T: Resource + ProtoTypes<A, P>,
    P: ProtoType<A>,
    M: Material,
    A: Asset + Clone + WindAffectable<M, A, T, P>,
{
    A::spawn(cmd, trigger, prototypes, q_chunks, q_chunk_config);
}
