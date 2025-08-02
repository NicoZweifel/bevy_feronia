use crate::prelude::*;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;

pub fn scatter_observer(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: Res<WindAffectedTypes<ExtendedWindAffectedMaterial>>,

    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<&ChunkConfig, With<ChunkRoot>>,
) {
    crate::observers::scatter_observer::<
        WindAffectedTypes<ExtendedWindAffectedMaterial>,
        WindAffectedType<ExtendedWindAffectedMaterial>,
        StandardMaterial,
        ExtendedWindAffectedMaterial,
    >(trigger, cmd, &prototypes, q_chunks, q_chunk_config);
}
