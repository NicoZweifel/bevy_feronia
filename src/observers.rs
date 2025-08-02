use crate::core::WindAffectable;
use crate::prelude::*;
use bevy::asset::Asset;
use bevy::pbr::{Material, MeshMaterial3d};
use bevy::prelude::*;

pub fn scatter_observer<T, P, M, A>(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: &Res<T>,
    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<&ChunkConfig, With<ChunkRoot>>,
) where
    T: Resource + ProtoTypes<A, P>,
    P: ProtoType<A>,
    M: Material,
    A: Asset + Clone + WindAffectable<M, A, T, P>,
{
    A::spawn(cmd, trigger, prototypes, q_chunks, q_chunk_config);
}
