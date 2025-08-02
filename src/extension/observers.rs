use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::camera::primitives::Aabb;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

pub fn scatter_observer(
    trigger: On<ScatterResults>,
    cmd: Commands,
    prototypes: Res<WindAffectedTypes<ExtendedWindAffectedMaterial>>,

    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
) {
    crate::observers::scatter_observer::<
        WindAffectedTypes<ExtendedWindAffectedMaterial>,
        WindAffectedType<ExtendedWindAffectedMaterial>,
        StandardMaterial,
        ExtendedWindAffectedMaterial,
    >(trigger, cmd, &prototypes, q_chunks, q_chunk_config);
}
