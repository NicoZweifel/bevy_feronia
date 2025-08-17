use crate::prelude::*;
use bevy::prelude::*;

pub fn on_add_chunk(trigger: On<Add, Chunk>, mut cmd: Commands, q_chunk: Query<&ChunkCoord>) {
    let chunk = trigger.target();

    debug!("Chunk added: {chunk}.");

    // TODO use for Deterministic Scattering Hash/Seed
    let Ok(_) = q_chunk.get(chunk) else {
        warn!("Chunk {chunk} has no chunk coord. Can't initialize Chunk without Coordinates.");
        return;
    };

    cmd.entity(chunk).insert(ChunkInitialize);
}

pub fn chunk_init(
    mut cmd: Commands,
    q_chunks: Query<Entity, (With<Chunk>, With<ChunkInitialize>)>,
) {
    for chunk in q_chunks.iter() {
        debug!("Chunk initialized: {chunk}.");

        cmd.entity(chunk).remove::<ChunkInitialize>();
    }
}
